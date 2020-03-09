//! The `message` module provides all functionality to deal with messages.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use maplit::*;
use crate::models::provider_states::ProviderState;
use super::*;
use super::body_from_json;

/// Struct that defines a message.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Eq)]
pub struct Message {
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
    pub metadata: HashMap<String, String>,

    /// Matching rules
    #[serde(rename = "matchingRules")]
    #[serde(default)]
    pub matching_rules: matchingrules::MatchingRules
}

impl Message {
    /// Returns a default message
    pub fn default() -> Message {
        Message {
            description: s!("message"),
            provider_states: vec![],
            contents: OptionalBody::Missing,
            metadata: hashmap!{},
            matching_rules: matchingrules::MatchingRules::default()
        }
    }

    /// Constructs a `Message` from the `Json` struct.
    pub fn from_json(index: usize, json: &Value, spec_version: &PactSpecification) -> Result<Message, String> {
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
                let metadata = match json.get("metadata") {
                    Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                        (k.clone(), match v {
                            &Value::String(ref s) => s.clone(),
                            _ => v.to_string()
                        })
                    }).collect(),
                    _ => hashmap!{}
                };
                Ok(Message {
                     description,
                     provider_states,
                     contents: body_from_json(json, "contents", &None),
                     matching_rules: matchingrules::matchers_from_json(json, &None),
                     metadata
                })
            },
            _ => Err(s!("Messages require Pact Specification version 3 or later"))
        }
    }

    /// Determins the content type of the message
    pub fn mimetype(&self) -> String {
        match self.metadata.get("contentType") {
            Some(v) => v.clone(),
            None => s!("application/json")
        }
    }
}

fn missing_body() -> OptionalBody {
    OptionalBody::Missing
}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use expectest::expect;
    use serde_json;

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
            metadata: hashmap!{ s!("contentType") => s!("text/plain") },
            .. Message::default()
        };
        expect!(message.mimetype()).to(be_equal_to("text/plain"));
    }

    #[test]
    fn message_mimetype_defaults_to_json() {
        let message = Message::default();
        expect!(message.mimetype()).to(be_equal_to("application/json"));
    }

    #[test]
    fn ignoring_v1_provider_state_when_deserializing_message() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {}
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
            "matchingRules": {}
        }"#;

        let message: Message = serde_json::from_str(message_json).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_states).to(be_equal_to(vec![ProviderState {
            name: s!("provider state"),
            params: hashmap!(),
        }]));
        expect!(message.matching_rules.rules.iter()).to(be_empty());
    }
}
