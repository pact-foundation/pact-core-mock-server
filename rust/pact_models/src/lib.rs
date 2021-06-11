//! The `pact_models` crate provides all the structs and traits required to model a Pact.

use std::fmt::{Display, Formatter};
use std::fmt;

use anyhow::anyhow;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

pub mod content_types;
pub mod bodies;
pub mod v4;
pub mod provider_states;
pub mod verify_json;
pub mod json_utils;
pub mod expression_parser;

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

impl From<&str> for PactSpecification {
  fn from(s: &str) -> Self {
    match s.to_uppercase().as_str() {
      "V1" => PactSpecification::V1,
      "V1.1" => PactSpecification::V1_1,
      "V2" => PactSpecification::V2,
      "V3" => PactSpecification::V3,
      "V4" => PactSpecification::V4,
      _ => PactSpecification::Unknown
    }
  }
}

impl From<String> for PactSpecification {
  fn from(s: String) -> Self {
    PactSpecification::from(s.as_str())
  }
}

impl From<&String> for PactSpecification {
  fn from(s: &String) -> Self {
    PactSpecification::from(s.as_str())
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

impl PactJsonVerifier for Consumer {
  fn verify_json(path: &str, pact_json: &Value, strict: bool) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(name) = values.get("name") {
          if !name.is_string() {
            results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::ERROR,
              format!("Must be a String, got {}", json_type_of(pact_json))))
          }
        } else if strict {
          results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::ERROR, "Missing name"))
        } else {
          results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::WARNING, "Missing name"))
        }
      }
      _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
        format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
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

impl PactJsonVerifier for Provider {
  fn verify_json(path: &str, pact_json: &Value, strict: bool) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(values) => {
        if let Some(name) = values.get("name") {
          if !name.is_string() {
            results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::ERROR,
                                                         format!("Must be a String, got {}", json_type_of(pact_json))))
          }
        } else if strict {
          results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::ERROR, "Missing name"))
        } else {
          results.push(PactFileVerificationResult::new(path.to_owned() + "/name", ResultLevel::WARNING, "Missing name"))
        }
      }
      _ => results.push(PactFileVerificationResult::new(path, ResultLevel::ERROR,
                                                        format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

/// Enumeration of the types of differences between requests and responses
#[derive(PartialEq, Debug, Clone, Eq)]
pub enum DifferenceType {
  /// Methods differ
  Method,
  /// Paths differ
  Path,
  /// Headers differ
  Headers,
  /// Query parameters differ
  QueryParameters,
  /// Bodies differ
  Body,
  /// Matching Rules differ
  MatchingRules,
  /// Response status differ
  Status
}


/// Enum that defines the different types of HTTP statuses
#[derive(Debug, Clone, Deserialize, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub enum HttpStatus {
  /// Informational responses (100–199)
  Information,
  /// Successful responses (200–299)
  Success,
  /// Redirects (300–399)
  Redirect,
  /// Client errors (400–499)
  ClientError,
  /// Server errors (500–599)
  ServerError,
  /// Explicit status codes
  StatusCodes(Vec<u16>),
  /// Non-error response(< 400)
  NonError,
  /// Any error response (>= 400)
  Error
}

impl HttpStatus {
  /// Parse a JSON structure into a HttpStatus
  pub fn from_json(value: &Value) -> anyhow::Result<Self> {
    match value {
      Value::String(s) => match s.as_str() {
        "info" => Ok(HttpStatus::Information),
        "success" => Ok(HttpStatus::Success),
        "redirect" => Ok(HttpStatus::Redirect),
        "clientError" => Ok(HttpStatus::ClientError),
        "serverError" => Ok(HttpStatus::ServerError),
        "nonError" => Ok(HttpStatus::NonError),
        "error" => Ok(HttpStatus::Error),
        _ => Err(anyhow!("'{}' is not a valid value for an HTTP Status", s))
      },
      Value::Array(a) => {
        let status_codes = a.iter().map(|status| match status {
          Value::Number(n) => if n.is_u64() {
            Ok(n.as_u64().unwrap() as u16)
          } else if n.is_i64() {
            Ok(n.as_i64().unwrap() as u16)
          } else {
            Ok(n.as_f64().unwrap() as u16)
          },
          Value::String(s) => s.parse::<u16>().map_err(|err| anyhow!(err)),
          _ => Err(anyhow!("'{}' is not a valid JSON value for an HTTP Status", status))
        }).collect::<Vec<anyhow::Result<u16>>>();
        if status_codes.iter().any(|it| it.is_err()) {
          Err(anyhow!("'{}' is not a valid JSON value for an HTTP Status", value))
        } else {
          Ok(HttpStatus::StatusCodes(status_codes.iter().map(|code| *code.as_ref().unwrap()).collect()))
        }
      }
      _ => Err(anyhow!("'{}' is not a valid JSON value for an HTTP Status", value))
    }
  }

  /// Generate a JSON structure for this status
  pub fn to_json(&self) -> Value {
    match self {
      HttpStatus::StatusCodes(codes) => json!(codes),
      HttpStatus::Information => json!("info"),
      HttpStatus::Success => json!("success"),
      HttpStatus::Redirect => json!("redirect"),
      HttpStatus::ClientError => json!("clientError"),
      HttpStatus::ServerError => json!("serverError"),
      HttpStatus::NonError => json!("nonError"),
      HttpStatus::Error => json!("error")
    }
  }
}

impl Display for HttpStatus {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      HttpStatus::Information => write!(f, "Informational response (100–199)"),
      HttpStatus::Success => write!(f, "Successful response (200–299)"),
      HttpStatus::Redirect => write!(f, "Redirect (300–399)"),
      HttpStatus::ClientError => write!(f, "Client error (400–499)"),
      HttpStatus::ServerError => write!(f, "Server error (500–599)"),
      HttpStatus::StatusCodes(status) =>
        write!(f, "{}", status.iter().map(|s| s.to_string()).join(", ")),
      HttpStatus::NonError => write!(f, "Non-error response (< 400)"),
      HttpStatus::Error => write!(f, "Error response (>= 400)")
    }
  }
}

#[cfg(test)]
mod tests;
