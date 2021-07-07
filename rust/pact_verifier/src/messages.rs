use std::collections::HashMap;

use ansi_term::{ANSIGenericString, Style};
use ansi_term::Colour::*;
use bytes::Bytes;
use maplit::*;
use serde_json::{json, Value};

use pact_matching::{match_message, Mismatch};
use pact_matching::models::Interaction;
use pact_matching::models::message::Message;
use pact_models::bodies::OptionalBody;
use pact_models::http_parts::HttpPart;
use pact_models::request::Request;
use pact_models::response::Response;

use crate::{MismatchResult, ProviderInfo, VerificationOptions};
use crate::callback_executors::RequestFilterExecutor;
use crate::provider_client::{make_provider_request, provider_client_error_to_string};

pub async fn verify_message_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &Box<dyn Interaction + Send>,
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
      let metadata = extract_metadata(actual_response);
      let actual = Message {
        contents: actual_response.body.clone(),
        metadata: metadata,
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
  interaction: &Message,
  match_result: &Result<Option<String>, MismatchResult>) {
  match match_result {
    Ok(_) => {
      display_result(Green.paint("OK"),
        interaction.metadata.iter()
          .map(|(k, v)| (k.clone(), serde_json::to_string(&v.clone()).unwrap_or_default(), Green.paint("OK"))).collect());
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
      },
      MismatchResult::Mismatches { ref mismatches, .. } => {
        let metadata_results = interaction.metadata.iter().map(|(k, v)| {
          (k.clone(), serde_json::to_string(&v.clone()).unwrap_or_default(), if mismatches.iter().any(|m| {
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
      }
    }
  }
}

fn display_result(body_result: ANSIGenericString<str>, metadata_result: Vec<(String, String, ANSIGenericString<str>)>) {
  println!("    generates a message which");
  if !metadata_result.is_empty() {
    println!("      includes metadata");
    for (key, value, result) in metadata_result {
      println!("        \"{}\" with value {} ({})", Style::new().bold().paint(key),
        Style::new().bold().paint(value), result);
    }
  }
  println!("      has a matching body ({})", body_result);
}

fn extract_metadata(actual_response: &Response) -> HashMap<String, Value> {
  let content_type = "contentType".to_string();

  let mut default = hashmap!{
    content_type => Value::String(actual_response.lookup_content_type().unwrap_or_default()),
  };

  actual_response.headers.clone().unwrap_or_default().iter().for_each(|(k,v)| {
    if k == "pact_message_metadata" {
      let json: String = v.first().unwrap_or(&"".to_string()).to_string();
      log::trace!("found raw metadata from headers: {:?}", json);

      let decoded = base64::decode(&json.as_str()).unwrap_or_default();
      log::trace!("have base64 decoded headers: {:?}", decoded);

      let metadata: HashMap<String, Value> = serde_json::from_slice(&decoded).unwrap_or_default();
      log::trace!("have JSON metadata from headers: {:?}", metadata);

      for (k, v) in metadata {
        default.insert(k, v);
      }
    }
  });

  default
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use pact_models::generators::Generators;
  use pact_models::matchingrules::MatchingRules;

  use super::*;

  #[test]
    fn extract_metadata_default() {
      let response = Response {
        status: 200,
        headers: Some(hashmap! {
          "content-type".into() => vec!["application/json".into()],
        }),
        body: OptionalBody::default(),
        generators: Generators{
          categories: hashmap!()
        },
        matching_rules: MatchingRules {
          rules: hashmap!()
        }
      };
      let expected = hashmap! {
        "contentType".to_string() => Value::String("application/json".to_string())
      };

      expect(extract_metadata(&response)).to(be_eq(expected));
    }

    #[test]
    fn extract_metadata_from_base64_header() {
      let response = Response {
        status: 200,
        headers: Some(hashmap! {
          "content-type".into() => vec!["application/json".into()],
          // must convert lowercase here, because the http framework actually lowercases this for us
          "PACT_MESSAGE_METADATA".to_lowercase().into() => vec!["ewogICJDb250ZW50LVR5cGUiOiAiYXBwbGljYXRpb24vanNvbiIsCiAgInRvcGljIjogImJheiIsCiAgIm51bWJlciI6IDI3LAogICJjb21wbGV4IjogewogICAgImZvbyI6ICJiYXIiCiAgfQp9Cg==".into()],
        }),
        body: OptionalBody::default(),
        generators: Generators{
          categories: hashmap!()
        },
        matching_rules: MatchingRules {
          rules: hashmap!()
        }
      };
      let expected = hashmap! {
        "contentType".to_string() => Value::String("application/json".to_string()), // From actual HTTP response header
        "Content-Type".to_string() => Value::String("application/json".to_string()), // From metadata header
        "complex".to_string() => json!({"foo": "bar"}),
        "topic".to_string() => Value::String("baz".into()),
        "number".to_string() => json!(27),
      };

      expect(extract_metadata(&response)).to(be_eq(expected));
    }
}
