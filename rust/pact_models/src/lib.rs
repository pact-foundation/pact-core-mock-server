//! The `pact_models` crate provides all the structs and traits required to model a Pact.

use std::fmt::{Display, Formatter};
use std::str::from_utf8;

use bytes::{Bytes, BytesMut};
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use base64::encode;

use crate::content_types::ContentType;

pub mod content_types;

/// Enum defining the pact specification versions supported by the library
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
#[allow(non_camel_case_types)]
pub enum PactSpecification {
  /// Unknown or unsupported specification version
  Unknown,
  /// First version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1)
  V1,
  /// Second version of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-1.1)
  V1_1,
  /// Version two of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-2)
  V2,
  /// Version three of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-3)
  V3,
  /// Version four of the pact specification (https://github.com/pact-foundation/pact-specification/tree/version-4)
  V4
}

impl Default for PactSpecification {
  fn default() -> Self {
        PactSpecification::Unknown
    }
}

impl PactSpecification {
  /// Returns the semantic version string of the specification version.
  pub fn version_str(&self) -> String {
    match *self {
        PactSpecification::V1 => "1.0.0",
        PactSpecification::V1_1 => "1.1.0",
        PactSpecification::V2 => "2.0.0",
        PactSpecification::V3 => "3.0.0",
        PactSpecification::V4 => "4.0",
        _ => "unknown"
    }.into()
  }

  /// Returns a descriptive string of the specification version.
  pub fn to_string(&self) -> String {
    match *self {
      PactSpecification::V1 => "V1",
      PactSpecification::V1_1 => "V1.1",
      PactSpecification::V2 => "V2",
      PactSpecification::V3 => "V3",
      PactSpecification::V4 => "V4",
      _ => "unknown"
    }.into()
  }
}

/// Struct that defines the consumer of the pact.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Consumer {
  /// Each consumer should have a unique name to identify it.
  pub name: String
}

impl Consumer {
  /// Builds a `Consumer` from the `Json` struct.
  pub fn from_json(pact_json: &Value) -> Consumer {
    let val = match pact_json.get("name") {
      Some(v) => match v.clone() {
        Value::String(s) => s,
        _ => v.to_string()
      },
      None => "consumer".to_string()
    };
    Consumer { name: val.clone() }
  }

  /// Converts this `Consumer` to a `Value` struct.
  pub fn to_json(&self) -> Value {
    json!({ "name" : self.name })
  }
}

/// Struct that defines a provider of a pact.
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct Provider {
  /// Each provider should have a unique name to identify it.
  pub name: String
}

impl Provider {
  /// Builds a `Provider` from a `Value` struct.
  pub fn from_json(pact_json: &Value) -> Provider {
    let val = match pact_json.get("name") {
      Some(v) => match v.clone() {
        Value::String(s) => s,
        _ => v.to_string()
      },
      None => "provider".to_string()
    };
    Provider { name: val.clone() }
  }

  /// Converts this `Provider` to a `Value` struct.
  pub fn to_json(&self) -> Value {
    json!({ "name" : self.name })
  }
}

/// Enum that defines the four main states that a body of a request and response can be in a pact
/// file.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OptionalBody {
  /// A body is missing if it is not present in the pact file
  Missing,
  /// An empty body that is present in the pact file.
  Empty,
  /// A JSON body that is the null value. This state is to protect other language implementations
  /// from null values. It is treated as `Empty`.
  Null,
  /// A non-empty body that is present in the pact file.
  Present(Bytes, Option<ContentType>)
}

impl OptionalBody {

  /// If the body is present in the pact file and not empty or null.
  pub fn is_present(&self) -> bool {
    match *self {
      OptionalBody::Present(_, _) => true,
      _ => false
    }
  }

  /// Returns the body if present, otherwise returns the empty buffer.
  pub fn value(&self) -> Option<Bytes> {
    match self {
      OptionalBody::Present(s, _) => Some(s.clone()),
      _ => None
    }
  }

  /// Returns the body if present as a UTF-8 string, otherwise returns the empty string.
  pub fn str_value(&self) -> &str {
    match self {
      OptionalBody::Present(s, _) => from_utf8(s).unwrap_or(""),
      _ => ""
    }
  }

  /// If the body has a content type associated to it
  pub fn has_content_type(&self) -> bool {
    match self {
      OptionalBody::Present(_, content_type) => content_type.is_some(),
      _ => false
    }
  }

  /// Parsed content type of the body
  pub fn content_type(&self) -> Option<ContentType> {
    match self {
      OptionalBody::Present(_, content_type) =>
        content_type.clone(),
      _ => None
    }
  }

  /// Converts this body into a V4 Pact file JSON format
  pub fn to_v4_json(&self) -> Value {
    match self {
      OptionalBody::Present(bytes, _) => {
        let content_type = self.content_type().unwrap_or_default();
        let (contents, encoded) = if content_type.is_json() {
          match serde_json::from_slice(bytes) {
            Ok(json_body) => (json_body, Value::Bool(false)),
            Err(err) => {
              warn!("Failed to parse json body: {}", err);
              (Value::String(encode(bytes)), Value::String("base64".to_string()))
            }
          }
        } else if content_type.is_binary() {
          (Value::String(encode(bytes)), Value::String("base64".to_string()))
        } else {
          match from_utf8(bytes) {
            Ok(s) => (Value::String(s.to_string()), Value::Bool(false)),
            Err(_) => (Value::String(encode(bytes)), Value::String("base64".to_string()))
          }
        };
        json!({
          "content": contents,
          "contentType": content_type.to_string(),
          "encoded": encoded
        })
      },
      OptionalBody::Empty => json!({"content": ""}),
      _ => Value::Null
    }
  }
}

impl From<String> for OptionalBody {
  fn from(s: String) -> Self {
    if s.is_empty() {
      OptionalBody::Empty
    } else {
      OptionalBody::Present(Bytes::from(s), None)
    }
  }
}

impl <'a> From<&'a str> for OptionalBody {
  fn from(s: &'a str) -> Self {
    if s.is_empty() {
      OptionalBody::Empty
    } else {
      let mut buf = BytesMut::with_capacity(0);
      buf.extend_from_slice(s.as_bytes());
      OptionalBody::Present(buf.freeze(), None)
    }
  }
}

impl Display for OptionalBody {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match *self {
      OptionalBody::Missing => write!(f, "Missing"),
      OptionalBody::Empty => write!(f, "Empty"),
      OptionalBody::Null => write!(f, "Null"),
      OptionalBody::Present(ref s, ref content_type) => {
        if let Some(content_type) = content_type {
          write!(f, "Present({} bytes, {})", s.len(), content_type)
        } else {
          write!(f, "Present({} bytes)", s.len())
        }
      }
    }
  }
}

#[cfg(test)]
mod tests;
