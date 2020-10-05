//! V4 specification models

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use serde_json::Value;

use crate::models::{Consumer, Interaction, Pact, PactSpecification, Provider, RequestResponsePact};
use crate::models::message_pact::MessagePact;

/// V4 Interaction Types
#[derive(Debug, Clone)]
pub enum V4Interaction {

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
    unimplemented!()
  }

  fn provider(&self) -> Provider {
    unimplemented!()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    unimplemented!()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    unimplemented!()
  }

  fn to_json(&self, pact_spec: PactSpecification) -> Value {
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

pub fn load_pact(source: &str, pact_json: &Value) -> Result<Box<dyn Pact>, String> {
  unimplemented!()
}
