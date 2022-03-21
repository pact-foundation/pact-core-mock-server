//! Synchronous interactions as a request message to a sequence of response messages

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use itertools::Itertools;
use log::*;
use serde_json::{json, Map, Value};

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::interaction::Interaction;
use crate::json_utils::{is_empty, json_to_string};
use crate::matchingrules::MatchingRules;
use crate::message::Message;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::interaction::{InteractionMarkup, parse_plugin_config, V4Interaction};
use crate::v4::message_parts::MessageContents;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

/// Synchronous interactions as a request message to a sequence of response messages
#[derive(Debug, Clone, Eq)]
pub struct SynchronousMessage {
  /// Interaction ID. This will only be set if the Pact file was fetched from a Pact Broker
  pub id: Option<String>,
  /// Unique key for this interaction
  pub key: Option<String>,
  /// A description for the interaction. Must be unique within the Pact file
  pub description: String,
  /// Optional provider state for the interaction.
  /// See https://docs.pact.io/getting_started/provider_states for more info on provider states.
  pub provider_states: Vec<ProviderState>,
  /// Annotations and comments associated with this interaction
  pub comments: HashMap<String, Value>,
  /// Request message
  pub request: MessageContents,
  /// Response messages
  pub response: Vec<MessageContents>,

  /// If this interaction is pending. Pending interactions will never fail the build if they fail
  pub pending: bool,

  /// Configuration added by plugins
  pub plugin_config: HashMap<String, HashMap<String, Value>>,

  /// Text markup to use to render the interaction in a UI
  pub interaction_markup: InteractionMarkup,

  /// Transport mechanism used with this message
  pub transport: Option<String>
}

impl SynchronousMessage {
  fn calc_hash(&self) -> String {
    let mut s = DefaultHasher::new();
    self.hash(&mut s);
    format!("{:x}", s.finish())
  }

  /// Creates a new version with a calculated key
  pub fn with_key(&self) -> SynchronousMessage {
    SynchronousMessage {
      key: Some(self.calc_hash()),
      .. self.clone()
    }
  }

  /// Parse the JSON into a SynchronousMessages structure
  pub fn from_json(json: &Value, index: usize) -> anyhow::Result<SynchronousMessage> {
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

      let provider_states = ProviderState::from_json(json);
      let request = json.get("request")
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'request' object"))?;
      let response = json.get("response")
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'response' array"))?
        .as_array()
        .ok_or_else(|| anyhow!("JSON for SynchronousMessages does not contain a 'response' array"))?;
      let responses =
        response.iter()
          .map(|message| MessageContents::from_json(message))
          .collect::<Vec<anyhow::Result<MessageContents>>>();

      let plugin_config = parse_plugin_config(json);
      let interaction_markup = json.get("interactionMarkup")
        .map(|markup| InteractionMarkup::from_json(markup)).unwrap_or_default();

      let transport = json.get("transport").map(|value| {
        match value {
          Value::String(s) => s.clone(),
          _ => value.to_string()
        }
      });

      if responses.iter().any(|res| res.is_err()) {
        let errors = responses.iter()
          .filter(|res| res.is_err())
          .map(|res| res.as_ref().unwrap_err().to_string())
          .join(", ");
        Err(anyhow!("Failed to parse SynchronousMessages responses - {}", errors))
      } else {
        Ok(SynchronousMessage {
          id,
          key,
          description,
          provider_states,
          comments,
          request: MessageContents::from_json(request)?,
          response: responses.iter().map(|res| res.as_ref().unwrap().clone()).collect(),
          pending: json.get("pending")
            .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default(),
          plugin_config,
          interaction_markup,
          transport
        })
      }
    } else {
      Err(anyhow!("Expected a JSON object for the interaction, got '{}'", json))
    }
  }
}

impl V4Interaction for SynchronousMessage {
  fn to_json(&self) -> Value {
    let mut json = json!({
      "type": V4InteractionType::Synchronous_Messages.to_string(),
      "key": self.key.clone().unwrap_or_else(|| self.calc_hash()),
      "description": self.description.clone(),
      "pending": self.pending,
      "request": self.request.to_json(),
      "response": self.response.iter().map(|m| m.to_json()).collect_vec()
    });

    if !self.provider_states.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("providerStates".to_string(), Value::Array(
        self.provider_states.iter().map(|p| p.to_json()).collect()));
    }

    let comments: Map<String, Value> = self.comments.iter()
      .filter(|(_k, v)| !is_empty(v))
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect();
    if !comments.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("comments".to_string(), Value::Object(comments));
    }

    if !self.plugin_config.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("pluginConfiguration".to_string(), self.plugin_config.iter()
        .map(|(k, v)|
          (k.clone(), Value::Object(v.iter().map(|(k, v)| (k.clone(), v.clone())).collect()))
        ).collect());
    }

    if !self.interaction_markup.is_empty() {
      let map = json.as_object_mut().unwrap();
      map.insert("interactionMarkup".to_string(), self.interaction_markup.to_json());
    }

    if let Some(transport) = &self.transport {
      let map = json.as_object_mut().unwrap();
      map.insert("transport".to_string(), Value::String(transport.clone()));
    }

    json
  }

  fn to_super(&self) -> &(dyn Interaction + Send + Sync) {
    self
  }

  fn to_super_mut(&mut self) -> &mut (dyn Interaction + Send + Sync) {
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
    V4InteractionType::Synchronous_Messages
  }

  fn plugin_config(&self) -> HashMap<String, HashMap<String, Value>> {
    self.plugin_config.clone()
  }

  fn plugin_config_mut(&mut self) -> &mut HashMap<String, HashMap<String, Value>> {
    &mut self.plugin_config
  }

  fn interaction_markup(&self) -> InteractionMarkup {
    self.interaction_markup.clone()
  }

  fn interaction_markup_mut(&mut self) -> &mut InteractionMarkup {
    &mut self.interaction_markup
  }

  fn transport(&self) -> Option<String> {
    self.transport.clone()
  }

  fn set_transport(&mut self, transport: Option<String>) {
    self.transport = transport.clone();
  }
}

impl Interaction for SynchronousMessage {
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

  fn set_id(&mut self, id: Option<String>) {
    self.id = id;
  }

  fn set_description(&mut self, description: &str) {
    self.description = description.to_string();
  }

  fn provider_states(&self) -> Vec<ProviderState> {
    self.provider_states.clone()
  }

  fn provider_states_mut(&mut self) -> &mut Vec<ProviderState> {
    &mut self.provider_states
  }

  fn contents(&self) -> OptionalBody {
    OptionalBody::Missing
  }

  fn contents_for_verification(&self) -> OptionalBody {
    self.response.first().map(|message| message.contents.clone()).unwrap_or_default()
  }

  fn content_type(&self) -> Option<ContentType> {
    self.request.message_content_type()
  }

  fn is_v4(&self) -> bool {
    true
  }

  fn as_v4(&self) -> Option<Box<dyn V4Interaction>> {
    Some(self.boxed_v4())
  }

  fn as_v4_mut(&mut self) -> Option<&mut dyn V4Interaction> {
    Some(self)
  }

  fn as_v4_http(&self) -> Option<SynchronousHttp> {
    None
  }

  fn as_v4_async_message(&self) -> Option<AsynchronousMessage> {
    None
  }

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    Some(self.clone())
  }

  fn as_v4_http_mut(&mut self) -> Option<&mut SynchronousHttp> {
    None
  }

  fn is_v4_sync_message(&self) -> bool {
    true
  }

  fn as_v4_async_message_mut(&mut self) -> Option<&mut AsynchronousMessage> {
    None
  }

  fn as_v4_sync_message_mut(&mut self) -> Option<&mut SynchronousMessage> {
    Some(self)
  }

  fn boxed(&self) -> Box<dyn Interaction + Send + Sync> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Interaction + Send + Sync> {
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

impl Default for SynchronousMessage {
  fn default() -> Self {
    SynchronousMessage {
      id: None,
      key: None,
      description: "Synchronous/Message Interaction".to_string(),
      provider_states: vec![],
      comments: Default::default(),
      request: Default::default(),
      response: Default::default(),
      pending: false,
      plugin_config: Default::default(),
      interaction_markup: Default::default(),
      transport: None
    }
  }
}

impl PartialEq for SynchronousMessage {
  fn eq(&self, other: &Self) -> bool {
    self.description == other.description && self.provider_states == other.provider_states &&
      self.request == other.request && self.response == other.response &&
      self.pending == other.pending
  }
}

impl Hash for SynchronousMessage {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.description.hash(state);
    self.provider_states.hash(state);
    self.request.hash(state);
    self.response.hash(state);
    self.pending.hash(state);
  }
}

impl Display for SynchronousMessage {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    let pending = if self.pending { " [PENDING]" } else { "" };
    write!(f, "V4 Synchronous Message Interaction{} ( id: {:?}, description: \"{}\", provider_states: {:?}, request: {}, response: {:?} )",
           pending, self.id, self.description, self.provider_states, self.request, self.response)
  }
}
