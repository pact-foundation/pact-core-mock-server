use std::collections::HashMap;

use ansi_term::{ANSIGenericString, Style};
use ansi_term::Colour::*;
use bytes::Bytes;
use log::{debug, warn};
use maplit::*;
use serde_json::{json, Value};

use pact_matching::{match_message, match_sync_message_response, Mismatch};
use pact_models::bodies::OptionalBody;
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::message::Message;
use pact_models::prelude::Pact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::sync_message::SynchronousMessage;

use crate::{MismatchResult, ProviderInfo, VerificationOptions};
use crate::callback_executors::RequestFilterExecutor;
use crate::provider_client::make_provider_request;

pub(crate) async fn verify_message_from_provider<'a, F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  pact: &Box<dyn Pact + Send + Sync + 'a>,
  interaction: &Box<dyn Interaction + Send + Sync>,
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

  let message_request = HttpRequest {
    method: "POST".into(),
    body: OptionalBody::Present(Bytes::from(request_body.to_string()), Some("application/json".into()), None),
    headers: Some(hashmap! {
        "Content-Type".to_string() => vec!["application/json".to_string()]
    }),
    .. HttpRequest::default()
  };

  match make_provider_request(provider, &message_request, options, client).await {
    Ok(ref actual_response) => {
      let metadata = extract_metadata(actual_response);
      let actual = AsynchronousMessage {
        contents: MessageContents {
          metadata,
          contents: actual_response.body.clone(),
          .. MessageContents::default()
        },
        .. AsynchronousMessage::default()
      };

      debug!("actual message = {:?}", actual);

      let mismatches = match_message(interaction, &actual.boxed(), pact).await;
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
      Err(MismatchResult::Error(err.to_string(), interaction.id().clone()))
    }
  }
}

pub fn display_message_result(
  interaction: &Message,
  match_result: &Result<Option<String>, MismatchResult>,
  output: &mut Vec<String>) {
  match match_result {
    Ok(_) => {
      display_result(Green.paint("OK"),
        interaction.metadata.iter()
          .map(|(k, v)| (k.clone(), serde_json::to_string(&v.clone()).unwrap_or_default(), Green.paint("OK"))).collect(),
        output
      );
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        output.push(format!("      {}", Red.paint(format!("Request Failed - {}", err_des))));
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

        display_result(body_result, metadata_results, output);
      }
    }
  }
}

fn display_result(
  body_result: ANSIGenericString<str>,
  metadata_result: Vec<(String, String, ANSIGenericString<str>)>,
  output: &mut Vec<String>
) {
  output.push("    generates a message which".to_string());
  if !metadata_result.is_empty() {
    output.push("      includes metadata".to_string());
    for (key, value, result) in metadata_result {
      output.push(format!("        \"{}\" with value {} ({})", Style::new().bold().paint(key),
        Style::new().bold().paint(value), result));
    }
  }
  output.push(format!("      has a matching body ({})", body_result));
}

fn extract_metadata(actual_response: &HttpResponse) -> HashMap<String, Value> {
  let content_type = "contentType".to_string();

  let mut default = hashmap!{
    content_type => Value::String(actual_response.lookup_content_type().unwrap_or_default()),
  };

  actual_response.headers.clone().unwrap_or_default().iter().for_each(|(k,v)| {
    if k.to_lowercase() == "pact-message-metadata" {
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

pub(crate) async fn verify_sync_message_from_provider<'a, F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  pact: &Box<dyn Pact + Send + Sync + 'a>,
  message: SynchronousMessage,
  options: &VerificationOptions<F>,
  client: &reqwest::Client,
  _: &HashMap<&str, Value>
) -> Result<Option<String>, MismatchResult> {
  if message.response.len() > 1 {
    warn!("Matching synchronous messages with more than one response is not currently supported, will only use the first response");
  }

  let mut request_body = json!({
    "description": message.description(),
    "request": message.request.to_json()
  });

  if !message.provider_states().is_empty() {
    if let Some(map) = request_body.as_object_mut() {
      map.insert("providerStates".into(), Value::Array(message.provider_states().iter()
        .map(|ps| ps.to_json()).collect()));
    }
  }

  let message_request = HttpRequest {
    method: "POST".into(),
    body: OptionalBody::Present(Bytes::from(request_body.to_string()), Some("application/json".into()), None),
    headers: Some(hashmap! {
        "Content-Type".to_string() => vec!["application/json".to_string()]
    }),
    .. HttpRequest::default()
  };

  match make_provider_request(provider, &message_request, options, client).await {
    Ok(ref actual_response) => {
      if actual_response.is_success() {
        let metadata = extract_metadata(actual_response);
        let actual_contents = MessageContents {
          metadata,
          contents: actual_response.body.clone(),
          ..MessageContents::default()
        };
        let actual = SynchronousMessage {
          response: vec![actual_contents],
          .. SynchronousMessage::default()
        };

        debug!("actual message = {:?}", actual);

        let mismatches = match_sync_message_response(&message, &message.response, &actual.response, pact).await;
        if mismatches.is_empty() {
          Ok(message.id().clone())
        } else {
          Err(MismatchResult::Mismatches {
            mismatches,
            expected: message.boxed(),
            actual: actual.boxed(),
            interaction_id: message.id().clone()
          })
        }
      } else {
        Err(MismatchResult::Error(format!("Request to fetch message from provider failed: status {}", actual_response.status), message.id().clone()))
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(err.to_string(), message.id().clone()))
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use pact_models::generators::Generators;
  use pact_models::matchingrules::MatchingRules;

  use super::*;

  #[test]
    fn extract_metadata_default() {
      let response = HttpResponse {
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
      let response = HttpResponse {
        status: 200,
        headers: Some(hashmap! {
          "content-type".into() => vec!["application/json".into()],
          // must convert lowercase here, because the http framework actually lowercases this for us
          "Pact-Message-Metadata".to_lowercase().into() => vec!["ewogICJDb250ZW50LVR5cGUiOiAiYXBwbGljYXRpb24vanNvbiIsCiAgInRvcGljIjogImJheiIsCiAgIm51bWJlciI6IDI3LAogICJjb21wbGV4IjogewogICAgImZvbyI6ICJiYXIiCiAgfQp9Cg==".into()],
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
