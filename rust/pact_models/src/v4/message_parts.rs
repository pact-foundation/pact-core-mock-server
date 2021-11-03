//! Structs for shared parts of message interactions

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use anyhow::anyhow;
use maplit::hashmap;
use serde_json::{json, Value};

use crate::bodies::OptionalBody;
use crate::content_types::ContentType;
use crate::generators::{Generators, generators_from_json, generators_to_json};
use crate::http_parts::HttpPart;
use crate::json_utils::{hash_json, json_to_string};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::message::Message;
use crate::PactSpecification;
use crate::v4::calc_content_type;
use crate::v4::http_parts::body_from_json;

/// Contents of a message interaction
#[derive(Default, Clone, Debug, Eq)]
pub struct MessageContents {
  /// The contents of the message
  pub contents: OptionalBody,
  /// Metadata associated with this message.
  pub metadata: HashMap<String, Value>,
  /// Matching rules
  pub matching_rules: MatchingRules,
  /// Generators
  pub generators: Generators,
}

impl MessageContents {
  /// Parse the JSON into a MessageContents struct
  pub fn from_json(json: &Value) -> anyhow::Result<MessageContents> {
    if json.is_object() {
      let metadata = match json.get("metadata") {
        Some(&Value::Object(ref v)) => v.iter().map(|(k, v)| {
          (k.clone(), v.clone())
        }).collect(),
        _ => hashmap! {}
      };
      let as_headers = metadata_to_headers(&metadata);
      Ok(MessageContents {
        metadata,
        contents: body_from_json(json, "contents", &as_headers),
        matching_rules: matchers_from_json(json, &None)?,
        generators: generators_from_json(json)?,
      })
    } else {
      Err(anyhow!("Expected a JSON object for the message contents, got '{}'", json))
    }
  }

  /// Convert this message part into a JSON struct
  pub fn to_json(&self) -> Value {
    let mut json = json!({});

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


  /// Returns the content type of the message by returning the content type associated with
  /// the body, or by looking it up in the message metadata
  pub fn message_content_type(&self) -> Option<ContentType> {
    calc_content_type(&self.contents, &metadata_to_headers(&self.metadata))
  }

  /// Convert this message contents to a V3 asynchronous message
  pub fn as_v3_message(&self) -> Message {
    Message {
      contents: self.contents.clone(),
      metadata: self.metadata.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone(),
      .. Message::default()
    }
  }
}

impl Display for MessageContents {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Message Contents ( contents: {}, metadata: {:?} )", self.contents,
           self.metadata)
  }
}

impl Hash for MessageContents {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.contents.hash(state);
    for (k, v) in &self.metadata {
      k.hash(state);
      hash_json(v, state);
    }
    self.matching_rules.hash(state);
    self.generators.hash(state);
  }
}

impl PartialEq for MessageContents {
  fn eq(&self, other: &Self) -> bool {
    self.contents == other.contents && self.metadata == other.metadata &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }
}

impl HttpPart for MessageContents {
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
    }).map(|(_, v)| json_to_string(&v[0]))
  }
}

pub(crate) fn metadata_to_headers(metadata: &HashMap<String, Value>) -> Option<HashMap<String, Vec<String>>> {
  metadata.get("contentType").map(|content_type| {
    hashmap! {
      "Content-Type".to_string() => vec![ json_to_string(content_type) ]
    }
  })
}
