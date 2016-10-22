//! The `message` module provides all functionality to deal with messages.

use std::collections::HashMap;
use rustc_serialize::json::Json;
use super::*;
use super::{matchers_from_json, body_from_json};

/// Struct that defines a message.
#[derive(PartialEq, Debug, Clone, Eq)]
pub struct Message {
    /// Description of this message interaction. This needs to be unique in the pact file.
    pub description: String,
    /// Optional provider state for the interaction.
    /// See http://docs.pact.io/documentation/provider_states.html for more info on provider states.
    pub provider_state: Option<String>,
    /// The contents of the message
    pub contents: OptionalBody,
    /// Metadata associated with this message.
    pub metadata: HashMap<String, String>,
    /// Matching rules
    pub matching_rules: Option<Matchers>
}

impl Message {
    /// Returns a default message
    pub fn default() -> Message {
        Message {
            description: s!("message"),
            provider_state: None,
            contents: OptionalBody::Missing,
            metadata: hashmap!{},
            matching_rules: None
        }
    }

    /// Constructs a `Message` from the `Json` struct.
    pub fn from_json(index: usize, json: &Json, spec_version: &PactSpecification) -> Result<Message, String> {
        match spec_version {
            &PactSpecification::V3 => {
                let description = match json.find("description") {
                    Some(v) => match *v {
                        Json::String(ref s) => s.clone(),
                        _ => v.to_string()
                    },
                    None => format!("Message {}", index)
                };
                let provider_state = match json.find("providerState") {
                    Some(v) => match *v {
                        Json::String(ref s) => if s.is_empty() {
                            None
                        } else {
                            Some(s.clone())
                        },
                        Json::Null => None,
                        _ => Some(v.to_string())
                    },
                    None => None
                };
                let metadata = match json.find("metadata") {
                    Some(&Json::Object(ref v)) => v.iter().map(|(k, v)| {
                        (k.clone(), match v {
                            &Json::String(ref s) => s.clone(),
                            _ => v.to_string()
                        })
                    }).collect(),
                    _ => hashmap!{}
                };
                Ok(Message {
                     description: description,
                     provider_state: provider_state,
                     contents: body_from_json(json, "contents", &None),
                     matching_rules: matchers_from_json(json, &None),
                     metadata: metadata
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;
    use expectest::prelude::*;
    use rustc_serialize::json::Json;

    #[test]
    fn loading_message_from_json() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state",
            "matchingRules": {}
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("String"));
        expect!(message.provider_state).to(be_some().value("provider state"));
        expect!(message.matching_rules).to(be_none());
    }

    #[test]
    fn defaults_to_number_if_no_description() {
        let message_json = r#"{
            "providerState": "provider state"
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.description).to(be_equal_to("Message 0"));
    }

    #[test]
    fn defaults_to_none_if_no_provider_state() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_state).to(be_none());
        expect!(message.matching_rules).to(be_none());
    }

    #[test]
    fn defaults_to_none_if_provider_state_null() {
        let message_json = r#"{
            "providerState": null
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.provider_state).to(be_none());
    }

    #[test]
    fn returns_an_error_if_the_spec_version_is_less_than_three() {
        let message_json = r#"{
            "description": "String",
            "providerState": "provider state"
        }"#;
        let result = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V1);
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
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to("{\"hello\":\"world\"}"));
    }

    #[test]
    fn message_with_non_json_body() {
        let message_json = r#"{
            "contents": "hello world",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to("hello world"));
    }

    #[test]
    fn message_with_empty_body() {
        let message_json = r#"{
            "contents": "",
            "metadata": {
                "contentType": "text/plain"
            }
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
        expect!(message.contents).to(be_equal_to(""));
    }

    #[test]
    fn message_with_missing_body() {
        let message_json = r#"{
        }"#;
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
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
        let message = Message::from_json(0, &Json::from_str(message_json).unwrap(), &PactSpecification::V3).unwrap();
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
}
