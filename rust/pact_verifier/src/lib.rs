//! The `pact_verifier` crate provides the core logic to performing verification of providers.
//! It implements the V3 Pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3).
#![type_length_limit="4776643"]
#![warn(missing_docs)]

mod provider_client;
mod pact_broker;
pub mod callback_executors;

use std::path::Path;
use std::io;
use std::fs;
use std::fmt::{Display, Formatter};
use pact_matching::*;
use pact_matching::models::*;
use pact_matching::models::provider_states::*;
use pact_matching::models::http_utils::HttpAuth;
use ansi_term::*;
use ansi_term::Colour::*;
use std::collections::HashMap;
use crate::provider_client::{make_provider_request, provider_client_error_to_string};
use regex::Regex;
use serde_json::Value;
use crate::pact_broker::{publish_verification_results, TestResult, Link};
use maplit::*;
use futures::stream::*;
use callback_executors::RequestFilterExecutor;
pub use callback_executors::NullRequestFilterExecutor;
use crate::callback_executors::{ProviderStateExecutor, ProviderStateError};
use log::*;
use futures::executor::block_on;

/// Source for loading pacts
#[derive(Debug, Clone)]
pub enum PactSource {
    /// Unknown pact source
    Unknown,
    /// Load the pact from a pact file
    File(String),
    /// Load all the pacts from a Directory
    Dir(String),
    /// Load the pact from a URL
    URL(String, Option<HttpAuth>),
    /// Load all pacts with the provider name from the pact broker url
    BrokerUrl(String, String, Option<HttpAuth>, Vec<Link>)
}

impl Display for PactSource {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match *self {
      PactSource::File(ref file) => write!(f, "File({})", file),
      PactSource::Dir(ref dir) => write!(f, "Dir({})", dir),
      PactSource::URL(ref url, _) => write!(f, "URL({})", url),
      PactSource::BrokerUrl(ref provider_name, ref broker_url, _, _) => {
          write!(f, "PactBroker({}, provider_name='{}')", broker_url, provider_name)
      }
      _ => write!(f, "Unknown")
    }
  }
}

/// Information about the Provider to verify
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    /// Provider Name
    pub name: String,
    /// Provider protocol, defaults to HTTP
    pub protocol: String,
    /// Hostname of the provider
    pub host: String,
    /// Port the provider is running on, defaults to 8080
    pub port: Option<u16>,
    /// Base path for the provider, defaults to /
    pub path: String
}

impl Default for ProviderInfo {
    /// Create a default provider info
  fn default() -> ProviderInfo {
        ProviderInfo {
            name: s!("provider"),
            protocol: s!("http"),
            host: s!("localhost"),
            port: Some(8080),
            path: s!("/")
        }
    }
}

/// Result of performing a match
#[derive(Debug, Clone)]
pub enum MismatchResult {
    /// Response mismatches
    Mismatches {
      /// Mismatches that occurred
      mismatches: Vec<Mismatch>,
      /// Expected Response
      expected: Response,
      /// Actual Response
      actual: Response,
      /// Interaction ID if fetched from a pact broker
      interaction_id: Option<String>
    },
    /// Error occurred
    Error(String, Option<String>)
}

impl MismatchResult {
  /// Return the interaction ID associated with the error, if any
  pub fn interaction_id(&self) -> Option<String> {
    match *self {
      MismatchResult::Mismatches { ref interaction_id, .. } => interaction_id.clone(),
      MismatchResult::Error(_, ref interaction_id) => interaction_id.clone()
    }
  }
}

impl From<ProviderStateError> for MismatchResult {
  fn from(error: ProviderStateError) -> Self {
    MismatchResult::Error(error.description, error.interaction_id)
  }
}

async fn verify_response_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &Interaction,
  options: &VerificationOptions<F>,
  client: &reqwest::Client
) -> Result<(), MismatchResult> {
  let expected_response = &interaction.response;
  match make_provider_request(provider, &pact_matching::generate_request(&interaction.request, &hashmap!{}), options, client).await {
    Ok(ref actual_response) => {
      let mismatches = match_response(expected_response.clone(), actual_response.clone());
      if mismatches.is_empty() {
        Ok(())
      } else {
        Err(MismatchResult::Mismatches {
          mismatches,
          expected: expected_response.clone(),
          actual: actual_response.clone(),
          interaction_id: interaction.id.clone()
        })
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(provider_client_error_to_string(err), interaction.id.clone()))
    }
  }
}

async fn execute_state_change<S: ProviderStateExecutor>(
  provider_state: &ProviderState,
  setup: bool,
  interaction_id: Option<String>,
  client: &reqwest::Client,
  provider_state_executor: &S
) -> Result<HashMap<String, Value>, MismatchResult> {
    if setup {
        println!("  Given {}", Style::new().bold().paint(provider_state.name.clone()));
    }
    let result = provider_state_executor.call(interaction_id, provider_state, setup, Some(client)).await;
    log::debug!("State Change: \"{:?}\" -> {:?}", provider_state, result);
    result.map_err(|err| MismatchResult::Error(err.description, err.interaction_id))
}

fn verify_interaction<F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider: &ProviderInfo,
  interaction: Interaction,
  options: &VerificationOptions<F>,
  provider_state_executor: &S
) -> Result<(), MismatchResult> {
  let client = reqwest::Client::new();

  if !interaction.provider_states.is_empty() {
    info!("Running provider state change handlers for '{}'", interaction.description);
    let interaction_id = interaction.id.clone();
    let sc_result: Vec<Result<HashMap<String, Value>, MismatchResult>> = block_on(
      futures::stream::iter(interaction.provider_states.iter())
        .then(|state| {
          execute_state_change(&state, true, interaction_id.clone(), &client, provider_state_executor)
        }).collect());

    if sc_result.iter().any(|result| result.is_err()) {
      return Err(MismatchResult::Error("One or more of the state change handlers has failed".to_string(), interaction.id))
    }
  }

  info!("Running provider verification for '{}'", interaction.description);
  let result = block_on(verify_response_from_provider(provider, &interaction, options, &client));

  if !interaction.provider_states.is_empty() {
    info!("Running provider state change handler teardowns for '{}'", interaction.description);
    let interaction_id = interaction.id.clone();
    let sc_teardown_result: Vec<Result<HashMap<String, Value>, MismatchResult>> = block_on(
      futures::stream::iter(interaction.provider_states.iter())
        .then(|state| {
          execute_state_change(&state, false, interaction_id.clone(), &client, provider_state_executor)
        }).collect());

    if sc_teardown_result.iter().any(|result| result.is_err()) {
      return Err(MismatchResult::Error("One or more of the state change handlers has failed during teardown phase".to_string(), interaction.id))
    }
  }

  result
}

fn display_result(status: u16, status_result: ANSIGenericString<str>,
  header_results: Option<Vec<(String, String, ANSIGenericString<str>)>>,
  body_result: ANSIGenericString<str>) {
  println!("    returns a response which");
  println!("      has status code {} ({})", Style::new().bold().paint(format!("{}", status)),
      status_result);
  if let Some(header_results) = header_results {
    println!("      includes headers");
    for (key, value, result) in header_results {
      println!("        \"{}\" with value \"{}\" ({})", Style::new().bold().paint(key),
               Style::new().bold().paint(value), result);
    }
  }
  println!("      has a matching body ({})", body_result);
}

fn walkdir(dir: &Path) -> io::Result<Vec<io::Result<Pact>>> {
    let mut pacts = vec![];
    log::debug!("Scanning {:?}", dir);
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walkdir(&path)?;
        } else {
            pacts.push(Pact::read_pact(&path))
        }
    }
    Ok(pacts)
}

fn display_body_mismatch(expected: &Response, actual: &Response, path: &str) {
  if let DetectedContentType::Json = expected.content_type_enum() {
    println!("{}", pact_matching::json::display_diff(&expected.body.str_value().to_string(),
                                                     &actual.body.str_value().to_string(), path));
  }
}

/// Filter information used to filter the interactions that are verified
#[derive(Debug, Clone)]
pub enum FilterInfo {
    /// No filter, all interactions will be verified
    None,
    /// Filter on the interaction description
    Description(String),
    /// Filter on the interaction provider state
    State(String),
    /// Filter on both the interaction description and provider state
    DescriptionAndState(String, String)
}

impl FilterInfo {

    /// If this filter is filtering on description
    pub fn has_description(&self) -> bool {
        match *self {
            FilterInfo::Description(_) => true,
            FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// If this filter is filtering on provider state
    pub fn has_state(&self) -> bool {
        match *self {
            FilterInfo::State(_) => true,
            FilterInfo::DescriptionAndState(_, _) => true,
            _ => false
        }
    }

    /// Value of the state to filter
    pub fn state(&self) -> String {
        match *self {
            FilterInfo::State(ref s) => s.clone(),
            FilterInfo::DescriptionAndState(_, ref s) => s.clone(),
            _ => s!("")
        }
    }

    /// Value of the description to filter
    pub fn description(&self) -> String {
        match *self {
            FilterInfo::Description(ref s) => s.clone(),
            FilterInfo::DescriptionAndState(ref s, _) => s.clone(),
            _ => s!("")
        }
    }

    /// If the filter matches the interaction provider state using a regular expression. If the
    /// filter value is the empty string, then it will match interactions with no provider state.
    ///
    /// # Panics
    /// If the state filter value can't be parsed as a regular expression
    pub fn match_state(&self, interaction: &Interaction) -> bool {
        if !interaction.provider_states.is_empty() {
            if self.state().is_empty() {
                false
            } else {
                let re = Regex::new(&self.state()).unwrap();
                interaction.provider_states.iter().any(|state| re.is_match(&state.name))
            }
        } else {
            self.has_state() && self.state().is_empty()
        }
    }

    /// If the filter matches the interaction description using a regular expression
    ///
    /// # Panics
    /// If the description filter value can't be parsed as a regular expression
    pub fn match_description(&self, interaction: &Interaction) -> bool {
        let re = Regex::new(&self.description()).unwrap();
        re.is_match(&interaction.description)
    }

}

fn filter_interaction(interaction: &Interaction, filter: &FilterInfo) -> bool {
    if filter.has_description() && filter.has_state() {
      filter.match_description(interaction) && filter.match_state(interaction)
    } else if filter.has_description() {
      filter.match_description(interaction)
    } else if filter.has_state() {
      filter.match_state(interaction)
    } else {
      true
    }
}

fn filter_consumers(consumers: &[String], res: &Result<(Pact, PactSource), String>) -> bool {
    consumers.is_empty() || res.is_err() || consumers.contains(&res.clone().unwrap().0.consumer.name)
}

/// Options to use when running the verification
#[derive(Debug, Clone)]
pub struct VerificationOptions<F> where F: RequestFilterExecutor {
    /// If results should be published back to the broker
    pub publish: bool,
    /// Provider version being published
    pub provider_version: Option<String>,
    /// Build URL to associate with the published results
    pub build_url: Option<String>,
    /// Request filter callback
    pub request_filter: Option<Box<F>>,
    /// Tags to use when publishing results
    pub provider_tags: Vec<String>
}

impl <F: RequestFilterExecutor> Default for VerificationOptions<F> {
  fn default() -> Self {
    VerificationOptions {
      publish: false,
      provider_version: None,
      build_url: None,
      request_filter: None,
      provider_tags: vec![]
    }
  }
}

/// Verify the provider with the given pact sources
pub async fn verify_provider<F: RequestFilterExecutor, S: ProviderStateExecutor>(
    provider_info: ProviderInfo,
    source: Vec<PactSource>,
    filter: FilterInfo,
    consumers: Vec<String>,
    options: VerificationOptions<F>,
    provider_state_executor: &S
) -> bool {
    let pact_results = fetch_pacts(source, consumers).await;

    let mut all_errors: Vec<(String, MismatchResult)> = vec![];
    for pact_result in pact_results {
        match pact_result {
            Ok((pact, pact_source)) => {
                println!("\nVerifying a pact between {} and {}",
                    Style::new().bold().paint(pact.consumer.name.clone()),
                    Style::new().bold().paint(pact.provider.name.clone()));

                if pact.interactions.is_empty() {
                    println!("         {}", Yellow.paint("WARNING: Pact file has no interactions"));
                } else {
                    let errors = verify_pact(&provider_info, &filter, pact, &options, provider_state_executor).await;
                    for error in errors.clone() {
                        all_errors.push(error);
                    }

                    if options.publish {
                        publish_result(&errors, &pact_source, &options).await
                    }
                }
            },
            Err(err) => {
                log::error!("Failed to load pact - {}", Red.paint(err.to_string()));
                all_errors.push((s!("Failed to load pact"), MismatchResult::Error(err.to_string(), None)));
            }
        }
    };

    if !all_errors.is_empty() {
        println!("\nFailures:\n");

        for (i, &(ref description, ref mismatch)) in all_errors.iter().enumerate() {
            match *mismatch {
                MismatchResult::Error(ref err, _) => println!("{}) {} - {}\n", i, description, err),
                MismatchResult::Mismatches { ref mismatches, ref expected, ref actual, .. } => {
                    let mismatch = mismatches.first().unwrap();
                    println!("{}) {}{}", i, description, mismatch.summary());
                    for mismatch in mismatches {
                        println!("    {}\n", mismatch.ansi_description());
                    }

                    if let Mismatch::BodyMismatch{ref path, ..} = mismatch {
                      display_body_mismatch(expected, actual, path);
                    }
                }
            }
        }

        println!("\nThere were {} pact failures\n", all_errors.len());
        false
    } else {
        true
    }
}

async fn fetch_pact(
    source: PactSource
) -> Vec<Result<(Pact, PactSource), String>> {
    match source {
        PactSource::File(ref file) => vec![Pact::read_pact(Path::new(&file))
            .map_err(|err| format!("Failed to load pact '{}' - {}", file, err))
            .map(|pact| (pact, source))],
        PactSource::Dir(ref dir) => match walkdir(Path::new(dir)) {
            Ok(pact_results) => pact_results.into_iter().map(|pact_result| {
                match pact_result {
                    Ok(pact) => Ok((pact, source.clone())),
                    Err(err) => Err(format!("Failed to load pact from '{}' - {}", dir, err))
                }
            }).collect(),
            Err(err) => vec![Err(format!("Could not load pacts from directory '{}' - {}", dir, err))]
        },
        PactSource::URL(ref url, ref auth) => vec![Pact::from_url(url, auth)
            .map_err(|err| format!("Failed to load pact '{}' - {}", url, err))
            .map(|pact| (pact, source))],
        PactSource::BrokerUrl(ref provider_name, ref broker_url, ref auth, _) => {
            let result = pact_broker::fetch_pacts_from_broker(
                broker_url.clone(),
                provider_name.clone(),
                auth.clone()
            ).await;

            match result {
                Ok(ref pacts) => pacts.iter().map(|p| {
                    match *p {
                        Ok((ref pact, ref links)) => {
                            log::debug!("Got pact with links {:?}", links);
                            Ok((pact.clone(), PactSource::BrokerUrl(provider_name.clone(), broker_url.clone(), auth.clone(), links.clone())))
                        },
                        Err(ref err) => Err(format!("Failed to load pact from '{}' - {:?}", broker_url, err))
                    }
                }).collect(),
                Err(err) => vec![Err(format!("Could not load pacts from the pact broker '{}' - {:?}", broker_url, err))]
            }
        },
        _ => vec![Err("Could not load pacts, unknown pact source".to_string())]
    }
}

async fn fetch_pacts(
    source: Vec<PactSource>,
    consumers: Vec<String>
) -> Vec<Result<(Pact, PactSource), String>> {
    futures::stream::iter(source)
        // .map(|pact_source| pact_source.clone())
        .then(|pact_source| async {
            futures::stream::iter(fetch_pact(pact_source).await)
        })
        .flatten()
        .filter(|res| futures::future::ready(filter_consumers(&consumers, res)))
        .collect()
        .await
}

async fn verify_pact<F: RequestFilterExecutor, S: ProviderStateExecutor>(
  provider_info: &ProviderInfo,
  filter: &FilterInfo,
  pact: Pact,
  options: &VerificationOptions<F>,
  provider_state_executor: &S
) -> Vec<(String, MismatchResult)> {
    let mut errors: Vec<(String, MismatchResult)> = vec![];

    let results: Vec<(Interaction, Result<(), MismatchResult>)> = futures::stream::iter(
      pact.interactions.clone().into_iter()
    )
      .filter(|interaction| futures::future::ready(filter_interaction(interaction, filter)))
      .then( |interaction| {
        futures::future::ready((interaction.clone(), verify_interaction(provider_info, interaction, options, provider_state_executor)))
      })
      .collect()
      .await;

    for (interaction, match_result) in results {
      let mut description = format!("Verifying a pact between {} and {}",
                                    pact.consumer.name.clone(), pact.provider.name.clone());
      if let Some((first, elements)) = interaction.provider_states.split_first() {
        description.push_str(&format!(" Given {}", first.name));
        for state in elements {
          description.push_str(&format!(" And {}", state.name));
        }
      }
      description.push_str(" - ");
      description.push_str(&interaction.description);
      println!("  {}", interaction.description);
      match match_result {
        Ok(()) => {
          display_result(
            interaction.response.status,
            Green.paint("OK"),
            interaction.response.headers.map(|h| h.iter().map(|(k, v)| {
              (k.clone(), v.join(", "), Green.paint("OK"))
            }).collect()), Green.paint("OK")
          )
        },
        Err(ref err) => match *err {
          MismatchResult::Error(ref err_des, _) => {
            println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
            errors.push((description, err.clone()));
          },
          MismatchResult::Mismatches { ref mismatches, .. } => {
            description.push_str(" returns a response which ");
            let status_result = if mismatches.iter().any(|m| m.mismatch_type() == "StatusMismatch") {
              Red.paint("FAILED")
            } else {
              Green.paint("OK")
            };
            let header_results = match interaction.response.headers {
              Some(ref h) => Some(h.iter().map(|(k, v)| {
                (k.clone(), v.join(", "), if mismatches.iter().any(|m| {
                  match *m {
                    Mismatch::HeaderMismatch { ref key, .. } => k == key,
                    _ => false
                  }
                }) {
                  Red.paint("FAILED")
                } else {
                  Green.paint("OK")
                })
              }).collect()),
              None => None
            };
            let body_result = if mismatches.iter().any(|m| m.mismatch_type() == "BodyMismatch" ||
              m.mismatch_type() == "BodyTypeMismatch") {
              Red.paint("FAILED")
            } else {
              Green.paint("OK")
            };

            display_result(interaction.response.status, status_result, header_results, body_result);
            errors.push((description.clone(), err.clone()));
          }
        }
      }
    }

    println!();

    errors
}

async fn publish_result<F: RequestFilterExecutor>(
  errors: &[(String, MismatchResult)],
  source: &PactSource,
  options: &VerificationOptions<F>
) {
  if let PactSource::BrokerUrl(_, broker_url, auth, links) = source.clone() {
    log::info!("Publishing verification results back to the Pact Broker");
    let result = if errors.is_empty() {
      log::debug!("Publishing a successful result to {}", source);
      TestResult::Ok
    } else {
      log::debug!("Publishing a failure result to {}", source);
      TestResult::Failed(Vec::from(errors))
    };
    let provider_version = options.provider_version.clone().unwrap();
    let publish_result = publish_verification_results(
      links,
      broker_url.clone(),
      auth.clone(),
      result,
      provider_version,
      options.build_url.clone(),
      options.provider_tags.clone()
    ).await;

    match publish_result {
      Ok(_) => log::info!("Results published to Pact Broker"),
      Err(ref err) => log::error!("Publishing of verification results failed with an error: {}", err)
    };
  }
}

#[cfg(test)]
mod tests;
