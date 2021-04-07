//! V4 specification models

use std::fmt;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap};
use std::collections::hash_map::DefaultHasher;
use std::fmt::{Display, Debug};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::string::ToString;
use std::sync::{Arc, Mutex};

use anyhow::Context as _;
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use log::*;
use maplit::*;
use nom::lib::std::fmt::Formatter;
use serde_json::{json, Value};

use crate::models::{Consumer, detect_content_type_from_bytes, generators, Interaction, matchingrules, OptionalBody, Pact, PACT_RUST_VERSION, PactSpecification, Provider, provider_states, ReadWritePact, RequestResponseInteraction, RequestResponsePact, HttpPart};
use crate::models::content_types::ContentType;
use crate::models::generators::{generators_to_json, Generators};
use crate::models::json_utils::{hash_json, json_to_string};
use crate::models::matchingrules::{matchers_to_json, MatchingRules};
use crate::models::message::Message;
use crate::models::message_pact::MessagePact;
use crate::models::provider_states::ProviderState;
use crate::models::v4::http_parts::{body_from_json, HttpRequest, HttpResponse};
use crate::models::file_utils::with_read_lock;

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

/// V4 Interaction trait
pub trait V4Interaction: Interaction {
  /// Convert the interaction to a JSON Value
  fn to_json(&self) -> Value;

  /// Convert the interaction to its super trait
  fn to_super(&self) -> &dyn Interaction;

  /// Key for this interaction
  fn key(&self) -> Option<String>;

  /// Clones this interaction and wraps it in a box
  fn boxed_v4(&self) -> Box<dyn V4Interaction>;
}

impl Debug for dyn V4Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(i) = self.as_v4_http() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&i, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Display for dyn V4Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(i) = self.as_v4_http() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_async_message() {
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
  pub response: HttpResponse
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
}

impl V4Interaction for SynchronousHttp {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Synchronous_HTTP.to_string(),
      "key": self.key.clone().unwrap_or_else(|| self.calc_hash()),
      "description": self.description.clone(),
      "request": self.request.to_json(),
      "response": self.response.to_json()
    });

    if !self.provider_states.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
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
}

impl Interaction for SynchronousHttp {
  fn type_of(&self) -> String {
    format!("V4 {}", V4InteractionType::Synchronous_HTTP)
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

  fn boxed(&self) -> Box<dyn Interaction> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Interaction> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn matching_rules(&self) -> Option<MatchingRules> {
    None
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
      response: HttpResponse::default()
    }
  }
}

impl PartialEq for SynchronousHttp {
  fn eq(&self, other: &Self) -> bool {
    self.description == other.description && self.provider_states == other.provider_states &&
      self.request == other.request && self.response == other.response
  }
}

impl Hash for SynchronousHttp {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.request.hash(state);
    self.response.hash(state);
  }
}

impl Display for SynchronousHttp {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "V4 Http Interaction ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {} )",
           self.id, self.description, self.provider_states, self.request, self.response)
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
  pub contents: OptionalBody,
  /// Metadata associated with this message.
  pub metadata: HashMap<String, Value>,
  /// Matching rules
  pub matching_rules: matchingrules::MatchingRules,
  /// Generators
  pub generators: generators::Generators
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
}

impl V4Interaction for AsynchronousMessage {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Asynchronous_Messages.to_string(),
      "key": self.key.clone().unwrap_or_else(|| self.calc_hash()),
      "description": self.description.clone()
    });

    if let Value::Object(body) = self.contents.to_v4_json() {
      let map = json.as_object_mut().unwrap();
      map.insert("contents".to_string(), Value::Object(body));
    }

    if !self.metadata.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("metadata".to_string(), Value::Object(
        self.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
      ));
    }

    if !self.provider_states.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    if !self.matching_rules.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("matchingRules".to_string(), matchers_to_json(&self.matching_rules, &PactSpecification::V4));
    }

    if !self.generators.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("generators".to_string(), generators_to_json(&self.generators, &PactSpecification::V4));
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
}

impl Interaction for AsynchronousMessage {
  fn type_of(&self) -> String {
    format!("V4 {}", V4InteractionType::Asynchronous_Messages)
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
      contents: self.contents.clone(),
      metadata: self.metadata.iter().map(|(k, v)| (k.clone(), json_to_string(v))).collect(),
      matching_rules: self.matching_rules.rename("content", "body"),
      generators: self.generators.clone()
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
    self.contents.clone()
  }

  fn content_type(&self) -> Option<ContentType> {
    calc_content_type(&self.contents, &metadata_to_headers(&self.metadata))
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

  fn boxed(&self) -> Box<dyn Interaction> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Interaction> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Interaction + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn matching_rules(&self) -> Option<MatchingRules> {
    Some(self.matching_rules.clone())
  }
}

impl Default for AsynchronousMessage {
  fn default() -> Self {
    AsynchronousMessage {
      id: None,
      key: None,
      description: "Asynchronous/Message Interaction".to_string(),
      provider_states: vec![],
      contents: OptionalBody::Missing,
      metadata: Default::default(),
      matching_rules: Default::default(),
      generators: Default::default()
    }
  }
}

impl PartialEq for AsynchronousMessage {
  fn eq(&self, other: &Self) -> bool {
    self.description == other.description && self.provider_states == other.provider_states &&
      self.contents == other.contents && self.metadata == other.metadata &&
      self.matching_rules == other.matching_rules &&
      self.generators == other.generators
  }
}

impl Hash for AsynchronousMessage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.contents.hash(state);
    for (k, v) in &self.metadata {
      k.hash(state);
      hash_json(v, state);
    }
    self.matching_rules.hash(state);
    self.generators.hash(state);
  }
}

impl Display for AsynchronousMessage {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "V4 Asynchronous Message Interaction ( id: {:?}, description: \"{}\", provider_states: {:?}, contents: {}, metadata: {:?} )",
           self.id, self.description, self.provider_states, self.contents, self.metadata)
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
    &self.contents
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.metadata.iter().find(|(k, _)| {
      let key = k.to_ascii_lowercase();
      key == "contenttype" || key == "content-type"
    }).map(|(_, v)| v.as_str().unwrap_or_default().to_string())
  }
}

fn calc_content_type(body: &OptionalBody, headers: &Option<HashMap<String, Vec<String>>>) -> Option<ContentType> {
  body.content_type()
    .or_else(|| headers.as_ref().map(|h| {
      match h.iter().find(|kv| kv.0.to_lowercase() == "content-type") {
        Some((_, v)) => ContentType::parse(v[0].as_str()).ok(),
        None => None
      }
    }).flatten())
    .or_else(|| if body.is_present() {
      detect_content_type_from_bytes(&*body.value().unwrap_or_default())
    } else {
      None
    })
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

  fn to_json(&self, _: PactSpecification) -> Value {
    json!({
      "consumer": self.consumer.to_json(),
      "provider": self.provider.to_json(),
      "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json()).collect()),
      "metadata": self.metadata_to_json()
    })
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
      serde_json::from_reader::<_, Value>(f)
        .context("Failed to parse Pact JSON")
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

  fn merge(&self, other: &dyn Pact) -> Result<V4Pact, String> {
    if self.consumer.name == other.consumer().name && self.provider.name == other.provider().name {
      Ok(V4Pact {
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
      })
    } else {
      Err(s!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
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
pub fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> Result<Box<dyn V4Interaction>, String> {
  match ijson.get("type") {
    Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        let id = ijson.get("_id").map(|id| json_to_string(id));
        let key = ijson.get("key").map(|id| json_to_string(id));
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
            Ok(Box::new(SynchronousHttp {
              id,
              key,
              description,
              provider_states,
              request: HttpRequest::from_json(&request),
              response: HttpResponse::from_json(&response)
            }))
          }
          V4InteractionType::Asynchronous_Messages => {
            let metadata = match ijson.get("metadata") {
              Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
                (k.clone(), v.clone())
              }).collect(),
              _ => hashmap!{}
            };
            let as_headers = metadata_to_headers(&metadata);
            Ok(Box::new(AsynchronousMessage {
              id,
              key,
              description,
              provider_states,
              metadata,
              contents: body_from_json(ijson, "contents", &as_headers),
              matching_rules: matchingrules::matchers_from_json(ijson, &None),
              generators: generators::generators_from_json(ijson)
            }))
          }
          V4InteractionType::Synchronous_Messages => {
            warn!("Interaction type '{}' is currently unimplemented. It will be ignored. Source: {}", i_type, source);
            Err(format!("Interaction type '{}' is currently unimplemented. It will be ignored. Source: {}", i_type, source))
          }
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        Err(format!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source))
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      Err(format!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source))
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
