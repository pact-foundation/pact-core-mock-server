//! The `message_pact` module defines a Pact
//! that contains Messages instead of Interactions.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::*;
use maplit::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use pact_models::PactSpecification;

use crate::models::{Consumer, Interaction, Pact, ReadWritePact, RequestResponsePact};
use crate::models::determine_spec_version;
use crate::models::file_utils::with_read_lock;
use crate::models::http_utils;
use crate::models::http_utils::HttpAuth;
use crate::models::message;
use crate::models::message::Message;
use crate::models::PACT_RUST_VERSION;
use crate::models::parse_meta_data;
use crate::models::Provider;
use crate::models::v4::V4Pact;

/// Struct that represents a pact between the consumer and provider of a service.
/// It contains a list of Messages instead of Interactions, but is otherwise
/// identical to `struct Pact`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessagePact {
    /// Consumer side of the pact
    pub consumer: Consumer,
    /// Provider side of the pact
    pub provider: Provider,
    /// List of messages between the consumer and provider.
    pub messages: Vec<message::Message>,
    /// Metadata associated with this pact file.
    pub metadata: BTreeMap<String, BTreeMap<String, String>>,
    /// Specification version of this pact
    pub specification_version: PactSpecification,
}

impl Pact for MessagePact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.messages.iter().map(|i| i as &dyn Interaction).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.clone()
  }

  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V3 => Ok(json!({
        "consumer": self.consumer.to_json(),
        "provider": self.provider.to_json(),
        "messages":
        Value::Array(self.messages.iter().map(|m| m.to_json(&pact_spec)).collect()),
        "metadata": self.metadata_to_json(&pact_spec)
      })),
      PactSpecification::V4 => self.as_v4_pact()?.to_json(pact_spec),
      _ => Err(anyhow!("Message Pacts require minimum V3 specification"))
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    Err(anyhow!("Can't convert a Message Pact to a different type"))
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    Ok(self.clone())
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    let interactions = self.messages.iter()
      .map(|i| i.as_v4())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    Ok(V4Pact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      interactions,
      metadata: self.metadata.iter().map(|(k, v)| (k.clone(), json!(v))).collect()
    })
  }

  fn specification_version(&self) -> PactSpecification {
    self.specification_version.clone()
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
    match interaction.as_message() {
      None => Err(anyhow!("Can only add message interactions to this Pact")),
      Some(interaction) => {
        self.messages.push(interaction);
        Ok(())
      }
    }
  }
}

impl MessagePact {

    /// Returns the specification version of this pact
    pub fn spec_version(&self) -> PactSpecification {
      determine_spec_version("<MessagePact>", &self.metadata)
    }

    /// Creates a `MessagePact` from a `Value` struct.
    pub fn from_json(file: &str, pact_json: &Value) -> anyhow::Result<MessagePact> {
        let metadata = parse_meta_data(pact_json);
        let spec_version = determine_spec_version(file, &metadata);

        let consumer = match pact_json.get("consumer") {
            Some(v) => Consumer::from_json(v),
            None => Consumer { name: s!("consumer") }
        };
        let provider = match pact_json.get("provider") {
            Some(v) => Provider::from_json(v),
            None => Provider { name: s!("provider") }
        };

        let messages = match pact_json.get("messages") {
            Some(Value::Array(msg_arr)) => {
                let mut messages = Vec::with_capacity(msg_arr.len());
                for (ix, msg) in msg_arr.iter().enumerate() {
                    messages.push(
                        Message::from_json(ix, msg, &spec_version)?
                    );
                }
                messages
            }
            Some(_) => bail!("Expecting 'messages' field to be Array"),
            None => vec![],
        };

        Ok(MessagePact {
            consumer,
            provider,
            messages,
            metadata,
            specification_version: spec_version.clone(),
        })
    }

    /// Creates a BTreeMap of the metadata of this pact.
    pub fn metadata_to_json(&self, pact_spec: &PactSpecification) -> BTreeMap<String, Value> {
        let mut md_map: BTreeMap<String, Value> = self.metadata.iter()
            .map(|(k, v)| {
                let key = match k.as_str() {
                  "pact-specification" => s!("pactSpecification"),
                  "pact-rust" => s!("pactRust"),
                  _ => k.clone()
                };
                (key, json!(v.iter()
                  .map(|(k, v)| (k.clone(), v.clone()))
                  .collect::<BTreeMap<String, String>>()))
            })
            .collect();

        md_map.insert(
            s!("pactSpecification"),
            json!({"version" : pact_spec.version_str()}));
        md_map.insert(
            s!("pactRust"),
            json!({"version" : s!(PACT_RUST_VERSION.unwrap_or("unknown"))}));
        md_map
    }

    /// Determines the default file name for the pact.
    /// This is based on the consumer and provider names.
    pub fn default_file_name(&self) -> String {
        format!("{}-{}.json", self.consumer.name, self.provider.name)
    }

    /// Reads the pact file from a URL and parses the resulting JSON
    /// into a `MessagePact` struct
    pub fn from_url(url: &String, auth: &Option<HttpAuth>) -> anyhow::Result<MessagePact> {
        let (url, json) = http_utils::fetch_json_from_url(url, auth)?;
        MessagePact::from_json(&url, &json)
    }

    /// Writes this pact out to the provided file path.
    /// All directories in the path will automatically created.
    /// If there is already a file at the path, it will be overwritten.
    pub fn overwrite_pact(
        &self,
        path: &Path,
        pact_spec: PactSpecification,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;

        debug!("Writing new pact file to {:?}", path);
        let mut file = File::create(path)?;

        file.write_all(
            format!("{}",
                serde_json::to_string_pretty(
                    &self.to_json(pact_spec)?)?
            ).as_bytes()
        )?;

        Ok(())
    }

    /// Returns a default MessagePact struct
    pub fn default() -> MessagePact {
        MessagePact {
            consumer: Consumer { name: s!("default_consumer") },
            provider: Provider { name: s!("default_provider") },
            messages: Vec::new(),
            metadata: MessagePact::default_metadata(),
            specification_version: PactSpecification::V3,
        }
    }

    /// Returns the default metadata
    pub fn default_metadata()
    -> BTreeMap<String, BTreeMap<String, String>> {
        btreemap!{
            s!("pact-specification") =>
                btreemap!{ s!("version") =>
                    PactSpecification::V3.version_str() },
            s!("pact-rust") =>
                btreemap!{ s!("version") =>
                    s!(PACT_RUST_VERSION.unwrap_or("unknown")) },
        }
    }
}

impl ReadWritePact for MessagePact {
  fn read_pact(path: &Path) -> anyhow::Result<MessagePact> {
    with_read_lock(path, 3, &mut |f| {
      let pact_json: Value = serde_json::from_reader(f)?;
      MessagePact::from_json(&format!("{:?}", path), &pact_json)
        .map_err(|e| anyhow!(e))
    })
  }

  fn merge(&self, pact: &dyn Pact) -> anyhow::Result<Box<dyn Pact>> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let messages: Vec<Result<Message, String>> = self.messages.iter()
        .merge_join_by(pact.interactions().iter(), |a, b| {
          let cmp = Ord::cmp(&a.description, &b.description());
          if cmp == Ordering::Equal {
            Ord::cmp(&a.provider_states.iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                     &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>())
          } else {
            cmp
          }
        })
        .map(|either| match either {
          Left(i) => Ok(i.clone()),
          Right(i) => i.as_message()
            .ok_or(format!("Can't convert interaction of type {} to V3 Asynchronous/Messages", i.type_of())),
          Both(_, i) => i.as_message()
            .ok_or(format!("Can't convert interaction of type {} to V3 Asynchronous/Messages", i.type_of()))
        })
        .collect();
      let errors: Vec<String> = messages.iter()
        .filter(|i| i.is_err())
        .map(|i| i.as_ref().unwrap_err().to_string())
        .collect();
      if errors.is_empty() {
        Ok(Box::new(MessagePact {
          provider: self.provider.clone(),
          consumer: self.consumer.clone(),
          messages: messages.iter()
            .filter(|i| i.is_ok())
            .map(|i| i.as_ref().unwrap().clone()).collect(),
          metadata: self.metadata.clone(),
          specification_version: self.specification_version.clone()
        }))
      } else {
        Err(anyhow!("Unable to merge pacts: {}", errors.join(", ")))
      }
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;

  use super::*;

  #[test]
    fn default_file_name_is_based_in_the_consumer_and_provider() {
        let pact = MessagePact { consumer: Consumer { name: s!("consumer") },
            provider: Provider { name: s!("provider") },
            messages: vec![],
            metadata: btreemap!{},
            specification_version: PactSpecification::V1_1
        };
        expect!(pact.default_file_name()).to(be_equal_to("consumer-provider.json"));
    }

    #[test]
    fn load_empty_pact() {
        let pact_json = r#"{}"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.provider.name).to(be_equal_to("provider"));
        expect!(pact.consumer.name).to(be_equal_to("consumer"));
        expect!(pact.messages.iter()).to(have_count(0));
        expect!(pact.metadata.iter()).to(have_count(0));
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_metadata() {
        let pact_json = r#"{}"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_spec_version() {
        let pact_json = r#"{
            "metadata" : {
            }
        }"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn missing_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {

                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
    }

    #[test]
    fn empty_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": ""
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
    }

    #[test]
    fn correct_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": "1.0.0"
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V1));
    }

    #[test]
    fn invalid_version_in_spec_version() {
        let pact_json = r#"{
            "metadata" : {
                "pact-specification": {
                    "version": "znjclkazjs"
                }
            }
        }"#;
        let pact = MessagePact::from_json(
            &s!(""),
            &serde_json::from_str(pact_json).unwrap()
        ).unwrap();
        expect!(pact.specification_version).to(be_equal_to(PactSpecification::Unknown));
    }

    #[test]
    fn load_basic_pact() {
        let pact_json = r#"
        {
            "provider": {
                "name": "Alice Service"
            },
            "consumer": {
                "name": "Consumer"
            },
            "messages": [
                {
                    "description": "Message Description",
                    "contents": {
                        "hello": "world"
                    },
                    "metadata": {
                        "contentType": "application/json"
                    }
                }
            ]
        }
        "#;
        let pact = MessagePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
        expect!(pact.as_ref()).to(be_ok());
        let pact = pact.unwrap();
        expect!(&pact.provider.name).to(be_equal_to("Alice Service"));
        expect!(&pact.consumer.name).to(be_equal_to("Consumer"));

        expect!(pact.messages.iter()).to(have_count(1));
        let message = pact.messages[0].clone();
        expect!(message.description)
            .to(be_equal_to("Message Description"));
        expect!(message.contents.str_value())
            .to(be_equal_to("{\"hello\":\"world\"}"));

        expect!(pact.specification_version).to(be_equal_to(PactSpecification::V3));
        expect!(pact.metadata.iter()).to(have_count(0));
    }

    #[test]
    #[ignore]
    fn to_json() {
        let pact_json = r#"
        {
            "provider": {
                "name": "Alice Service"
            },
            "consumer": {
                "name": "Consumer"
            },
            "messages": [
                {
                    "description": "Message Description",
                    "contents": {
                        "hello": "world"
                    },
                    "metadata": {
                        "contentType": "application/json"
                    }
                }
            ]
        }
        "#;
        let pact = MessagePact::from_json(&s!(""), &serde_json::from_str(pact_json).unwrap());
        expect!(pact.as_ref()).to(be_ok());
        let pact = pact.unwrap();
        let contents = pact.to_json(PactSpecification::V3);
        expect!(contents.unwrap().to_string()).to(be_equal_to("{\"consumer\":{\"name\":\"Consumer\"},\"messages\":[{\"contents\":\"{\\\"hello\\\":\\\"world\\\"}\",\"description\":\"Message Description\",\"metadata\":{}}],\"metadata\":{\"pactRust\":{\"version\":\"0.9.2\"},\"pactSpecification\":{\"version\":\"3.0.0\"}},\"provider\":{\"name\":\"Alice Service\"}}"));
    }
}
