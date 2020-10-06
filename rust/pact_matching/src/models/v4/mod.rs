//! V4 specification models

use std::fmt;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::string::ToString;

use log::*;
use maplit::*;
use nom::lib::std::fmt::Formatter;
use serde_json::Value;

use crate::models::{Consumer, Interaction, Pact, PactSpecification, Provider, RequestResponsePact, OptionalBody, RequestResponseInteraction};
use crate::models::json_utils::json_to_string;
use crate::models::message_pact::MessagePact;
use crate::models::message::Message;
use crate::models::content_types::ContentType;
use crate::models::provider_states::ProviderState;

/// V4 Interaction Type
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum V4InteractionType {
  Synchronous_HTTP,
  Asynchronous_Messages,
  Synchronous_Messages
}

impl Default for V4InteractionType {
  fn default() -> Self {
    V4InteractionType::Synchronous_HTTP
  }
}

impl Display for V4InteractionType {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match *self {
      V4InteractionType::Synchronous_HTTP => f.write_str("Synchronous/HTTP"),
      V4InteractionType::Asynchronous_Messages => f.write_str("Asynchronous/Messages"),
      V4InteractionType::Synchronous_Messages => f.write_str("Synchronous/Messages")
    }
  }
}

impl V4InteractionType {
  /// Returns the V4 interaction type from the string value
  pub fn from_str(type_str: &str) -> Result<V4InteractionType, String> {
    match type_str {
      "Synchronous/HTTP" => Ok(V4InteractionType::Synchronous_HTTP),
      "Asynchronous/Messages" => Ok(V4InteractionType::Asynchronous_Messages),
      "Synchronous/Messages" => Ok(V4InteractionType::Synchronous_Messages),
      _ => Err(format!("'{}' is not a valid V4 interaction type", type_str))
    }
  }
}

/// V4 Interaction Types
#[derive(Debug, Clone)]
pub enum V4Interaction {
  SynchronousHttp {  },
  AsynchronousMessages {  }
}

impl Interaction for V4Interaction {
  fn is_request_response(&self) -> bool {
    unimplemented!()
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    unimplemented!()
  }

  fn is_message(&self) -> bool {
    unimplemented!()
  }

  fn as_message(&self) -> Option<Message> {
    unimplemented!()
  }

  fn id(&self) -> Option<String> {
    unimplemented!()
  }

  fn description(&self) -> String {
    unimplemented!()
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    unimplemented!()
  }

  fn contents(&self) -> OptionalBody {
    unimplemented!()
  }

  fn content_type(&self) -> Option<ContentType> {
    unimplemented!()
  }
}

/// V4 spec Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone)]
pub struct V4Pact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of messages between the consumer and provider.
  pub interactions: Vec<V4Interaction>,
  /// Metadata associated with this pact.
  pub metadata: BTreeMap<String, Value>
}

impl Pact for V4Pact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i as &dyn Interaction).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), json_to_string(v))).collect())),
        _ => None
      }
    }).filter(|v| v.is_some())
      .map(|v| v.unwrap())
      .collect()
  }

  fn to_json(&self, _: PactSpecification) -> Value {
    unimplemented!()
  }

  fn as_request_response_pact(&self) -> Result<RequestResponsePact, String> {
    unimplemented!()
  }

  fn as_message_pact(&self) -> Result<MessagePact, String> {
    unimplemented!()
  }

  fn as_v4_pact(&self) -> Result<V4Pact, String> {
    Ok(self.clone())
  }
}

pub fn from_json(source: &str, pact_json: &Value) -> Result<Box<dyn Pact>, String> {
  let metadata = meta_data_from_json(pact_json);
  let consumer = match pact_json.get("consumer") {
    Some(v) => Consumer::from_json(v),
    None => Consumer { name: s!("consumer") }
  };
  let provider = match pact_json.get("provider") {
    Some(v) => Provider::from_json(v),
    None => Provider { name: s!("provider") }
  };
  Ok(Box::new(V4Pact {
    consumer,
    provider,
    interactions: interactions_from_json(pact_json),
    metadata
  }))
}

fn interactions_from_json(json: &Value) -> Vec<V4Interaction> {
  match json.get("interactions") {
    Some(v) => match *v {
      Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
        match ijson.get("type") {
          Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
            Ok(i_type) => match i_type {
              V4InteractionType::Synchronous_HTTP => {
                Some(V4Interaction::SynchronousHttp {})
              }
              V4InteractionType::Asynchronous_Messages => {
                Some(V4Interaction::AsynchronousMessages {})
              }
              V4InteractionType::Synchronous_Messages => {
                warn!("Interaction type '{}' is currently unimplemented. It will be ignored", i_type);
                None
              }
            },
            Err(_) => {
              warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored", index, i_type);
              None
            }
          },
          None => {
            warn!("Interaction {} has no type attribute. It will be ignored", index);
            None
          }
        }
      }).filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect(),
      _ => vec![]
    },
    None => vec![]
  }
}

fn meta_data_from_json(pact_json: &Value) -> BTreeMap<String, Value> {
  match pact_json.get("metadata") {
    Some(v) => match *v {
      Value::Object(ref obj) => obj.iter()
        .map(|(k, v)| (k.clone(), v.clone())).collect(),
      _ => btreemap!{}
    },
    None => btreemap!{}
  }
}
