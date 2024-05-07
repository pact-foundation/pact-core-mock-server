//! Utility functions needed for mock server support

use serde_json::Value;

/// Unpack a JSON boolean value, returning a None if the JSON value is not a boolean
pub(crate) fn json_to_bool(value: &Value) -> Option<bool> {
  match value {
    Value::Bool(b) => Some(*b),
    _ => None
  }
}
