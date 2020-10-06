//! V4 specification models

use std::fmt;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Display;
use std::string::ToString;

use log::*;
use maplit::*;
use nom::lib::std::fmt::Formatter;
use serde_json::{Value, json};

use crate::models::{Consumer, Interaction, Pact, PactSpecification, Provider, RequestResponsePact, OptionalBody, RequestResponseInteraction, provider_states, matchingrules, generators};
use crate::models::json_utils::json_to_string;
use crate::models::message_pact::MessagePact;
use crate::models::message::Message;
use crate::models::content_types::ContentType;
use crate::models::provider_states::ProviderState;
use crate::json::value_of;
use crate::models::v4::http_parts::{HttpRequest, HttpResponse};

/// V4 Interaction Type
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
pub enum V4InteractionType {
  /// Synchronous HTTP Request Response
  Synchronous_HTTP,
  /// Asynchronous Messages
  Asynchronous_Messages,
  /// Synchronous Messages in the form Request message -> Response messages
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

pub mod http_parts;

/// V4 Interaction Types
#[derive(Debug, Clone)]
pub enum V4Interaction {
  /// Synchronous HTTP request/response interaction
  SynchronousHttp {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    id: Option<String>,
    /// A description for the interaction. Must be unique within the Pact file
    description: String,
    /// Optional provider states for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    provider_states: Vec<provider_states::ProviderState>,
    /// Request of the interaction
    request: HttpRequest,
    /// Response of the interaction
    response: HttpResponse
  },

  /// Asynchronous interactions as a sequence of messages
  AsynchronousMessages {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    id: Option<String>,
    /// A description for the interaction. Must be unique within the Pact file
    description: String,
    /// Optional provider state for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    provider_states: Vec<ProviderState>,
    /// The contents of the message
    contents: OptionalBody,
    /// Metadata associated with this message.
    metadata: HashMap<String, Value>,
    /// Matching rules
    matching_rules: matchingrules::MatchingRules,
    /// Generators
    generators: generators::Generators
  }
}

impl Interaction for V4Interaction {
  fn is_request_response(&self) -> bool {
    match self {
      V4Interaction::SynchronousHttp { .. } => true,
      _ => false
    }
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    unimplemented!()
  }

  fn is_message(&self) -> bool {
    match self {
      V4Interaction::AsynchronousMessages { .. } => true,
      _ => false
    }
  }

  fn as_message(&self) -> Option<Message> {
    unimplemented!()
  }

  fn id(&self) -> Option<String> {
    match self {
      V4Interaction::SynchronousHttp { id, .. } => id.clone(),
      V4Interaction::AsynchronousMessages { id, .. } => id.clone()
    }
  }

  fn description(&self) -> String {
    match self {
      V4Interaction::SynchronousHttp { description, .. } => description.clone(),
      V4Interaction::AsynchronousMessages { description, .. } => description.clone()
    }
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    match self {
      V4Interaction::SynchronousHttp { provider_states, .. } => provider_states.clone(),
      V4Interaction::AsynchronousMessages { provider_states, .. } => provider_states.clone()
    }
  }

  fn contents(&self) -> OptionalBody {
    unimplemented!()
  }

  fn content_type(&self) -> Option<ContentType> {
    unimplemented!()
  }
}

impl Default for V4Interaction {
  fn default() -> Self {
    V4Interaction::SynchronousHttp {
      id: None,
      description: "Synchronous/HTTP Interaction".to_string(),
      provider_states: vec![],
      request: HttpRequest::default(),
      response: HttpResponse::default()
    }
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

  fn specification_version(&self) -> PactSpecification {
    PactSpecification::V4
  }
}

/// Creates a V4 Pact from the provided JSON struct
pub fn from_json(source: &str, pact_json: &Value) -> Result<Box<dyn Pact>, String> {
  let metadata = meta_data_from_json(pact_json);
  let consumer = match pact_json.get("consumer") {
    Some(v) => Consumer::from_json(v),
    None => Consumer { name: "consumer".into() }
  };
  let provider = match pact_json.get("provider") {
    Some(v) => Provider::from_json(v),
    None => Provider { name: "provider".into() }
  };
  Ok(Box::new(V4Pact {
    consumer,
    provider,
    interactions: interactions_from_json(pact_json, source),
    metadata
  }))
}

fn interactions_from_json(json: &Value, source: &str) -> Vec<V4Interaction> {
  match json.get("interactions") {
    Some(v) => match *v {
      Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
        interaction_from_json(source, index, ijson)
      }).filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect(),
      _ => vec![]
    },
    None => vec![]
  }
}

fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> Option<V4Interaction> {
  match ijson.get("type") {
    Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        let id = ijson.get("_id").map(|id| value_of(id));
        let description = match ijson.get("description") {
          Some(v) => match *v {
            Value::String(ref s) => s.clone(),
            _ => v.to_string()
          },
          None => format!("Interaction {}", index)
        };
        let provider_states = provider_states::ProviderState::from_json(ijson);
        match i_type {
          V4InteractionType::Synchronous_HTTP => {
            let request = ijson.get("request").cloned().unwrap_or_default();
            let response = ijson.get("response").cloned().unwrap_or_default();
            Some(V4Interaction::SynchronousHttp {
              id,
              description,
              provider_states,
              request: HttpRequest::from_json(&request),
              response: HttpResponse::from_json(&response)
            })
          }
          V4InteractionType::Asynchronous_Messages => {
            Some(V4Interaction::AsynchronousMessages {
              id,
              description,
              provider_states,
              contents: OptionalBody::Missing,
              metadata: hashmap! {},
              matching_rules: matchingrules::matchers_from_json(ijson, &None),
              generators: generators::generators_from_json(ijson)
            })
          }
          V4InteractionType::Synchronous_Messages => {
            warn!("Interaction type '{}' is currently unimplemented. It will be ignored. Source: {}", i_type, source);
            None
          }
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        None
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      None
    }
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

#[cfg(test)]
mod tests;
