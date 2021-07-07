//! V4 specification models

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::*;
use maplit::*;
use nom::lib::std::fmt::Formatter;
use serde_json::{json, Value};

use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::file_utils::with_read_lock;
use pact_models::generators::{Generators, generators_from_json, generators_to_json};
use pact_models::http_parts::HttpPart;
use pact_models::json_utils::{hash_json, json_to_string};
use pact_models::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use pact_models::provider_states::{self, ProviderState};
use pact_models::v4::http_parts::{body_from_json, HttpRequest, HttpResponse};
use pact_models::v4::V4InteractionType;
use pact_models::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

use crate::models::{
  Interaction,
  Pact,
  PACT_RUST_VERSION,
  ReadWritePact,
  RequestResponseInteraction,
  RequestResponsePact
};
use crate::models::message::Message;
use crate::models::message_pact::MessagePact;
use crate::models::v4::message_parts::MessageContents;
use crate::models::v4::sync_message::SynchronousMessages;

pub mod sync_message;
pub mod message_parts;

/// V4 Interaction trait
pub trait V4Interaction: Interaction + Send + Sync {
  /// Convert the interaction to a JSON Value
  fn to_json(&self) -> Value;

  /// Convert the interaction to its super trait
  fn to_super(&self) -> &dyn Interaction;

  /// Key for this interaction
  fn key(&self) -> Option<String>;

  /// Clones this interaction and wraps it in a box
  fn boxed_v4(&self) -> Box<dyn V4Interaction>;

  /// Annotations and comments associated with this interaction
  fn comments(&self) -> HashMap<String, Value>;

  /// Mutable access to the annotations and comments associated with this interaction
  fn comments_mut(&mut self) -> &mut HashMap<String, Value>;

  /// Type of this V4 interaction
  fn v4_type(&self) -> V4InteractionType;
}

impl Display for dyn V4Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(i) = self.as_v4_http() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_sync_message() {
      std::fmt::Display::fmt(&i, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Clone for Box<dyn V4Interaction> {
  fn clone(&self) -> Self {
    if let Some(http) = self.as_v4_http() {
      Box::new(http)
    } else if let Some(message) = self.as_v4_async_message() {
      Box::new(message)
    } else if let Some(message) = self.as_v4_sync_message() {
      Box::new(message)
    } else {
      panic!("Internal Error - Tried to clone an interaction that was not valid")
    }
  }
}

/// V4 HTTP Interaction Type
#[derive(Debug, Clone, Eq)]
pub struct SynchronousHttp {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider states for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  pub provider_states: Vec<provider_states::ProviderState>,
  /// Request of the interaction
  pub request: HttpRequest,
  /// Response of the interaction
  pub response: HttpResponse,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool
}

impl SynchronousHttp {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> SynchronousHttp {
    SynchronousHttp {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Parse the JSON into a SynchronousHttp interaction
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<SynchronousHttp> {
    if json.is_object() {
      let id = json.get("_id").map(|id| json_to_string(id));
      let key = json.get("key").map(|id| json_to_string(id));
      let description = match json.get("description") {
        Some(v) => match *v {
          Value::String(ref s) => s.clone(),
          _ => v.to_string()
        },
        None => format!("Interaction {}", index)
      };
      let comments = match json.get("comments") {
        Some(v) => match v {
          Value::Object(map) => map.iter()
            .map(|(k, v)| (k.clone(), v.clone())).collect(),
          _ => {
            warn!("Interaction comments must be a JSON Object, but received {}. Ignoring", v);
            Default::default()
          }
        },
        None => Default::default()
      };
      let provider_states = provider_states::ProviderState::from_json(json);
      let request = json.get("request").cloned().unwrap_or_default();
      let response = json.get("response").cloned().unwrap_or_default();
      Ok(SynchronousHttp {
        id,
        key,
        description,
        provider_states,
        request: HttpRequest::from_json(&request),
        response: HttpResponse::from_json(&response),
        comments,
        pending: json.get("pending")
          .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default()
      })
    } else {
      Err(anyhow!("Expected a JSON object for the interaction, got '{}'", json))
    }
  }
}

impl V4Interaction for SynchronousHttp {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Synchronous_HTTP.to_string(),
      "key": self.key.clone().unwrap_or_else(|| self.calc_hash()),
      "description": self.description.clone(),
      "request": self.request.to_json(),
      "response": self.response.to_json(),
      "pending": self.pending
    });

    if !self.provider_states.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    if !self.comments.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("comments".to_string(), self.comments.iter()
        .map(|(k, v)| (k.clone(), v.clone())).collect());
    }

    json
  }

  fn to_super(&self) -> &dyn Interaction {
    self
  }

  fn key(&self) -> Option<String> {
    self.key.clone()
  }

  fn boxed_v4(&self) -> Box<dyn V4Interaction> {
    Box::new(self.clone())
  }

  fn comments(&self) -> HashMap<String, Value> {
    self.comments.clone()
  }

  fn comments_mut(&mut self) -> &mut HashMap<String, Value> {
    &mut self.comments
  }

  fn v4_type(&self) -> V4InteractionType {
    V4InteractionType::Synchronous_HTTP
  }
}

impl Interaction for SynchronousHttp {
  fn type_of(&self) -> String {
    format!("V4 {}", self.v4_type())
  }

  fn is_request_response(&self) -> bool {
    true
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    Some(RequestResponseInteraction {
      id: self.id.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      request: self.request.as_v3_request(),
      response: self.response.as_v3_response()
    })
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
    true
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction>> {
    Some(self.boxed_v4())
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    Some(self.clone())
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

  fn pending(&self) -> bool {
    self.pending
  }
}

impl Default for SynchronousHttp {
  fn default() -> Self {
    SynchronousHttp {
      id: None,
      key: None,
      description: "Synchronous/HTTP Interaction".to_string(),
      provider_states: vec![],
      request: HttpRequest::default(),
      response: HttpResponse::default(),
      comments: Default::default(),
      pending: false
    }
  }
}

impl PartialEq for SynchronousHttp {
  fn eq(&self, other: &Self) -> bool {
    self.description == other.description && self.provider_states == other.provider_states &&
      self.request == other.request && self.response == other.response &&
      self.pending == other.pending
  }
}

impl Hash for SynchronousHttp {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.request.hash(state);
    self.response.hash(state);
    self.pending.hash(state);
  }
}

impl Display for SynchronousHttp {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Http Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           pending, self.id, self.description, self.provider_states, self.request, self.response)
  }
}

/// Asynchronous interactions as a sequence of messages
#[derive(Debug, Clone, Eq)]
pub struct AsynchronousMessage {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider state for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// The contents of the message
  pub contents: MessageContents,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool
}

impl AsynchronousMessage {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> AsynchronousMessage {
    AsynchronousMessage {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Returns the content type of the message by returning the content type associated with
  /// the body, or by looking it up in the message metadata
  pub fn message_content_type(&self) -> Option<ContentType> {
    self.contents.message_content_type()
  }

  /// Parse the JSON into an AsynchronousMessage interaction
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<AsynchronousMessage> {
    if json.is_object() {
      let id = json.get("_id").map(|id| json_to_string(id));
      let key = json.get("key").map(|id| json_to_string(id));
      let description = match json.get("description") {
        Some(v) => match *v {
          Value::String(ref s) => s.clone(),
          _ => v.to_string()
        },
        None => format!("Interaction {}", index)
      };
      let comments = match json.get("comments") {
        Some(v) => match v {
          Value::Object(map) => map.iter()
            .map(|(k, v)| (k.clone(), v.clone())).collect(),
          _ => {
            warn!("Interaction comments must be a JSON Object, but received {}. Ignoring", v);
            Default::default()
          }
        },
        None => Default::default()
      };
      let provider_states = provider_states::ProviderState::from_json(json);
      let metadata = match json.get("metadata") {
        Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
          (k.clone(), v.clone())
        }).collect(),
        _ => hashmap! {}
      };
      let as_headers = metadata_to_headers(&metadata);
      Ok(AsynchronousMessage {
        id,
        key,
        description,
        provider_states,
        contents: MessageContents {
          metadata,
          contents: body_from_json(&json, "contents", &as_headers),
          matching_rules: matchers_from_json(&json, &None),
          generators: generators_from_json(&json)
        },
        comments,
        pending: json.get("pending")
          .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default()
      })
    } else {
      Err(anyhow!("Expected a JSON object for the interaction, got '{}'", json))
    }
  }
}

impl V4Interaction for AsynchronousMessage {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Asynchronous_Messages.to_string(),
      "key": self.key.clone().unwrap_or_else(|| self.calc_hash()),
      "description": self.description.clone(),
      "pending": self.pending
    });

    if let Value::Object(body) = self.contents.contents.to_v4_json() {
      let map = json.as_object_mut().unwrap();
      map.insert("contents".to_string(), Value::Object(body));
    }

    if !self.contents.metadata.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("metadata".to_string(), Value::Object(
        self.contents.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
      ));
    }

    if !self.provider_states.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    if !self.contents.matching_rules.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("matchingRules".to_string(), matchers_to_json(&self.contents.matching_rules, &PactSpecification::V4));
    }

    if !self.contents.generators.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("generators".to_string(), generators_to_json(&self.contents.generators, &PactSpecification::V4));
    }

    if !self.comments.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("comments".to_string(), self.comments.iter()
        .map(|(k, v)| (k.clone(), v.clone())).collect());
    }

    json
  }

  fn to_super(&self) -> &dyn Interaction {
    self
  }

  fn key(&self) -> Option<String> {
    self.key.clone()
  }

  fn boxed_v4(&self) -> Box<dyn V4Interaction> {
    Box::new(self.clone())
  }

  fn comments(&self) -> HashMap<String, Value> {
    self.comments.clone()
  }

  fn comments_mut(&mut self) -> &mut HashMap<String, Value> {
    &mut self.comments
  }

  fn v4_type(&self) -> V4InteractionType {
    V4InteractionType::Asynchronous_Messages
  }
}

impl Interaction for AsynchronousMessage {
  fn type_of(&self) -> String {
    format!("V4 {}", self.v4_type())
  }

  fn is_request_response(&self) -> bool {
    false
  }

  fn as_request_response(&self) -> Option<RequestResponseInteraction> {
    None
  }

  fn is_message(&self) -> bool {
    true
  }

  fn as_message(&self) -> Option<Message> {
    Some(Message {
      id: self.id.clone(),
      description: self.description.clone(),
      provider_states: self.provider_states.clone(),
      contents: self.contents.contents.clone(),
      metadata: self.contents.metadata.clone(),
      matching_rules: self.contents.matching_rules.rename("content", "body"),
      generators: self.contents.generators.clone()
    })
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
    self.contents.contents.clone()
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.contents.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.message_content_type()
  }

  fn is_v4(&self) -> bool {
    true
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction>> {
    Some(self.boxed_v4())
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    None
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    Some(self.clone())
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
    Some(self.contents.matching_rules.clone())
  }

  fn pending(&self) -> bool {
    self.pending
  }
}

impl Default for AsynchronousMessage {
  fn default() -> Self {
    AsynchronousMessage {
      id: None,
      key: None,
      description: "Asynchronous/Message Interaction".to_string(),
      provider_states: vec![],
      contents: MessageContents {
        contents: OptionalBody::Missing,
        metadata: Default::default(),
        matching_rules: Default::default(),
        generators: Default::default()
      },
      comments: Default::default(),
      pending: false
    }
  }
}

impl PartialEq for AsynchronousMessage {
  fn eq(&self, other: &Self) -> bool {
    self.description == other.description && self.provider_states == other.provider_states &&
      self.contents == other.contents && self.pending == other.pending
  }
}

impl Hash for AsynchronousMessage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.contents.contents.hash(state);
    for (k, v) in &self.contents.metadata {
      k.hash(state);
      hash_json(v, state);
    }
    self.contents.matching_rules.hash(state);
    self.contents.generators.hash(state);
    self.pending.hash(state);
  }
}

impl Display for AsynchronousMessage {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Asynchronous Message Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           pending, self.id, self.description, self.provider_states, self.contents.contents, self.contents.metadata)
  }
}

impl HttpPart for AsynchronousMessage {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
    unimplemented!()
  }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    unimplemented!()
  }

  fn body(&self) -> &OptionalBody {
    &self.contents.contents
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.contents.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.contents.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.contents.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| json_to_string(v))
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
  fn verify_json(_path: &str, pact_json: &Value, _strict: bool) -> Vec<PactFileVerificationResult> {
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

fn interactions_from_json(json: &Value, source: &str) -> Vec<Box<dyn V4Interaction>> {
  match json.get("interactions") {
    Some(v) => match *v {
      Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
        interaction_from_json(source, index, ijson).ok()
      }).filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect(),
      _ => vec![]
    },
    None => vec![]
  }
}

/// Create an interaction from a JSON struct
pub fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> anyhow::Result<Box<dyn V4Interaction>> {
  match ijson.get("type") {
    Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        match i_type {
          V4InteractionType::Synchronous_HTTP => SynchronousHttp::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Asynchronous_Messages => AsynchronousMessage::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Synchronous_Messages => SynchronousMessages::from_json(ijson, index).map(|i| i.boxed_v4())
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        Err(anyhow!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source))
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      Err(anyhow!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source))
    }
  }
}

fn metadata_to_headers(metadata: &HashMap<String, Value>) -> Option<HashMap<String, Vec<String>>> {
  if let Some(content_type) = metadata.get("contentType") {
    Some(hashmap! {
      "Content-Type".to_string() => vec![ json_to_string(content_type) ]
    })
  } else {
    None
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
