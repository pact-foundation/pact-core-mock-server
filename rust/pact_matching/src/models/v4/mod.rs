//! V4 specification models

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::fmt::Debug;
use std::path::Path;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use maplit::*;
use serde_json::{json, Value};

use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::file_utils::with_read_lock;
use pact_models::interaction::Interaction;
use pact_models::json_utils::json_to_string;
use pact_models::v4::interaction::{V4Interaction, interactions_from_json};
use pact_models::v4::V4InteractionType;
use pact_models::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

use crate::models::{
  Pact,
  PACT_RUST_VERSION,
  ReadWritePact,
  RequestResponsePact
};
use crate::models::message_pact::MessagePact;

/// V4 spec Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone)]
pub struct V4Pact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of messages between the consumer and provider.
  pub interactions: Vec<Box<dyn V4Interaction>>,
  /// Metadata associated with this pact.
  pub metadata: BTreeMap<String, Value>
}

impl V4Pact {
  fn metadata_to_json(&self) -> Value {
    let mut md_map: serde_json::Map<String, Value> = self.metadata.iter()
      .map(|(k, v)| {
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, v.clone())
      })
      .collect();

    md_map.insert("pactSpecification".to_string(), json!({"version" : PactSpecification::V4.version_str()}));
    md_map.insert("pactRust".to_string(), json!({"version" : PACT_RUST_VERSION.unwrap_or("unknown")}));
    Value::Object(md_map)
  }

  /// If this Pact has any interactions of the given type
  pub fn has_interactions(&self, interaction_type: V4InteractionType) -> bool {
    self.interactions.iter().any(|interaction| interaction.v4_type() == interaction_type)
  }

  /// If this Pact has different types of interactions
  pub fn has_mixed_interactions(&self) -> bool {
    let interaction_types: HashSet<_> = self.interactions.iter().map(|i| i.v4_type()).collect();
    interaction_types.len() > 1
  }
}

impl Pact for V4Pact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i.to_super()).collect()
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

  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V4 => Ok(json!({
        "consumer": self.consumer.to_json(),
        "provider": self.provider.to_json(),
        "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json()).collect()),
        "metadata": self.metadata_to_json()
      })),
      _ => if self.has_mixed_interactions() {
        Err(anyhow!("A Pact with mixed interaction types can't be downgraded to {:?}", pact_spec))
      } else if self.interactions.is_empty() || self.has_interactions(V4InteractionType::Synchronous_HTTP) {
        self.as_request_response_pact()?.to_json(pact_spec)
      } else if self.has_interactions(V4InteractionType::Asynchronous_Messages) {
        self.as_message_pact()?.to_json(pact_spec)
      } else {
        let interaction = self.interactions.first().unwrap();
        Err(anyhow!("A Pact with {} interactions can't be downgraded to {:?}", interaction.type_of(), pact_spec))
      }
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_request_response())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    let metadata = self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), v.to_string())).collect())),
        _ => None
      }
    }).filter(|val| val.is_some())
      .map(|val| val.unwrap())
      .collect();
    Ok(RequestResponsePact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      interactions,
      metadata,
      specification_version: PactSpecification::V3
    })
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_message())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    let metadata = self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), v.to_string())).collect())),
        _ => None
      }
    }).filter(|val| val.is_some())
      .map(|val| val.unwrap())
      .collect();
    Ok(MessagePact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      messages: interactions,
      metadata,
      specification_version: PactSpecification::V3
    })
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    Ok(self.clone())
  }

  fn specification_version(&self) -> PactSpecification {
    PactSpecification::V4
  }

  fn boxed(&self) -> Box<dyn Pact + Send> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Pact + Send> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()> {
    match interaction.as_v4() {
      None => Err(anyhow!("Can only add interactions that can be converted to V4 to this Pact")),
      Some(interaction) => {
        self.interactions.push(interaction);
        Ok(())
      }
    }
  }
}

impl Default for V4Pact {
  fn default() -> Self {
    V4Pact {
      consumer: Default::default(),
      provider: Default::default(),
      interactions: vec![],
      metadata: Default::default()
    }
  }
}

impl ReadWritePact for V4Pact {
  fn read_pact(path: &Path) -> anyhow::Result<V4Pact> {
    let json = with_read_lock(path, 3, &mut |f| {
      serde_json::from_reader::<_, Value>(f).context("Failed to parse Pact JSON")
    })?;
    let metadata = meta_data_from_json(&json);
    let consumer = match json.get("consumer") {
      Some(v) => Consumer::from_json(v),
      None => Consumer { name: "consumer".into() }
    };
    let provider = match json.get("provider") {
      Some(v) => Provider::from_json(v),
      None => Provider { name: "provider".into() }
    };
    Ok(V4Pact {
      consumer,
      provider,
      interactions: interactions_from_json(&json, &*path.to_string_lossy()),
      metadata
    })
  }

  fn merge(&self, other: &dyn Pact) -> anyhow::Result<Box<dyn Pact>> {
    if self.consumer.name == other.consumer().name && self.provider.name == other.provider().name {
      Ok(Box::new(V4Pact {
        consumer: self.consumer.clone(),
        provider: self.provider.clone(),
        interactions: self.interactions.iter()
          .merge_join_by(other.interactions().iter().map(|i| i.as_v4().unwrap()), |a, b| {
            match (a.key(), b.key()) {
              (Some(key_a), Some(key_b)) => Ord::cmp(&key_a, &key_b),
              (_, _) => {
                let type_a = a.type_of();
                let type_b = b.type_of();
                let cmp = Ord::cmp(&type_a, &type_b);
                if cmp == Ordering::Equal {
                  let cmp = Ord::cmp(&a.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                                     &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>());
                  if cmp == Ordering::Equal {
                    Ord::cmp(&a.description(), &b.description())
                  } else {
                    cmp
                  }
                } else {
                  cmp
                }
              }
            }
          })
          .map(|either| {
            match either {
              Left(i) => i.clone(),
              Right(i) => i.boxed_v4(),
              Both(i, _) => i.clone()
            }
          })
          .collect(),
        metadata: self.metadata.clone()
      }))
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

impl PactJsonVerifier for V4Pact {
  fn verify_json(_path: &str, pact_json: &Value, _strict: bool, _spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(_values) => {

      }
      _ => results.push(PactFileVerificationResult::new("/", ResultLevel::ERROR,
        &format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

/// Creates a V4 Pact from the provided JSON struct
pub fn from_json(source: &str, pact_json: &Value) -> anyhow::Result<Box<dyn Pact>> {
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
