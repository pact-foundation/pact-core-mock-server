//! Synchronous HTTP Request/Response Pact

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::warn;
use maplit::{btreemap, hashset};
use serde_json::{json, Value};

use crate::{Consumer, PactSpecification, Provider};
#[cfg(not(target_family = "wasm"))] use crate::file_utils::with_read_lock;
#[cfg(not(target_family = "wasm"))] use crate::http_utils::{self, HttpAuth};
use crate::interaction::{Interaction, PactConflict, parse_interactions};
use crate::iterator_utils::CartesianProductIterator;
use crate::message_pact::MessagePact;
use crate::pact::{determine_spec_version, metadata_schema, Pact, parse_meta_data, ReadWritePact, verify_metadata};
use crate::PACT_RUST_VERSION;
use crate::plugins::PluginData;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::pact::V4Pact;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestResponsePact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of interactions between the consumer and provider.
  pub interactions: Vec<RequestResponseInteraction>,
  /// Metadata associated with this pact file.
  pub metadata: BTreeMap<String, BTreeMap<String, String>>,
  /// Specification version of this pact
  pub specification_version: PactSpecification
}

impl Pact for RequestResponsePact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<Box<dyn Interaction + Send + Sync>> {
    self.interactions.iter().map(|i| i.boxed()).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.clone()
  }

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V4 => self.as_v4_pact()?.to_json(pact_spec),
      _ => Ok(json!({
          "consumer": self.consumer.to_json(),
          "provider": self.provider.to_json(),
          "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json(&pact_spec)).collect()),
          "metadata": self.metadata_to_json(&pact_spec)
      }))
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    Ok(self.clone())
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    Err(anyhow!("Can't convert a Request/response Pact to a different type"))
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_v4())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    Ok(V4Pact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      interactions,
      metadata: self.metadata.iter().map(|(k, v)| (k.clone(), json!(v))).collect(),
      .. V4Pact::default()
    })
  }

  fn specification_version(&self) -> PactSpecification {
    self.specification_version.clone()
  }

  fn boxed(&self) -> Box<dyn Pact + Send + Sync> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Pact + Send + Sync> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()> {
    match interaction.as_request_response() {
      None => Err(anyhow!("Can only add request/response interactions to this Pact")),
      Some(interaction) => {
        self.interactions.push(interaction);
        Ok(())
      }
    }
  }

  fn requires_plugins(&self) -> bool {
    false
  }

  fn plugin_data(&self) -> Vec<PluginData> {
    Vec::default()
  }

  fn is_v4(&self) -> bool {
    false
  }

  fn add_plugin(
    &mut self,
    _name: &str,
    _version: &str,
    _plugin_data: Option<HashMap<String, Value>>
  ) -> anyhow::Result<()> {
    Err(anyhow!("Plugins can only be used with V4 format pacts"))
  }
}

impl RequestResponsePact {

  /// Returns the specification version of this pact
  pub fn spec_version(&self) -> PactSpecification {
    determine_spec_version(&"<Pact>".to_string(), &self.metadata)
  }

  /// Creates a `Pact` from a `Value` struct.
  pub fn from_json(file: &str, pact_json: &Value
  ) -> anyhow::Result<RequestResponsePact> {
    let metadata = parse_meta_data(pact_json);
    let spec_version = determine_spec_version(file, &metadata);

    let consumer = match pact_json.get("consumer") {
      Some(v) => Consumer::from_json(v),
      None => Consumer { name: "consumer".to_string() }
    };
    let provider = match pact_json.get("provider") {
      Some(v) => Provider::from_json(v),
      None => Provider { name: "provider".to_string() }
    };
    Ok(RequestResponsePact {
      consumer,
      provider,
      interactions: parse_interactions(pact_json, spec_version.clone())?,
      metadata,
      specification_version: spec_version,
    })
  }

  /// Creates a BTreeMap of the metadata of this pact.
  pub fn metadata_to_json(&self, pact_spec: &PactSpecification) -> BTreeMap<String, Value> {
    let mut md_map: BTreeMap<String, Value> = self.metadata.iter()
      .map(|(k, v)| {
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, json!(v.iter()
                  .map(|(k, v)| (k.clone(), v.clone()))
                  .collect::<BTreeMap<String, String>>()))
      })
      .collect();

    md_map.insert("pactSpecification".to_string(), json!({"version" : pact_spec.version_str()}));
    md_map.insert("pactRust".to_string(), json!({"version" : PACT_RUST_VERSION.unwrap_or("unknown").to_string()}));
    md_map
  }

  /// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
  #[cfg(not(target_family = "wasm"))]
  pub fn from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<RequestResponsePact> {
    let (url, json) = http_utils::fetch_json_from_url(&url.to_string(), auth)?;
    RequestResponsePact::from_json(&url, &json)
  }

  /// Returns a default RequestResponsePact struct
  pub fn default() -> RequestResponsePact {
    RequestResponsePact {
      consumer: Consumer { name: "default_consumer".to_string() },
      provider: Provider { name: "default_provider".to_string() },
      interactions: Vec::new(),
      metadata: RequestResponsePact::default_metadata(),
      specification_version: PactSpecification::V3
    }
  }

  /// Returns the default metadata
  pub fn default_metadata() -> BTreeMap<String, BTreeMap<String, String>> {
    btreemap!{
      "pact-specification".to_string() => btreemap!{ "version".to_string() => PactSpecification::V3.version_str() },
      "pact-rust".to_string() => btreemap!{ "version".to_string() => PACT_RUST_VERSION.unwrap_or("unknown").to_string() }
    }
  }

  /// Generate the JSON schema properties for the given Pact specification
  pub fn schema(spec_version: PactSpecification) -> Value {
    json!({
      "properties": {
        "consumer": Consumer::schema(spec_version),
        "interactions": {
          "description": "The interactions between the consumer and provider",
          "type": "array",
          "items": RequestResponseInteraction::schema(spec_version),
        },
        "metadata": {
          "description": "Metadata associated with the Pact file",
          "$ref": "#/definitions/metadata"
        },
        "provider": Provider::schema(spec_version)
      },
      "required": [
        "consumer",
        "interactions",
        "provider"
      ],
      "definitions": {
        "metadata": metadata_schema(spec_version)
      }
    })
  }
}

impl ReadWritePact for RequestResponsePact {
  #[cfg(not(target_family = "wasm"))]
  fn read_pact(path: &Path) -> anyhow::Result<RequestResponsePact> {
    with_read_lock(path, 3, &mut |f| {
      let pact_json = serde_json::from_reader(f)
        .context("Failed to parse Pact JSON")?;
      RequestResponsePact::from_json(&format!("{:?}", path), &pact_json)
    })
  }

  fn merge(&self, pact: &dyn Pact) -> anyhow::Result<Box<dyn Pact + Send + Sync>> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let conflicts = CartesianProductIterator::new(&self.interactions, &pact.interactions())
        .map(|(i1, i2)| i1.conflicts_with(i2.as_ref()))
        .filter(|conflicts| !conflicts.is_empty())
        .collect::<Vec<Vec<PactConflict>>>();
      let num_conflicts = conflicts.len();
      if num_conflicts > 0 {
        warn!("The following conflicting interactions where found:");
        for interaction_conflicts in conflicts {
          warn!(" Interaction '{}':", interaction_conflicts.first().unwrap().interaction);
          for conflict in interaction_conflicts {
            warn!("   {}", conflict.description);
          }
        }
        Err(anyhow!("Unable to merge pacts, as there were {} conflict(s) between the interactions. Please clean out your pact directory before running the tests.",
                    num_conflicts))
      } else {
        let interactions: Vec<Result<RequestResponseInteraction, String>> = self.interactions.iter()
          .merge_join_by(pact.interactions().iter(), |a, b| {
            let cmp = Ord::cmp(&a.provider_states.iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                               &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>());
            if cmp == Ordering::Equal {
              Ord::cmp(&a.description, &b.description())
            } else {
              cmp
            }
          })
          .map(|either| match either {
            Left(i) => Ok(i.clone()),
            Right(i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of())),
            Both(_, i) => i.as_request_response()
              .ok_or(format!("Can't convert interaction of type {} to V3 Synchronous/HTTP", i.type_of()))
          })
          .collect();

        let errors: Vec<String> = interactions.iter()
          .filter(|i| i.is_err())
          .map(|i| i.as_ref().unwrap_err().to_string())
          .collect();
        if errors.is_empty() {
          Ok(Box::new(RequestResponsePact {
            provider: self.provider.clone(),
            consumer: self.consumer.clone(),
            interactions: interactions.iter()
              .filter(|i| i.is_ok())
              .map(|i| i.as_ref().unwrap().clone()).collect(),
            metadata: self.metadata.clone(),
            specification_version: self.specification_version.clone()
          }))
        } else {
          Err(anyhow!("Unable to merge pacts: {}", errors.join(", ")))
        }
      }
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

impl PactJsonVerifier for RequestResponsePact {
  fn verify_json(_path: &str, pact_json: &Value, strict: bool, spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(consumer) = values.get("consumer") {
          results.extend(Consumer::verify_json("/consumer", consumer, strict, spec_version));
        } else if strict {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::ERROR, "Missing consumer"))
        } else {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::WARNING, "Missing consumer"))
        }

        if let Some(provider) = values.get("provider") {
          results.extend(Provider::verify_json("/provider", provider, strict, spec_version));
        } else if strict {
          results.push(PactFileVerificationResult::new("/provider", ResultLevel::ERROR, "Missing provider"))
        } else {
          results.push(PactFileVerificationResult::new("/provider", ResultLevel::WARNING, "Missing provider"))
        }

        if let Some(interactions) = values.get("interactions") {
          match interactions {
            Value::Array(values) => if values.is_empty() {
              results.push(PactFileVerificationResult::new("/interactions", ResultLevel::WARNING, "Interactions is empty"))
            } else {
              results.extend(values.iter().enumerate()
                .flat_map(|(index, interaction)| {
                  RequestResponseInteraction::verify_json(&format!("/interactions/{}", index), interaction, strict, spec_version)
                }))
            }
            _ => results.push(PactFileVerificationResult::new("/interactions", ResultLevel::ERROR,
                                                              &format!("Must be an Object, got {}", json_type_of(pact_json))))
          }
        } else {
          results.push(PactFileVerificationResult::new("/interactions", ResultLevel::WARNING, "Missing interactions"))
        }

        if let Some(metadata) = values.get("metadata") {
          results.extend(verify_metadata(metadata, spec_version));
        }

        let valid_attr = hashset! { "consumer", "provider", "interactions", "metadata" };
        for (key, _) in values {
          if !valid_attr.contains(key.as_str()) {
            results.push(PactFileVerificationResult::new(&format!("/{}", key),
                                                         if strict { ResultLevel::ERROR } else { ResultLevel::WARNING },
                                                         &format!("Unexpected attribute '{}'", key)));
          }
        }
      }
      _ => results.push(PactFileVerificationResult::new("/", ResultLevel::ERROR,
                                                        &format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

#[cfg(test)]
mod tests {

}
