use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use cucumber::{given, then, when, World};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_models::{Consumer, generators, matchingrules, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::generators::Generator;
use pact_models::matchingrules::MatchingRule;
use pact_models::pact::{Pact, read_pact};
use pact_models::provider_states::ProviderState;
use pact_models::request::Request;
use pact_models::response::Response;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;
use reqwest::Client;
use serde_json::{json, Value};
use uuid::Uuid;

use pact_matching::Mismatch;
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::{MockServer, MockServerConfig};
use pact_verifier::{
  FilterInfo,
  NullRequestFilterExecutor,
  PactSource,
  ProviderInfo,
  ProviderTransport,
  PublishOptions,
  VerificationOptions,
  verify_provider_async
};
use pact_verifier::callback_executors::ProviderStateExecutor;
use pact_verifier::verification_result::{VerificationExecutionResult, VerificationMismatchResult};

use crate::v1_steps::common::setup_common_interactions;

#[derive(Debug, World)]
pub struct ProviderWorld {
  pub interactions: Vec<RequestResponseInteraction>,
  pub provider_key: String,
  pub provider_server: Arc<Mutex<MockServer>>,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub publish_options: Option<PublishOptions>,
  pub verification_results: VerificationExecutionResult,
  pub mock_brokers: Vec<Arc<Mutex<MockServer>>>,
  pub provider_state_executor: Arc<MockProviderStateExecutor>
}

impl Default for ProviderWorld {
  fn default() -> Self {
    ProviderWorld {
      interactions: vec![],
      provider_key: "".to_string(),
      provider_server: Default::default(),
      provider_info: ProviderInfo::default(),
      sources: vec![],
      publish_options: None,
      verification_results: VerificationExecutionResult {
        result: false,
        .. VerificationExecutionResult::new()
      },
      mock_brokers: vec![],
      provider_state_executor: Default::default()
    }
  }
}

#[derive(Debug, Default)]
pub struct MockProviderStateExecutor {
  pub params: Arc<Mutex<Vec<(ProviderState, bool)>>>,
  pub fail_mode: AtomicBool
}

impl MockProviderStateExecutor {
  pub fn set_fail_mode(&self, mode: bool) {
    self.fail_mode.store(mode, Ordering::Relaxed);
  }

  pub fn was_called(&self, is_setup: bool) -> bool {
    let params = self.params.lock().unwrap();
    params.iter().find(|(_, setup)| *setup == is_setup).is_some()
  }

  pub fn was_called_for_state(&self, state_name: &str, is_setup: bool) -> bool {
    let params = self.params.lock().unwrap();
    params.iter().find(|(state, setup)| {
      state.name == state_name && *setup == is_setup
    }).is_some()
  }
}

#[async_trait]
impl ProviderStateExecutor for MockProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    _client: Option<&Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    let mut lock = self.params.try_lock();
    if let Ok(ref mut params) = lock {
      params.push((provider_state.clone(), setup));
    }

    if self.fail_mode.load(Ordering::Relaxed) {
      Err(anyhow!("ProviderStateExecutor is in fail mode"))
    } else {
      Ok(hashmap! {})
    }
  }

  fn teardown(self: &Self) -> bool {
    return true
  }
}

#[given("the following HTTP interactions have been defined:")]
fn the_following_http_interactions_have_been_setup(world: &mut ProviderWorld, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let interactions = setup_common_interactions(table);
    world.interactions.extend(interactions);
  }
}

#[given(expr = "a provider is started that returns the response from interaction \\{{int}}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_response_from_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V1,
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    world.provider_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  world.provider_server = mock_server;

  let ms = world.provider_server.lock().unwrap();
  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "[::1]".to_string(),
    port: ms.port,
    transports: vec![ProviderTransport {
      port: ms.port,
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };

  Ok(())
}

#[given(expr = "a Pact file for interaction \\{{int}} is to be verified")]
fn a_pact_file_for_interaction_is_to_be_verified(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V1)?.to_string()));
  Ok(())
}

#[given(expr = "a Pact file for interaction \\{{int}} is to be verified with a provider state {string} defined")]
fn a_pact_file_for_interaction_is_to_be_verified_with_a_provider_state(
  world: &mut ProviderWorld,
  num: usize,
  state: String
) -> anyhow::Result<()> {
  let mut interaction = world.interactions.get(num - 1).unwrap().clone();
  interaction.provider_states.push(ProviderState {
    name: state,
    params: Default::default(),
  });
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V1)?.to_string()));
  Ok(())
}

#[when("the verification is run")]
async fn the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
  world.verification_results = verify_provider_async(
    world.provider_info.clone(),
    world.sources.clone(),
    FilterInfo::None,
    vec![],
    &options,
    world.publish_options.as_ref(),
    &world.provider_state_executor,
    None
  ).await?;
  Ok(())
}

#[then("the verification will be successful")]
fn the_verification_will_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Ok(())
  } else {
    Err(anyhow!("Verification failed"))
  }
}

#[given(expr = "a provider is started that returns the responses from interactions {string}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_responses_from_interactions(
  world: &mut ProviderWorld,
  ids: String
) -> anyhow::Result<()> {
  let interactions = ids.split(",")
    .map(|id| id.trim().parse::<usize>().unwrap())
    .map(|index| world.interactions.get(index - 1).unwrap().clone())
    .collect();
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions,
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V1,
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    world.provider_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  world.provider_server = mock_server;

  let ms = world.provider_server.lock().unwrap();
  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "[::1]".to_string(),
    port: ms.port,
    transports: vec![ProviderTransport {
      port: ms.port,
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };
  Ok(())
}

#[then("the verification will NOT be successful")]
fn the_verification_will_not_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Err(anyhow!("Was expecting the verification to fail"))
  } else {
    Ok(())
  }
}

#[then(expr = "the verification results will contain a {string} error")]
fn the_verification_results_will_contain_a_error(world: &mut ProviderWorld, err: String) -> anyhow::Result<()> {
  if world.verification_results.errors.iter().any(|(_, r)| {
    match dbg!(r) {
      VerificationMismatchResult::Mismatches { mismatches, .. } => {
        mismatches.iter().any(|mismatch| {
          match mismatch {
            Mismatch::MethodMismatch { .. } => false,
            Mismatch::PathMismatch { .. } => false,
            Mismatch::StatusMismatch { .. } => err == "Response status did not match",
            Mismatch::QueryMismatch { .. } => false,
            Mismatch::HeaderMismatch { .. } => false,
            Mismatch::BodyTypeMismatch { .. } => false,
            Mismatch::BodyMismatch { .. } => false,
            Mismatch::MetadataMismatch { .. } => false
          }
        })
      }
      VerificationMismatchResult::Error { error, .. } => match err.as_str() {
        "State change request failed" => error == "One or more of the setup state change handlers has failed",
        _ => error.as_str() == err
      }
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find error message in verification results"))
  }
}

#[given(expr = "a Pact file for interaction \\{{int}} is to be verified from a Pact broker")]
async fn a_pact_file_for_interaction_is_to_be_verified_from_a_pact_broker(
  world: &mut ProviderWorld,
  num: usize
) -> anyhow::Result<()> {
  let interaction = world.interactions.get(num - 1).unwrap().clone();
  let pact = RequestResponsePact {
    consumer: Consumer { name: format!("c_{}", num) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![interaction.clone()],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  let mut pact_json = pact.to_json(PactSpecification::V1)?;
  let pact_json_inner = pact_json.as_object_mut().unwrap();
  pact_json_inner.insert("_links".to_string(), json!({
    "pb:publish-verification-results": {
      "title": "Publish verification results",
      "href": format!("http://localhost:1234/pacts/provider/p/consumer/c_{}/verification-results", num)
    }
  }));
  let interactions_json = pact_json_inner.get_mut("interactions").unwrap().as_array_mut().unwrap();
  let interaction_json = interactions_json.get_mut(0).unwrap().as_object_mut().unwrap();
  interaction_json.insert("_id".to_string(), json!(interaction.id.unwrap()));

  let f = PathBuf::from(format!("pact-compatibility-suite/fixtures/pact-broker_c{}.json", num));
  let mut broker_pact = read_pact(&*f)
    .expect(format!("could not load fixture 'pact-broker_c{}.json'", num).as_str())
    .as_request_response_pact().unwrap();

  // AAARGH! My head. Adding a Pact Interaction to a Pact file for fetching a Pact file for verification
  let matching_rules = matchingrules! {
    "body" => { "$._links.pb:publish-verification-results.href" => [
      MatchingRule::Regex(format!(".*(\\/pacts\\/provider\\/p\\/consumer\\/c_{}\\/verification-results)", num))
    ] }
  };
  let generators = generators! {
    "BODY" => {
      "$._links.pb:publish-verification-results.href" => Generator::MockServerURL(
        format!("http://localhost:1234/pacts/provider/p/consumer/c_{}/verification-results", num),
        format!(".*(\\/pacts\\/provider\\/p\\/consumer\\/c_{}\\/verification-results)", num)
      )
    }
  };
  let interaction = RequestResponseInteraction {
    request: Request {
      path: format!("/pacts/provider/p/consumer/c_{}", num),
      .. Request::default()
    },
    response: Response {
      headers: Some(hashmap!{
        "content-type".to_string() => vec![ "application/json".to_string() ]
      }),
      body: OptionalBody::Present(Bytes::from(pact_json.to_string()),
                                  Some(JSON.clone()), None),
      matching_rules,
      generators,
      .. Response::default()
    },
    .. RequestResponseInteraction::default()
  };
  broker_pact.interactions.push(interaction);

  let config = MockServerConfig {
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    Uuid::new_v4().to_string(), broker_pact.boxed(), "127.0.0.1:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  let broker_port = {
    let ms = mock_server.lock().unwrap();
    ms.port
  };
  world.mock_brokers.push(mock_server);

  world.sources.push(PactSource::BrokerWithDynamicConfiguration {
    provider_name: "p".to_string(),
    broker_url: format!("http://localhost:{}", broker_port.unwrap()),
    enable_pending: false,
    include_wip_pacts_since: None,
    provider_tags: vec![],
    provider_branch: None,
    selectors: vec![],
    auth: None,
    links: vec![],
  });
  Ok(())
}

#[then("a verification result will NOT be published back")]
fn a_verification_result_will_not_be_published_back(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let ms = broker.lock().unwrap();
    let verification_requests = ms.metrics.requests_by_path.iter()
      .find(|(path, _)| {
        path.ends_with("/verification-results")
      })
      .map(|(_, count)| *count)
      .unwrap_or(0);
    verification_requests > 0
  });
  if verification_results {
    Err(anyhow!("Was expecting no verification results"))
  } else {
    Ok(())
  }
}

#[given("publishing of verification results is enabled")]
fn publishing_of_verification_results_is_enabled(world: &mut ProviderWorld) {
  world.publish_options = Some(PublishOptions {
    provider_version: Some("1.2.3".to_string()),
    build_url: None,
    provider_tags: vec![],
    provider_branch: None,
  });
}

#[then(expr = "a successful verification result will be published back for interaction \\{{int}}")]
fn a_successful_verification_result_will_be_published_back_for_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()>  {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let ms = broker.lock().unwrap();
    let vec = ms.matches();
    let verification_request = vec.iter()
      .find(|result| {
        let expected_path = format!("/pacts/provider/p/consumer/c_{}/verification-results", num);
        match result {
          MatchResult::RequestMatch(req, _) => req.path == expected_path,
          MatchResult::RequestMismatch(req, _) => req.path == expected_path,
          MatchResult::RequestNotFound(req) => req.path == expected_path,
          MatchResult::MissingRequest(req) => req.path == expected_path
        }
      });
    if let Some(result) = verification_request {
      match result {
        MatchResult::RequestMatch(req, _) => if let Some(body) = req.body.value() {
          if let Ok(json) = serde_json::from_slice::<Value>(body.as_ref()) {
            if let Some(success) = json.get("success") {
              match success {
                Value::Bool(b) => *b,
                _ => false
              }
            } else {
              false
            }
          } else {
            false
          }
        } else {
          false
        },
        _ => false
      }
    } else {
      false
    }
  });
  if verification_results {
    Ok(())
  } else {
    Err(anyhow!("Either no verification results was published, or it was incorrect"))
  }
}

#[then(expr = "a failed verification result will be published back for the interaction \\{{int}}")]
fn a_failed_verification_result_will_be_published_back_for_the_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()>  {
  let verification_results = world.mock_brokers.iter().any(|broker| {
    let ms = broker.lock().unwrap();
    let vec = ms.matches();
    let verification_request = vec.iter()
      .find(|result| {
        let expected_path = format!("/pacts/provider/p/consumer/c_{}/verification-results", num);
        match result {
          MatchResult::RequestMatch(req, _) => req.path == expected_path,
          MatchResult::RequestMismatch(req, _) => req.path == expected_path,
          MatchResult::RequestNotFound(req) => req.path == expected_path,
          MatchResult::MissingRequest(req) => req.path == expected_path
        }
      });
    if let Some(result) = verification_request {
      match result {
        MatchResult::RequestMatch(req, _) => if let Some(body) = req.body.value() {
          if let Ok(json) = serde_json::from_slice::<Value>(body.as_ref()) {
            if let Some(success) = json.get("success") {
              match success {
                Value::Bool(b) => !*b,
                _ => false
              }
            } else {
              false
            }
          } else {
            false
          }
        } else {
          false
        },
        _ => false
      }
    } else {
      false
    }
  });
  if verification_results {
    Ok(())
  } else {
    Err(anyhow!("Either no verification results was published, or it was incorrect"))
  }
}

#[given("a provider state callback is configured")]
fn a_provider_state_callback_is_configured(world: &mut ProviderWorld) -> anyhow::Result<()> {
  world.provider_state_executor.set_fail_mode(false);
  Ok(())
}

#[given("a provider state callback is configured, but will return a failure")]
fn a_provider_state_callback_is_configured_but_will_return_a_failure(world: &mut ProviderWorld) -> anyhow::Result<()> {
  world.provider_state_executor.set_fail_mode(true);
  Ok(())
}

#[then("the provider state callback will be called before the verification is run")]
fn the_provider_state_callback_will_be_called_before_the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called"))
  }
}

#[then(expr = "the provider state callback will receive a setup call with {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_setup_call_with_as_the_provider_state_parameter(
  world: &mut ProviderWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), true) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback was not called for state '{}'", state))
  }
}

#[then("the provider state callback will be called after the verification is run")]
fn the_provider_state_callback_will_be_called_after_the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state callback teardown was not called"))
  }
}

#[then(expr = "the provider state callback will receive a teardown call {string} as the provider state parameter")]
fn the_provider_state_callback_will_receive_a_teardown_call_as_the_provider_state_parameter(
  world: &mut ProviderWorld,
  state: String
) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called_for_state(state.as_str(), false) {
    Ok(())
  } else {
    Err(anyhow!("Provider state teardown callback was not called for state '{}'", state))
  }
}

#[then("the provider state callback will NOT receive a teardown call")]
fn the_provider_state_callback_will_not_receive_a_teardown_call(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.provider_state_executor.was_called(false) {
    Err(anyhow!("Provider state callback teardown was called but was expecting no call"))
  } else {
    Ok(())
  }
}
