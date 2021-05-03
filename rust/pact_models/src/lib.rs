//! The `pact_models` crate provides all the structs and traits required to model a Pact.

#[allow(unused_imports)] use log::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub mod content_types;
pub mod bodies;

/// Enum defining the pact specification versions supported by the library
#[cfg_attr(feature = "ffi", repr(C))]
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

#[cfg(test)]
mod tests;
