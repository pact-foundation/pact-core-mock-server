//! The `models` module provides all the structures required to model a Pact.

use std::{fmt, fs};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::default::Default;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::Path;
use std::str;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::{iproduct, Itertools};
use itertools::EitherOrBoth::{Both, Left, Right};
use lazy_static::*;
use log::*;
use maplit::{btreemap, hashset};
use serde_json::{json, Value};

use pact_models::{Consumer, DifferenceType, http_utils, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::*;
use pact_models::file_utils::{with_read_lock, with_read_lock_for_open_file, with_write_lock};
use pact_models::http_parts::HttpPart;
use pact_models::http_utils::HttpAuth;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::MatchingRules;
use pact_models::provider_states::ProviderState;
use pact_models::request::Request;
use pact_models::response::Response;
use pact_models::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

pub use crate::models::message::Message;
pub use crate::models::message_pact::MessagePact;
use crate::models::v4::{AsynchronousMessage, interaction_from_json, SynchronousHttp, V4Interaction, V4Pact};
use crate::models::v4::sync_message::SynchronousMessages;

pub(crate) mod matchingrules;
pub(crate) mod generators;

/// Version of the library
pub const PACT_RUST_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

/// Struct that defined an interaction conflict
#[derive(Debug, Clone)]
pub struct PactConflict {
    /// Description of the interactions
    pub interaction: String,
    /// Conflict description
    pub description: String
}

/// Interaction Trait
pub trait Interaction: Debug {
  /// The type of the interaction
  fn type_of(&self) -> String;

  /// If this is a request/response interaction
  fn is_request_response(&self) -> bool;

  /// Returns the request/response interaction if it is one
  fn as_request_response(&self) -> Option<RequestResponseInteraction>;

  /// If this is a message interaction
  fn is_message(&self) -> bool;

  /// Returns the message interaction if it is one
  fn as_message(&self) -> Option<Message>;

  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  fn id(&self) -> Option<String>;

  /// Description of this interaction. This needs to be unique in the pact file.
  fn description(&self) -> String;

  /// Optional provider states for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  fn provider_states(&self) -> Vec<ProviderState>;

  /// Body of the response or message
  #[deprecated(
    since = "0.8.14",
    note = "Some interactions have multiple contents (like request/response), so it is impossible \
      to know which to return for this method"
  )]
  fn contents(&self) -> OptionalBody;

  /// The contents of the part to use for verification. For example, with HTTP interactions, this
  /// will be the response body
  fn contents_for_verification(&self) -> OptionalBody;

  /// Determine the content type of the interaction. If a `Content-Type` header or metadata value is present, the
  /// value of that value will be returned. Otherwise, the contents will be inspected.
  #[deprecated(
  since = "0.8.14",
  note = "Some interactions have multiple contents (like request/response), so it is impossible \
      to know which to return for this method"
  )]
  fn content_type(&self) -> Option<ContentType>;

  /// If this is a V4 interaction
  fn is_v4(&self) -> bool;

  /// Returns the interaction in V4 format
  fn as_v4(&self) -> Option<Box<dyn V4Interaction>>;

  /// Returns the interaction in V4 format
  fn as_v4_http(&self) -> Option<SynchronousHttp>;

  /// Returns the interaction in V4 format
  fn as_v4_async_message(&self) -> Option<AsynchronousMessage>;

  /// Returns the interaction in V4 format
  fn as_v4_sync_message(&self) -> Option<SynchronousMessages>;

  /// Clones this interaction and wraps it in a Box
  fn boxed(&self) -> Box<dyn Interaction + Send>;

  /// Clones this interaction and wraps it in an Arc
  fn arced(&self) -> Arc<dyn Interaction + Send>;

  /// Clones this interaction and wraps it in an Arc and Mutex
  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync>>;

  /// Returns the matching rules associated with this interaction (if there are any)
  fn matching_rules(&self) -> Option<MatchingRules>;

  /// If this interaction is pending (V4 only)
  fn pending(&self) -> bool { false }
}

impl Display for dyn Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(req_res) = self.as_request_response() {
      std::fmt::Display::fmt(&req_res, f)
    } else if let Some(mp) = self.as_message() {
      std::fmt::Display::fmt(&mp, f)
    } else if let Some(mp) = self.as_v4_http() {
      std::fmt::Display::fmt(&mp, f)
    } else if let Some(mp) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&mp, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Clone for Box<dyn Interaction> {
  fn clone(&self) -> Self {
    if self.is_v4() {
      if let Some(http) = self.as_v4_http() {
        Box::new(http)
      } else if let Some(message) = self.as_v4_async_message() {
        Box::new(message)
      } else {
        panic!("Internal Error - Tried to clone an interaction that was not valid")
      }
    } else if let Some(req_res) = self.as_request_response() {
      Box::new(req_res)
    } else if let Some(mp) = self.as_message() {
      Box::new(mp)
    } else {
      panic!("Internal Error - Tried to clone an interaction that was not valid")
    }
  }
}

/// Struct that defines an interaction (request and response pair)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestResponseInteraction {
    /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
    pub id: Option<String>,
    /// Description of this interaction. This needs to be unique in the pact file.
    pub description: String,
    /// Optional provider states for the interaction.
    /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
    pub provider_states: Vec<ProviderState>,
    /// Request of the interaction
    pub request: Request,
    /// Response of the interaction
    pub response: Response
}

impl Interaction for RequestResponseInteraction {
  fn type_of(&self) -> String {
    "V3 Synchronous/HTTP".into()
  }

  fn is_request_response(&self) -> bool {
    true
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    Some(self.clone())
  }

  fn is_message(&self) -> bool {
    false
  }

  fn as_message(&self) -> Option<Message> {
    None
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
    self.response.body.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.response.body.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.response.content_type()
  }

  fn is_v4(&self) -> bool {
    false
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction>> {
    self.as_v4_http().map(|i| i.boxed_v4())
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    Some(SynchronousHttp {
      id: self.id.clone(),
      key: None,
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.as_v4_request(),
      response: self.response.as_v4_response(),
      .. Default::default()
    }.with_key())
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    None
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
    None
  }
}

impl RequestResponseInteraction {
    /// Constructs an `Interaction` from the `Value` struct.
    pub fn from_json(index: usize, pact_json: &Value, spec_version: &PactSpecification) -> RequestResponseInteraction {
        let id = pact_json.get("_id").map(|id| json_to_string(id));
        let description = match pact_json.get("description") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => format!("Interaction {}", index)
        };
        let provider_states = ProviderState::from_json(pact_json);
        let request = match pact_json.get("request") {
            Some(v) => Request::from_json(v, spec_version),
            None => Request::default()
        };
        let response = match pact_json.get("response") {
            Some(v) => Response::from_json(v, spec_version),
            None => Response::default()
        };
      RequestResponseInteraction {
          id,
          description,
          provider_states,
          request,
          response
        }
    }

    /// Converts this interaction to a `Value` struct.
    pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
        let mut value = json!({
            s!("description"): Value::String(self.description.clone()),
            s!("request"): self.request.to_json(spec_version),
            s!("response"): self.response.to_json(spec_version)
        });
        if !self.provider_states.is_empty() {
            let map = value.as_object_mut().unwrap();
            match spec_version {
                &PactSpecification::V3 => map.insert(s!("providerStates"),
                                                     Value::Array(self.provider_states.iter().map(|p| p.to_json()).collect())),
                _ => map.insert(s!("providerState"), Value::String(
                    self.provider_states.first().unwrap().name.clone()))
            };
        }
        value
    }

    /// Returns list of conflicts if this interaction conflicts with the other interaction.
    ///
    /// Two interactions conflict if they have the same description and provider state, but they request and
    /// responses are not equal
    pub fn conflicts_with(&self, other: &dyn Interaction) -> Vec<PactConflict> {
      if let Some(other) = other.as_request_response() {
        if self.description == other.description && self.provider_states == other.provider_states {
          let mut conflicts = self.request.differences_from(&other.request).iter()
            .filter(|difference| match difference.0 {
              DifferenceType::MatchingRules | DifferenceType::Body => false,
              _ => true
            })
            .map(|difference| PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
            .collect::<Vec<PactConflict>>();
          for difference in self.response.differences_from(&other.response) {
            match difference.0 {
              DifferenceType::MatchingRules | DifferenceType::Body => (),
              _ => conflicts.push(PactConflict { interaction: self.description.clone(), description: difference.1.clone() })
            };
          }
          conflicts
        } else {
          vec![]
        }
      } else {
        vec![PactConflict {
          interaction: self.description.clone(),
          description: format!("You can not combine message and request/response interactions")
        }]
      }
    }

  /// Generate the JSON schema properties for the given Pact specification
  pub fn schema(_spec_version: PactSpecification) -> Value {
    json!({})
  }
}

impl Default for RequestResponseInteraction {
  fn default() -> Self {
    RequestResponseInteraction {
      id: None,
      description: s!("Default Interaction"),
      provider_states: vec![],
      request: Request::default(),
      response: Response::default()
    }
  }
}

impl Display for RequestResponseInteraction {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Interaction ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           self.id, self.description, self.provider_states, self.request, self.response)
  }
}

impl PactJsonVerifier for RequestResponseInteraction {
  fn verify_json(_path: &str, _pact_json: &Value, _strict: bool) -> Vec<PactFileVerificationResult> {
    vec![]
  }
}

/// Trait for a Pact (request/response or message)
pub trait Pact: Debug + ReadWritePact {
  /// Consumer side of the pact
  fn consumer(&self) -> Consumer;
  /// Provider side of the pact
  fn provider(&self) -> Provider;
  /// Interactions in the Pact
  fn interactions(&self) -> Vec<&dyn Interaction>;
  /// Pact metadata
  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>>;
  /// Converts this pact to a `Value` struct.
  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value>;
  /// Attempt to downcast to a concrete Pact
  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact>;
  /// Attempt to downcast to a concrete Message Pact
  fn as_message_pact(&self) -> anyhow::Result<MessagePact>;
  /// Attempt to downcast to a concrete V4 Pact
  fn as_v4_pact(&self) -> anyhow::Result<V4Pact>;
  /// Specification version of this Pact
  fn specification_version(&self) -> PactSpecification;
  /// Clones this Pact and wraps it in a Box
  fn boxed(&self) -> Box<dyn Pact + Send>;
  /// Clones this Pact and wraps it in an Arc
  fn arced(&self) -> Arc<dyn Pact + Send>;
  /// Clones this Pact and wraps it in an Arc and Mutex
  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync>>;
  /// Adds an interactions in the Pact
  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()>;
}

pub mod message;
pub mod message_pact;
pub mod v4;

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

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i as &dyn Interaction).collect()
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
    match interaction.as_request_response() {
      None => Err(anyhow!("Can only add request/response interactions to this Pact")),
      Some(interaction) => {
        self.interactions.push(interaction);
        Ok(())
      }
    }
  }
}

/// Construct Metadata from JSON value
pub fn parse_meta_data(pact_json: &Value) -> BTreeMap<String, BTreeMap<String, String>> {
    match pact_json.get("metadata") {
        Some(v) => match *v {
            Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                let val = match *v {
                    Value::Object(ref obj) => obj.iter().map(|(k, v)| {
                        match *v {
                            Value::String(ref s) => (k.clone(), s.clone()),
                            _ => (k.clone(), v.to_string())
                        }
                    }).collect(),
                    _ => btreemap!{}
                };
                let key = match k.as_str() {
                  "pact-specification" => s!("pactSpecification"),
                  "pact-rust" => s!("pactRust"),
                  _ => k.clone()
                };
                (key, val)
            }).collect(),
            _ => btreemap!{}
        },
        None => btreemap!{}
    }
}

fn metadata_schema(spec_version: PactSpecification) -> Value {
  if spec_version < PactSpecification::V3 {
    json!({
      "properties": {
        "pactSpecification": {
          "additionalProperties": false,
          "properties": {
            "version": {
              "type": "string"
            }
          },
          "required": ["version"],
          "type": "object"
        },
        "pactSpecificationVersion": {
          "type": "string"
        },
        "pact-specification": {
          "additionalProperties": false,
          "properties": {
              "version": {
                  "type": "string"
              }
          },
          "required": ["version"],
          "type": "object"
        }
      },
      "type": "object"
    })
  } else {
    json!({
      "properties": {
        "pactSpecification": {
          "additionalProperties": false,
          "properties": {
            "version": {
              "type": "string"
            }
          },
          "required": ["version"],
          "type": "object"
        }
      },
      "type": "object"
    })
  }
}

fn parse_interactions(pact_json: &Value, spec_version: PactSpecification) -> Vec<RequestResponseInteraction> {
    match pact_json.get("interactions") {
        Some(v) => match *v {
            Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
              RequestResponseInteraction::from_json(index, ijson, &spec_version)
            }).collect(),
            _ => vec![]
        },
        None => vec![]
    }
}

/// Determines the Pact specification version from the metadata of the Pact file
pub fn determine_spec_version(file: &str, metadata: &BTreeMap<String, BTreeMap<String, String>>) -> PactSpecification {
  let specification = if metadata.contains_key("pact-specification") {
    metadata.get("pact-specification")
  } else {
    metadata.get("pactSpecification")
  };
  match specification {
    Some(spec) => {
      match spec.get("version") {
        Some(ver) => match lenient_semver::parse(ver) {
          Ok(ver) => match ver.major {
            1 => match ver.minor {
              0 => PactSpecification::V1,
              1 => PactSpecification::V1_1,
              _ => {
                warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V1 specification", ver, file);
                PactSpecification::V1
              }
            },
            2 => PactSpecification::V2,
            3 => PactSpecification::V3,
            4 => PactSpecification::V4,
            _ => {
                warn!("Unsupported specification version '{}' found in the metadata in the pact file {:?}, will try load it as a V3 specification", ver, file);
                PactSpecification::Unknown
            }
          },
          Err(err) => {
            warn!("Could not parse specification version '{}' found in the metadata in the pact file {:?}, assuming V3 specification - {}", ver, file, err);
            PactSpecification::Unknown
          }
        },
        None => {
          warn!("No specification version found in the metadata in the pact file {:?}, assuming V3 specification", file);
          PactSpecification::V3
        }
      }
    },
    None => {
      warn!("No metadata found in pact file {:?}, assuming V3 specification", file);
      PactSpecification::V3
    }
  }
}

impl RequestResponsePact {

    /// Returns the specification version of this pact
    pub fn spec_version(&self) -> PactSpecification {
        determine_spec_version(&s!("<Pact>"), &self.metadata)
    }

    /// Creates a `Pact` from a `Value` struct.
    pub fn from_json(file: &str, pact_json: &Value) -> RequestResponsePact {
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
        RequestResponsePact {
            consumer,
            provider,
            interactions: parse_interactions(pact_json, spec_version.clone()),
            metadata,
            specification_version: spec_version
        }
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

        md_map.insert(s!("pactSpecification"), json!({"version" : pact_spec.version_str()}));
        md_map.insert(s!("pactRust"), json!({"version" : s!(PACT_RUST_VERSION.unwrap_or("unknown"))}));
        md_map
    }

    /// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
    pub fn from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<RequestResponsePact> {
      http_utils::fetch_json_from_url(&url.to_string(), auth).map(|(ref url, ref json)| RequestResponsePact::from_json(url, json))
    }

    /// Returns a default RequestResponsePact struct
    pub fn default() -> RequestResponsePact {
      RequestResponsePact {
            consumer: Consumer { name: s!("default_consumer") },
            provider: Provider { name: s!("default_provider") },
            interactions: Vec::new(),
            metadata: RequestResponsePact::default_metadata(),
            specification_version: PactSpecification::V3
        }
    }

  /// Returns the default metadata
  pub fn default_metadata() -> BTreeMap<String, BTreeMap<String, String>> {
    btreemap!{
      s!("pact-specification") => btreemap!{ s!("version") => PactSpecification::V3.version_str() },
      s!("pact-rust") => btreemap!{ s!("version") => s!(PACT_RUST_VERSION.unwrap_or("unknown")) }
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
  fn read_pact(path: &Path) -> anyhow::Result<RequestResponsePact> {
    with_read_lock(path, 3, &mut |f| {
      let pact_json = serde_json::from_reader(f)
        .context("Failed to parse Pact JSON")?;
      Ok(RequestResponsePact::from_json(&format!("{:?}", path), &pact_json))
    })
  }

  fn merge(&self, pact: &dyn Pact) -> anyhow::Result<Box<dyn Pact>> {
    if self.consumer.name == pact.consumer().name && self.provider.name == pact.provider().name {
      let conflicts = iproduct!(self.interactions.clone(), pact.interactions().clone())
        .map(|i| i.0.conflicts_with(i.1))
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
  fn verify_json(_path: &str, pact_json: &Value, strict: bool) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(consumer) = values.get("consumer") {
          results.extend(Consumer::verify_json("/consumer", consumer, strict));
        } else if strict {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::ERROR, "Missing consumer"))
        } else {
          results.push(PactFileVerificationResult::new("/consumer", ResultLevel::WARNING, "Missing consumer"))
        }

        if let Some(provider) = values.get("provider") {
          results.extend(Provider::verify_json("/provider", provider, strict));
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
                  RequestResponseInteraction::verify_json(&format!("/interactions/{}", index), interaction, strict)
                }))
            }
            _ => results.push(PactFileVerificationResult::new("/interactions", ResultLevel::ERROR,
              &format!("Must be an Object, got {}", json_type_of(pact_json))))
          }
        } else {
          results.push(PactFileVerificationResult::new("/interactions", ResultLevel::WARNING, "Missing interactions"))
        }

        if let Some(metadata) = values.get("metadata") {
          results.extend(verify_metadata(metadata));
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

pub(crate) fn verify_metadata(metadata: &Value) -> Vec<PactFileVerificationResult> {
  let mut results = vec![];

  match metadata {
    Value::Object(values) => {
      let spec_value = if let Some(spec_value) = values.get("pactSpecification") {
        Some(spec_value)
      } else if let Some(spec_value) = values.get("pact-specification") {
        results.push(PactFileVerificationResult::new("/metadata", ResultLevel::WARNING,
          &format!("'pact-specification' is deprecated, use 'pactSpecification' instead")));
        Some(spec_value)
      } else {
        None
      };
      if let Some(spec) = spec_value {
        match spec {
          Value::Object(values) => {
            if let Some(version) = values.get("version") {
              match version {
                Value::Null => results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::WARNING,
                                                                            &format!("pactSpecification version is NULL"))),
                Value::String(version) => if PactSpecification::parse_version(version).is_err() {
                  results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::ERROR,
                                                               &format!("'{}' is not a valid Pact specification version", version)))
                }
                _ => results.push(PactFileVerificationResult::new("/metadata/pactSpecification/version", ResultLevel::ERROR,
                                                                  &format!("Version must be a String, got {}", json_type_of(version))))
              }
            } else {
              results.push(PactFileVerificationResult::new("/metadata/pactSpecification", ResultLevel::WARNING,
                                                           &format!("pactSpecification is missing the version attribute")));
            }
          }
          _ => results.push(PactFileVerificationResult::new("/metadata/pactSpecification", ResultLevel::ERROR,
            &format!("pactSpecification must be an Object, got {}", json_type_of(spec))))
        }
      }
    }
    _ => results.push(PactFileVerificationResult::new("/metadata", ResultLevel::ERROR,
      &format!("Metadata must be an Object, got {}", json_type_of(metadata))))
  }

  results
}

/// Converts the JSON struct into an HTTP Interaction
pub fn http_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> anyhow::Result<Box<dyn Interaction + Send>> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| i.boxed()),
    _ => Ok(Box::new(RequestResponseInteraction::from_json(0, json, spec)))
  }
}

/// Converts the JSON struct into a Message Interaction
pub fn message_interaction_from_json(source: &str, json: &Value, spec: &PactSpecification) -> anyhow::Result<Box<dyn Interaction + Send>> {
  match spec {
    PactSpecification::V4 => interaction_from_json(source, 0, json)
      .map(|i| i.boxed()),
    _ => Message::from_json(0, json, spec).map(|i| i.boxed())
  }
}

/// Reads the pact file and parses the resulting JSON into a `Pact` struct
pub fn read_pact(file: &Path) -> anyhow::Result<Box<dyn Pact>> {
  let mut f = File::open(file)?;
  read_pact_from_file(&mut f, file)
}

/// Reads the pact from the file and parses the resulting JSON into a `Pact` struct
pub fn read_pact_from_file(file: &mut File, path: &Path) -> anyhow::Result<Box<dyn Pact>> {
  let buf = with_read_lock_for_open_file(path, file, 3, &mut |f| {
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;
    Ok(buf)
  })?;
  let pact_json = serde_json::from_str(&buf)
    .context("Failed to parse Pact JSON")
    .map_err(|err| {
      error!("read_pact_from_file: {}", err);
      debug!("read_pact_from_file: file contents = '{}'", buf);
      err
    })?;
  load_pact_from_json(&*path.to_string_lossy(), &pact_json)
    .map_err(|e| anyhow!(e))
}

/// Reads the pact file from a URL and parses the resulting JSON into a `Pact` struct
pub fn load_pact_from_url(url: &str, auth: &Option<HttpAuth>) -> anyhow::Result<Box<dyn Pact>> {
  let (url, pact_json) = http_utils::fetch_json_from_url(&url.to_string(), auth)?;
  load_pact_from_json(&url, &pact_json)
}

/// Loads a Pact model from a JSON Value
pub fn load_pact_from_json(source: &str, json: &Value) -> anyhow::Result<Box<dyn Pact>> {
  match json {
    Value::Object(map) => if map.contains_key("messages") {
      let pact = MessagePact::from_json(source, json)?;
      Ok(Box::new(pact))
    } else {
      let metadata = parse_meta_data(json);
      let spec_version = determine_spec_version(source, &metadata);
      match spec_version {
        PactSpecification::V4 => v4::from_json(&source, json),
        _ => Ok(Box::new(RequestResponsePact::from_json(source, json)))
      }
    },
    _ => Err(anyhow!("Failed to parse Pact JSON from source '{}' - it is not a valid pact file", source))
  }
}

/// Trait for objects that can represent Pacts and can be read and written
pub trait ReadWritePact {
  /// Reads the pact file and parses the resulting JSON into a `Pact` struct
  fn read_pact(path: &Path) -> anyhow::Result<Self> where Self: std::marker::Sized;

  /// Merges this pact with the other pact, and returns a new Pact with the interactions sorted.
  /// Returns an error if there is a merge conflict, which will occur if the other pact is a different
  /// type, or if a V3 Pact then if any interaction has the
  /// same description and provider state and the requests and responses are different.
  fn merge(&self, other: &dyn Pact) -> anyhow::Result<Box<dyn Pact>>;

  /// Determines the default file name for the pact. This is based on the consumer and
  /// provider names.
  fn default_file_name(&self) -> String;
}

lazy_static!{
  static ref WRITE_LOCK: Mutex<()> = Mutex::new(());
}

/// Writes the pact out to the provided path. If there is an existing pact at the path, the two
/// pacts will be merged together unless overwrite is true. Returns an error if the file can not
/// be written or the pacts can not be merged.
pub fn write_pact(
  pact: Box<dyn Pact>,
  path: &Path,
  pact_spec: PactSpecification,
  overwrite: bool
) -> anyhow::Result<()> {
  fs::create_dir_all(path.parent().unwrap())?;
  let _lock = WRITE_LOCK.lock().unwrap();
  if !overwrite && path.exists() {
    debug!("Merging pact with file {:?}", path);
    let mut f = fs::OpenOptions::new().read(true).write(true).open(&path)?;
    let existing_pact = read_pact_from_file(&mut f, path)?;

    if existing_pact.specification_version() < pact.specification_version() {
      warn!("Note: Existing pact is an older specification version ({:?}), and will be upgraded",
            existing_pact.specification_version());
    }

    let merged_pact = pact.merge(existing_pact.borrow())?;
    let pact_json = serde_json::to_string_pretty(&merged_pact.to_json(pact_spec)?)?;

    with_write_lock(path, &mut f, 3, &mut |f| {
      f.set_len(0)?;
      f.seek(SeekFrom::Start(0))?;
      f.write_all(pact_json.as_bytes())?;
      Ok(())
    })
  } else {
    debug!("Writing new pact file to {:?}", path);
    let result = serde_json::to_string_pretty(&pact.to_json(pact_spec)?)?;
    let mut file = File::create(path)?;
    with_write_lock(path, &mut file, 3, &mut |f| {
      f.write_all(result.as_bytes())?;
      Ok(())
    })
  }
}

#[cfg(test)]
mod tests;
