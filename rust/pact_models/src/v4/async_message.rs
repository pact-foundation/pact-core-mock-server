//! Models for asynchronous message interactions

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use log::warn;
use maplit::hashmap;
use serde_json::{json, Value, Map};

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::generators::{Generators, generators_from_json, generators_to_json};
use crate::http_parts::HttpPart;
use crate::interaction::Interaction;
use crate::json_utils::{hash_json, json_to_string, is_empty};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::message::Message;
use crate::PactSpecification;
use crate::provider_states::ProviderState;
use crate::sync_interaction::RequestResponseInteraction;
use crate::v4::http_parts::body_from_json;
use crate::v4::interaction::{V4Interaction, parse_plugin_config, InteractionMarkup};
use crate::v4::message_parts::{MessageContents, metadata_to_headers};
use crate::v4::sync_message::SynchronousMessage;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

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
  pub pending: bool,

  /// Configuration added by plugins
  pub plugin_config: HashMap<String, HashMap<String, Value>>,

  /// Text markup to use to render the interaction in a UI
  pub interaction_markup: InteractionMarkup
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

      let provider_states = ProviderState::from_json(json);
      let metadata = match json.get("metadata") {
        Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
          (k.clone(), v.clone())
        }).collect(),
        _ => hashmap! {}
      };
      let as_headers = metadata_to_headers(&metadata);

      let plugin_config = parse_plugin_config(json);
      let interaction_markup = json.get("interactionMarkup")
        .map(|markup| InteractionMarkup::from_json(markup)).unwrap_or_default();

      Ok(AsynchronousMessage {
        id,
        key,
        description,
        provider_states,
        contents: MessageContents {
          metadata,
          contents: body_from_json(&json, "contents", &as_headers),
          matching_rules: matchers_from_json(&json, &None)?,
          generators: generators_from_json(&json)?,
        },
        comments,
        pending: json.get("pending")
          .map(|value| value.as_bool().unwrap_or_default()).unwrap_or_default(),
        plugin_config,
        interaction_markup
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

  fn plugin_config(&self) -> HashMap<String, HashMap<String, Value>> {
    self.plugin_config.clone()
  }

  fn interaction_markup(&self) -> InteractionMarkup {
    self.interaction_markup.clone()
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

  fn as_v4_sync_message(&self) -> Option<SynchronousMessage> {
    None
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
      pending: false,
      plugin_config: Default::default(),
      interaction_markup: Default::default()
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

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use crate::interaction::Interaction;
  use crate::matchingrules;
  use crate::matchingrules::MatchingRule;
  use crate::v4::async_message::AsynchronousMessage;
  use crate::v4::message_parts::MessageContents;

  #[test]
  fn when_downgrading_message_to_v3_rename_the_matching_rules_from_content_to_body() {
    let message = AsynchronousMessage {
      contents: MessageContents {
        matching_rules: matchingrules! { "content" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] } },
        .. MessageContents::default()
      },
      .. AsynchronousMessage::default()
    };
    let v3 = message.as_message().unwrap();
    expect!(v3.matching_rules).to(be_equal_to(
      matchingrules! { "body" => { "user_id" => [ MatchingRule::Regex("^[0-9]+$".into()) ] }}
    ));
  }
}
