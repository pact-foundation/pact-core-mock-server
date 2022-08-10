//! Module for handling interaction content (bodies)

use std::fmt::{Display, Formatter};
use std::str::from_utf8;

use base64::encode;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::warn;

use crate::content_types::{ContentType, ContentTypeHint};

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
  Present(Bytes, Option<ContentType>, Option<ContentTypeHint>)
}

impl OptionalBody {

  /// If the body is present in the pact file and not empty or null.
  pub fn is_present(&self) -> bool {
    matches!(*self, OptionalBody::Present(_, _, _))
  }

  /// Returns the body if present, otherwise returns None.
  pub fn value(&self) -> Option<Bytes> {
    match self {
      OptionalBody::Present(s, _, _) => Some(s.clone()),
      _ => None
    }
  }

  /// Returns the body as a UTF-8 string if present and is a textual form, otherwise returns None.
  pub fn value_as_string(&self) -> Option<String> {
    match self {
      OptionalBody::Present(s, ct, hint) => {
        if Self::is_text(ct, hint) {
          from_utf8(s)
            .map(|s| s.to_string())
            .ok()
        } else {
          None
        }
      },
      _ => None
    }
  }

  fn is_text(ct: &Option<ContentType>, hint: &Option<ContentTypeHint>) -> bool {
    if let Some(hint) = hint {
      *hint == ContentTypeHint::TEXT
    } else if let Some(ct) = ct {
      ct.is_text()
    } else {
      false
    }
  }

  /// Returns the body if present as a UTF-8 string, otherwise returns the empty string.
  #[deprecated(since = "0.4.2", note = "This does not deal with binary bodies, use value_as_str or display_string instead")]
  pub fn str_value(&self) -> &str {
    match self {
      OptionalBody::Present(s, _, _) => from_utf8(s).unwrap_or(""),
      _ => ""
    }
  }

  /// For text bodies (are present and have either a content type hint of TEXT or a content type
  /// that is a known textual form), returns the body as a UTF-8 string. Otherwise, if the body is
  /// present, will display the first 32 bytes in hexidecimal form. Otherwise returns the empty string.
  pub fn display_string(&self) -> String {
    match self {
      OptionalBody::Present(s, ct, hint) => {
        if Self::is_text(ct, hint) {
          from_utf8(s)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| self.display_bytes(32))
        } else {
          self.display_bytes(32)
        }
      },
      _ => String::default()
    }
  }

  /// If the body has a content type associated to it
  pub fn has_content_type(&self) -> bool {
    match self {
      OptionalBody::Present(_, content_type, _) => content_type.is_some(),
      _ => false
    }
  }

  /// Parsed content type of the body
  pub fn content_type(&self) -> Option<ContentType> {
    match self {
      OptionalBody::Present(_, content_type, _) =>
        content_type.clone(),
      _ => None
    }
  }

  /// Converts this body into a V4 Pact file JSON format
  pub fn to_v4_json(&self) -> Value {
    match self {
      OptionalBody::Present(bytes, content_type, ct_override) => {
        let content_type = content_type.as_ref().cloned().unwrap_or_default();
        let content_type_override = ct_override.unwrap_or_default();
        let (contents, encoded) = if content_type.is_json() {
          match serde_json::from_slice(bytes) {
            Ok(json_body) => (json_body, Value::Bool(false)),
            Err(err) => {
              warn!("Failed to parse json body: {}", err);
              (Value::String(encode(bytes)), Value::String("base64".to_string()))
            }
          }
        } else if content_type_override == ContentTypeHint::BINARY || content_type.is_binary() {
          (Value::String(encode(bytes)), Value::String("base64".to_string()))
        } else {
          match from_utf8(bytes) {
            Ok(s) => (Value::String(s.to_string()), Value::Bool(false)),
            Err(_) => (Value::String(encode(bytes)), Value::String("base64".to_string()))
          }
        };

        if let Some(ct_override) = ct_override {
          json!({
            "content": contents,
            "contentType": content_type.to_string(),
            "contentTypeHint": ct_override.to_string(),
            "encoded": encoded
          })
        } else {
          json!({
            "content": contents,
            "contentType": content_type.to_string(),
            "encoded": encoded
          })
        }
      },
      OptionalBody::Empty => json!({"content": ""}),
      _ => Value::Null
    }
  }

  /// Set the content type of the body. If the body is missing or empty, this is a no-op.
  pub fn set_content_type(&mut self, content_type: &ContentType) {
    if let OptionalBody::Present(_, ct, _) = self {
       *ct = Some(content_type.clone());
    }
  }

  pub(crate) fn display_bytes(&self, max_bytes: usize) -> String {
    if let OptionalBody::Present(bytes, _, _) = self {
      if bytes.len() <= max_bytes {
        let b_str: String = bytes.iter().map(|b| format!("{:0X}", b)).collect();
        format!("{} ({} bytes)", b_str, bytes.len())
      } else {
        let b_str: String = bytes
          .slice(0..max_bytes)
          .iter()
          .map(|b| format!("{:0X}", b))
          .collect();
        format!("{}... ({} bytes)", b_str, bytes.len())
      }
    } else {
      String::default()
    }
  }
}

impl From<String> for OptionalBody {
  fn from(s: String) -> Self {
    if s.is_empty() {
      OptionalBody::Empty
    } else {
      OptionalBody::Present(Bytes::from(s), None, None)
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
      OptionalBody::Present(buf.freeze(), None, None)
    }
  }
}

impl From<Value> for OptionalBody {
  fn from(json: Value) -> Self {
    OptionalBody::from(&json)
  }
}

impl From<&Value> for OptionalBody {
  fn from(json: &Value) -> Self {
    OptionalBody::Present(Bytes::from(json.to_string()),
                          Some(ContentType::from("application/json;charset=UTF-8")),
                          None)
  }
}

impl Display for OptionalBody {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match *self {
      OptionalBody::Missing => write!(f, "Missing"),
      OptionalBody::Empty => write!(f, "Empty"),
      OptionalBody::Null => write!(f, "Null"),
      OptionalBody::Present(ref s, ref content_type, _) => {
        if let Some(content_type) = content_type {
          write!(f, "Present({} bytes, {})", s.len(), content_type)
        } else {
          write!(f, "Present({} bytes)", s.len())
        }
      }
    }
  }
}

impl Default for OptionalBody {
  fn default() -> Self {
    OptionalBody::Missing
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;

  use crate::content_types::{ContentTypeHint, JSON, TEXT};

  use super::OptionalBody;

  #[test]
  fn display_tests() {
    expect!(format!("{}", OptionalBody::Missing)).to(be_equal_to("Missing"));
    expect!(format!("{}", OptionalBody::Empty)).to(be_equal_to("Empty"));
    expect!(format!("{}", OptionalBody::Null)).to(be_equal_to("Null"));
    expect!(format!("{}", OptionalBody::Present("hello".into(), None, None))).to(be_equal_to("Present(5 bytes)"));
    expect!(format!("{}", OptionalBody::Present("\"hello\"".into(), Some(JSON.clone()), None))).to(be_equal_to("Present(7 bytes, application/json)"));
  }

  #[test]
  fn display_bytes_test() {
    expect!(OptionalBody::Missing.display_bytes(8)).to(be_equal_to(""));
    expect!(OptionalBody::Empty.display_bytes(8)).to(be_equal_to(""));
    expect!(OptionalBody::Null.display_bytes(8)).to(be_equal_to(""));
    expect!(OptionalBody::Present("hello".into(), None, None).display_bytes(8)).to(be_equal_to("68656C6C6F (5 bytes)"));
    expect!(OptionalBody::Present("12345678".into(), None, None).display_bytes(8)).to(be_equal_to("3132333435363738 (8 bytes)"));
    expect!(OptionalBody::Present("123456789012345".into(), None, None).display_bytes(8)).to(be_equal_to("3132333435363738... (15 bytes)"));
  }

  #[test]
  fn display_string_test() {
    expect!(OptionalBody::Missing.display_string()).to(be_equal_to(""));
    expect!(OptionalBody::Empty.display_string()).to(be_equal_to(""));
    expect!(OptionalBody::Null.display_string()).to(be_equal_to(""));

    expect!(OptionalBody::Present("hello".into(), None, None).display_string()).to(be_equal_to("68656C6C6F (5 bytes)"));
    expect!(OptionalBody::Present("12345678".into(), None, None).display_string()).to(be_equal_to("3132333435363738 (8 bytes)"));
    expect!(OptionalBody::Present("123456789012345".into(), None, None).display_string()).to(be_equal_to("313233343536373839303132333435 (15 bytes)"));

    expect!(OptionalBody::Present("hello".into(), Some(TEXT.clone()), None).display_string()).to(be_equal_to("hello"));
    expect!(OptionalBody::Present("hello".into(), None, Some(ContentTypeHint::TEXT)).display_string()).to(be_equal_to("hello"));
  }

  #[test]
  fn value_as_string_test() {
    expect!(OptionalBody::Missing.value_as_string()).to(be_none());
    expect!(OptionalBody::Empty.value_as_string()).to(be_none());
    expect!(OptionalBody::Null.value_as_string()).to(be_none());

    expect!(OptionalBody::Present("hello".into(), None, None).value_as_string()).to(be_none());
    expect!(OptionalBody::Present("hello".into(), Some(TEXT.clone()), None).value_as_string()).to(be_some().value("hello"));
    expect!(OptionalBody::Present("hello".into(), None, Some(ContentTypeHint::TEXT)).value_as_string()).to(be_some().value("hello"));
    expect!(OptionalBody::Present("hello".into(), None, Some(ContentTypeHint::BINARY)).value_as_string()).to(be_none());
  }
}
