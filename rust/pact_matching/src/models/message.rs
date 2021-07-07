//! The `message` module provides all functionality to deal with messages.

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use base64::encode;
use maplit::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use pact_models::json_utils::{json_to_string, body_from_json};
use pact_models::{generators, PactSpecification};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::Generators;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRules;
use pact_models::provider_states::ProviderState;
use pact_models::http_parts::HttpPart;

use crate::models::{Interaction, RequestResponseInteraction};
use crate::models::v4::{AsynchronousMessage, SynchronousHttp, V4Interaction};
use crate::models::v4::message_parts::MessageContents;
use crate::models::v4::sync_message::SynchronousMessages;

/// Struct that defines a message.
#[derive(PartialEq, Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Message {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    pub id: Option<String>,

    /// Description of this message interaction. This needs to be unique in the pact file.
    pub description: String,

    /// Optional provider state for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    #[serde(rename = "providerStates")]
    #[serde(default)]
    pub provider_states: Vec<ProviderState>,

    /// The contents of the message
    #[serde(default = "missing_body")]
    pub contents: OptionalBody,

    /// Metadata associated with this message.
    #[serde(default)]
    pub metadata: HashMap<String, Value>,

    /// Matching rules
    #[serde(rename = "matchingRules")]
    #[serde(default)]
    pub matching_rules: matchingrules::MatchingRules,

    /// Generators
    #[serde(default)]
    pub generators: generators::Generators
}

impl Interaction for Message {
  fn type_of(&self) -> String {
    "V3 Asynchronous/Messages".into()
  }

  fn is_request_response(&self) -> bool {
    false
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    None
  }

  fn is_message(&self) -> bool {
    true
  }

  fn as_message(&self) -> Option<Message> {
    Some(self.clone())
  }

  fn id(&self) -> Option<String> {
    self.id.clone()
  }

  fn description(&self) -> String {
    self.description.clone()
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    self.provider_states.clone()
  }

  fn contents(&self) -> OptionalBody {
    self.contents.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.message_content_type()
  }

  fn is_v4(&self) -> bool {
    false
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction>> {
    self.as_v4_async_message().map(|i| i.boxed_v4())
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    None
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    Some(AsynchronousMessage {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: MessageContents {
        contents: self.contents.clone(),
        metadata: self.metadata.iter()
          .map(|(k, v)| (k.clone(), v.clone()))
          .collect(),
        matching_rules: self.matching_rules.rename("body", "content"),
        generators: self.generators.clone()
      },
      .. Default::default()
    })
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessages> {
    None
  }

  fn boxed(&self) -> Box<dyn Interaction + Send> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Interaction + Send> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn matching_rules(&self) -> Option<MatchingRules> {
    Some(self.matching_rules.clone())
  }
}

impl Message {
    /// Returns a default message
    pub fn default() -> Message {
      Message {
        id: None,
        description: s!("message"),
        provider_states: vec![],
        contents: OptionalBody::Missing,
        metadata: hashmap!{
          "contentType".into() => "application/json".into()
        },
        matching_rules: matchingrules::MatchingRules::default(),
        generators: Generators::default()
      }
    }

    /// Constructs a `Message` from the `Json` struct.
    pub fn from_json(index: usize, json: &Value, spec_version: &PactSpecification) -> anyhow::Result<Message> {
        match spec_version {
            &PactSpecification::V3 => {
                let description = match json.get("description") {
                    Some(v) => match *v {
                        Value::String(ref s) => s.clone(),
                        _ => v.to_string()
                    },
                    None => format!("Message {}", index)
                };
                let provider_states = ProviderState::from_json(json);
                let metadata = match json.get("metaData").or(json.get("metadata")) {
                  Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                      (k.clone(), v.clone())
                  }).collect(),
                  _ => hashmap!{},
                };
                Ok(Message {
                  id: None,
                  description,
                  provider_states,
                  contents: body_from_json(json, "contents", &None),
                  matching_rules: matchingrules::matchers_from_json(json, &None),
                  metadata,
                  generators: Generators::default()
                })
            },
            _ => Err(anyhow!("Messages require Pact Specification version 3"))
        }
    }

    /// Converts this interaction to a `Value` struct.
    /// note: spec version is preserved for compatibility with the RequestResponsePact interface
    /// and for future use
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
      let mut value = json!({
          s!("description"): Value::String(self.description.clone()),
          s!("metadata"): self.metadata
      });
      {
        let map = value.as_object_mut().unwrap();

        if self.matching_rules.is_not_empty() {
            map.insert(s!("matchingRules"), matchingrules::matchers_to_json(
            &self.matching_rules.clone(), spec_version));
        }
        if self.generators.is_not_empty() {
          map.insert(s!("generators"), generators::generators_to_json(
            &self.generators.clone(), spec_version));
        }

        match self.contents {
          OptionalBody::Present(ref body, _) => {
            let content_type = self.message_content_type().unwrap_or_default();
            if content_type.is_json() {
            match serde_json::from_slice(body) {
                Ok(json_body) => { map.insert("contents".to_string(), json_body); },
              Err(err) => {
                log::warn!("Failed to parse json body: {}", err);
                map.insert(s!("contents"), Value::String(encode(body)));
              }
            }
            } else if content_type.is_binary() {
              map.insert("contents".to_string(), Value::String(encode(body)));
          } else {
              match from_utf8(body) {
                Ok(s) => map.insert("contents".to_string(), Value::String(s.to_string())),
                Err(_) => map.insert("contents".to_string(), Value::String(encode(body)))
            };
            }
          },
          OptionalBody::Empty => { map.insert("contents".to_string(), Value::String("".to_string())); },
          OptionalBody::Missing => (),
          OptionalBody::Null => { map.insert("contents".to_string(), Value::Null); }
        }
        if !self.provider_states.is_empty() {
          map.insert("providerStates".to_string(), Value::Array(self.provider_states.iter().map(|p| p.to_json()).collect()));
        }
      }

      value
  }

  /// Returns the content type of the message by returning the content type associated with
  /// the body, or by looking it up in the message metadata
  pub fn message_content_type(&self) -> Option<ContentType> {
    let body = &self.contents;
    if body.has_content_type() {
      body.content_type()
    } else {
      match self.metadata.iter().find(|(k, _)| {
        let key = k.to_ascii_lowercase();
        key == "contenttype" || key == "content-type"
      }) {
        Some((_, v)) => ContentType::parse(json_to_string(&v).as_str()).ok(),
        None => self.detect_content_type()
      }
    }
  }
}

impl HttpPart for Message {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
    unimplemented!()
  }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    unimplemented!()
  }

  fn body(&self) -> &OptionalBody {
    &self.contents
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| json_to_string(&v[0]))
  }
}

impl Display for Message {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Message ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           self.id, self.description, self.provider_states, self.contents, self.metadata)
  }
}

fn missing_body() -> OptionalBody {
    OptionalBody::Missing
}

#[cfg(test)]
mod tests {
  use std::{env, fs};
  use std::path::Path;

  use bytes::Bytes;
  use expectest::expect;
  use expectest::prelude::*;
  use serde_json;

  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;

  use super::*;

  #[test]
    fn loading_message_from_json() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {}
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: s!("provider state"),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn defaults_to_number_if_no_description() {
        let message_json = r#"{
            "providerState": "provider state"
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("Message 0"));
    }

    #[test]
    fn defaults_to_none_if_no_provider_state() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_states.iter()).to(be_empty());
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn defaults_to_none_if_provider_state_null() {
        let message_json = r#"{
            "providerState": null
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_states.iter()).to(be_empty());
    }

    #[test]
    fn returns_an_error_if_the_spec_version_is_less_than_three() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state"
        }"#;
        let result = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V1);
        expect!(result).to(be_err());
    }

    #[test]
    fn message_with_json_body() {
        let message_json = r#"{
            "contents": {
                "hello": "world"
            },
            "metadata": {
                "contentType": "application/json"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.str_value()).to(be_equal_to("{\"hello\":\"world\"}"));
    }

    #[test]
    fn message_with_non_json_body() {
        let message_json = r#"{
            "contents": "hello world",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.str_value()).to(be_equal_to("hello world"));
    }

    #[test]
    fn message_with_empty_body() {
        let message_json = r#"{
            "contents": "",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents.str_value()).to(be_equal_to(""));
    }

    #[test]
    fn message_with_missing_body() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to(OptionalBody::Missing));
    }

    #[test]
    fn message_with_null_body() {
        let message_json = r#"{
            "contents": null,
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to(OptionalBody::Null));
    }

    #[test]
    fn message_mimetype_is_based_on_the_metadata() {
      let message = Message {
        metadata: hashmap!{ s!("contentType") => Value::String("text/plain".to_string()) },
        .. Message::default()
      };
      expect!(message.message_content_type().unwrap_or_default().to_string()).to(be_equal_to("text/plain"));
    }

    #[test]
    fn message_mimetype_defaults_to_json() {
      let message = Message::default();
      expect!(message.message_content_type().unwrap_or_default().to_string()).to(be_equal_to("application/json"));
    }

    #[test]
    fn ignoring_v1_provider_state_when_deserializing_message() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {},
            "generators": {}
        }"#;

        // This line should panic, because providerState is not the name of the field.
        let message: Message = serde_json::from_str(message_json).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states.iter()).to(be_empty());
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

    #[test]
    fn loading_message_from_json_by_deserializing() {
        let message_json = r#"{
            "description": "String",
            "providerStates": [{ "name": "provider state", "params": {} }],
            "matchingRules": {},
            "generators": {}
        }"#;

        let message: Message = serde_json::from_str(message_json).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: s!("provider state"),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }

  #[test]
  fn when_upgrading_message_pact_to_v4_rename_the_matching_rules_from_body_to_content() {
    let message = Message {
      contents: OptionalBody::Missing,
      matching_rules: matchingrules! { "body" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] } },
      .. Message::default()
    };
    let v4 = message.as_v4_async_message().unwrap();
    expect!(v4.contents.matching_rules).to(be_equal_to(
      matchingrules! { "content" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] }}
    ));
  }

  #[test]
  fn message_with_json_body_serialises() {
    let message_json = r#"{
        "contents": {
            "hello": "world"
        },
        "metadata": {
            "contentType": "application/json"
        }
    }"#;
    let message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
    let v = message.to_json(&PactSpecification::V3);
    expect!(v.get("contents").unwrap().get("hello").unwrap().as_str().unwrap()).to(be_equal_to("world"));
  }

  #[test]
  fn message_with_binary_body_serialises() {
    let message_json = r#"{
        "metadata": {
            "contentType": "application/octet-stream"
        }
    }"#;

    let file = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("tests/data")
      .join("message_with_binary_body_serialises.zip")
      .to_owned();

    let content_type = ContentType::parse("application/octet-stream").unwrap();
    let contents = fs::read(file).unwrap();
    let encoded = concat!(
      "UEsDBAoAAAAAAI2rtlKd3GsXCgAAAAoAAAAIABwAZmlsZS50eHRVVAkAA9nqqGDb6qhgdXgLAAEE9QEAAAQUAAAAdGVzdCBkYXRhClBL",
      "AQIeAwoAAAAAAI2rtlKd3GsXCgAAAAoAAAAIABgAAAAAAAEAAACkgQAAAABmaWxlLnR4dFVUBQAD2eqoYHV4CwABBPUBAAAEFAAAAFBLBQYAAAAAAQABAE4",
      "AAABMAAAAAAA=");

    let mut message = Message::from_json(0, &serde_json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
    message.contents = OptionalBody::Present(Bytes::from(contents), Some(content_type));
    let json = message.to_json(&PactSpecification::V3);

    expect!(json.get("contents").unwrap().as_str().unwrap()).to(be_equal_to(encoded));
  }
}
