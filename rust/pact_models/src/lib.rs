//! The `pact_models` crate provides all the structs and traits required to model a Pact.

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests;
