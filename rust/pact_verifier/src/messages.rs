use std::collections::HashMap;

use ansi_term::{ANSIGenericString, Style};
use ansi_term::Colour::*;
use bytes::Bytes;
use maplit::*;
use serde_json::{json, Value};

use pact_matching::{match_message, Mismatch};
use pact_matching::models::{Interaction, Request};
use pact_matching::models::HttpPart;
use pact_matching::models::message::Message;
use pact_models::OptionalBody;

use crate::{MismatchResult, ProviderInfo, VerificationOptions};
use crate::callback_executors::RequestFilterExecutor;
use crate::provider_client::{make_provider_request, provider_client_error_to_string};

pub async fn verify_message_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &Box<dyn Interaction>,
  options: &VerificationOptions<F>,
  client: &reqwest::Client,
  _: &HashMap<&str, Value>
) -> Result<Option<String>, MismatchResult> {
  let mut request_body = json!({
    "description": interaction.description()
  });
  if !interaction.provider_states().is_empty() {
    if let Some(map) = request_body.as_object_mut() {
      map.insert("providerStates".into(), Value::Array(interaction.provider_states().iter()
        .map(|ps| ps.to_json()).collect()));
    }
  }
  let message_request = Request {
    method: "POST".into(),
    body: OptionalBody::Present(Bytes::from(request_body.to_string()), Some("application/json".into())),
    headers: Some(hashmap! {
        "Content-Type".to_string() => vec!["application/json".to_string()]
    }),
    .. Request::default()
  };
  match make_provider_request(provider, &message_request, options, client).await {
    Ok(ref actual_response) => {
      let actual = Message {
        contents: actual_response.body.clone(),
        metadata: hashmap!{
          "contentType".into() => actual_response.lookup_content_type().unwrap_or_default()
        },
        .. Message::default()
      };
      log::debug!("actual message = {:?}", actual);
      let mismatches = match_message(interaction, &actual.boxed());
      if mismatches.is_empty() {
        Ok(interaction.id().clone())
      } else {
        Err(MismatchResult::Mismatches {
          mismatches,
          expected: interaction.boxed(),
          actual: actual.boxed(),
          interaction_id: interaction.id().clone()
        })
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(provider_client_error_to_string(err), interaction.id().clone()))
    }
  }
}

pub fn display_message_result(
  errors: &mut Vec<(Option<String>, String, Option<MismatchResult>)>,
  interaction: &Message,
  match_result: &Result<Option<String>, MismatchResult>,
  description: &String
) {
  match match_result {
    Ok(id) => {
      display_result(Green.paint("OK"),
        interaction.metadata.iter()
          .map(|(k, v)| (k.clone(), v.clone(), Green.paint("OK"))).collect()
      );
      errors.push((id.clone(), description.clone(), None));
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
        errors.push((err.interaction_id().clone(), description.clone(), Some(err.clone())));
      },
      MismatchResult::Mismatches { ref mismatches, .. } => {
        let metadata_results = interaction.metadata.iter().map(|(k, v)| {
          (k.clone(), v.clone(), if mismatches.iter().any(|m| {
            match *m {
              Mismatch::MetadataMismatch { ref key, .. } => k == key,
              _ => false
            }
          }) {
            Red.paint("FAILED")
          } else {
            Green.paint("OK")
          })
        }).collect();
        let body_result = if mismatches.iter().any(|m| m.mismatch_type() == "BodyMismatch" ||
          m.mismatch_type() == "BodyTypeMismatch") {
          Red.paint("FAILED")
        } else {
          Green.paint("OK")
        };

        display_result(body_result, metadata_results);
        errors.push((interaction.id.clone(), description.clone(), Some(err.clone())));
      }
    }
  }
}

fn display_result(body_result: ANSIGenericString<str>, metadata_result: Vec<(String, String, ANSIGenericString<str>)>) {
  println!("    generates a message which");
  if !metadata_result.is_empty() {
    println!("      includes metadata");
    for (key, value, result) in metadata_result {
      println!("        \"{}\" with value \"{}\" ({})", Style::new().bold().paint(key),
        Style::new().bold().paint(value), result);
    }
  }
  println!("      has a matching body ({})", body_result);
}
