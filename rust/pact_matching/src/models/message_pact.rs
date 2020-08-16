//! The `message_pact` module defines a Pact
//! that contains Messages instead of Interactions.

use std::collections::BTreeMap;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::path::Path;

use log::*;
use maplit::*;
use serde_json::{Value, json};

use crate::models::Consumer;
use crate::models::PactSpecification;
use crate::models::Provider;
use crate::models::VERSION;
use crate::models::determine_spec_version;
use crate::models::http_utils::HttpAuth;
use crate::models::http_utils;
use crate::models::message::Message;
use crate::models::message;
use crate::models::parse_meta_data;

/// Struct that represents a pact between the consumer and provider of a service.
/// It contains a list of Messages instead of Interactions, but is otherwise
/// identical to `struct Pact`.
#[derive(Debug, Clone)]
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

impl MessagePact {

    /// Returns the specification version of this pact
    pub fn spec_version(&self) -> PactSpecification {
        determine_spec_version(&s!("<MessagePact>"), &self.metadata)
    }

    /// Creates a `MessagePact` from a `Value` struct.
    pub fn from_json(file: &String, pact_json: &Value)
    -> Result<MessagePact, String> {
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
            Some(_) => return Err(
                "Expecting 'messages' field to be Array".to_string()
                ),
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

    /// Converts this pact to a `Value` struct.
    pub fn to_json(&self, pact_spec: PactSpecification) -> Value {
        json!({
            s!("consumer"): self.consumer.to_json(),
            s!("provider"): self.provider.to_json(),
            s!("messages"):
                Value::Array(self.messages.iter().map(
                    |i| serde_json::to_value(i).unwrap())
                    .collect()),
            s!("metadata"): json!(self.metadata_to_json(&pact_spec))
        })
    }

    /// Creates a BTreeMap of the metadata of this pact.
    pub fn metadata_to_json(&self, pact_spec: &PactSpecification)
    -> BTreeMap<String, Value> {
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
            json!({"version" : s!(VERSION.unwrap_or("unknown"))}));
        md_map
    }

    /// Determines the default file name for the pact.
    /// This is based on the consumer and provider names.
    pub fn default_file_name(&self) -> String {
        format!("{}-{}.json", self.consumer.name, self.provider.name)
    }

    /// Reads the pact file and parses the resulting JSON
    /// into a `MessagePact` struct
    pub fn read_pact(file: &Path) -> Result<MessagePact, String> {
        let mut f = File::open(file)
            .map_err(|e| format!("{:?}", e))?;
        let pact_json: Value = serde_json::from_reader(&mut f)
            .map_err(|e| format!("Failed to parse MessagePact JSON - {}", e))?;
        MessagePact::from_json(&format!("{:?}", file), &pact_json)
    }

    /// Reads the pact file from a URL and parses the resulting JSON
    /// into a `MessagePact` struct
    pub fn from_url(url: &String, auth: &Option<HttpAuth>)
    -> Result<MessagePact, String> {
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
    ) -> Result<(), String> {
        fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("{:?}", e))?;

        debug!("Writing new pact file to {:?}", path);
        let mut file = File::create(path)
            .map_err(|e| format!("{:?}", e))?;

        file.write_all(
            format!("{}",
                serde_json::to_string_pretty(
                    &self.to_json(pact_spec)).unwrap()
            ).as_bytes()
        ).map_err(|e| format!("{:?}", e))?;

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
                    s!(VERSION.unwrap_or("unknown")) },
        }
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
}
