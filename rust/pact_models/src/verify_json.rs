//! Module for verifying the state of the Pact JSON (file format verification)

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Level of the result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PactFileVerificationResultLevel {
  /// A validation error
  Error,
  /// Validation warning
  WARNING,
  /// Just a note to display
  NOTICE
}

/// Single verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PactFileVerificationResult {
  /// Path into the JSON
  pub path: String,
  /// Level if the result
  pub level: PactFileVerificationResultLevel,
  /// Message associated with the result
  pub message: String
}

/// Trait for Pact JSON file format verifiers
pub trait PactJsonVerifier {
  /// Verify the JSON format. Will return an error if the list contains any Error result
  fn verify_json(pact_json: &Value) -> Vec<PactFileVerificationResult>;
}
