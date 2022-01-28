//! Structs for storing and returning the result of the verification execution

use std::collections::HashMap;
use pact_matching::Mismatch;

/// Main struct for returning the verification execution result
#[derive(Debug, Clone)]
pub struct VerificationExecutionResult {
  /// Overall pass/fail result
  pub result: bool,
  /// Notices provided by the Pact Broker
  pub notices: Vec<HashMap<String, String>>,
  /// Collected standard output
  pub output: Vec<String>,
  /// Errors that occurred, but are marked as pending
  pub pending_errors: Vec<(String, MismatchResult)>,
  /// Errors that occurred that are not considered pending
  pub errors: Vec<(String, MismatchResult)>,
}

impl VerificationExecutionResult {
  /// Create a new VerificationExecutionResult with default values
  pub fn new() -> Self {
    VerificationExecutionResult {
      result: true,
      notices: vec![],
      output: vec![],
      pending_errors: vec![],
      errors: vec![]
    }
  }
}

/// Result of performing a match. This is a reduced version of super::MismatchResult to make
/// it thread and panic boundary safe
#[derive(Debug, Clone)]
pub enum MismatchResult {
  /// Response mismatches
  Mismatches {
    /// Mismatches that occurred
    mismatches: Vec<Mismatch>,
    /// Interaction ID if fetched from a pact broker
    interaction_id: Option<String>
  },
  /// Error occurred
  Error {
    /// Error that occurred
    error: String,
    /// Interaction ID if fetched from a pact broker
    interaction_id: Option<String>
  }
}

impl From<&crate::MismatchResult> for MismatchResult {
  fn from(result: &crate::MismatchResult) -> Self {
    match result {
      crate::MismatchResult::Mismatches { mismatches, interaction_id, .. } => {
        MismatchResult::Mismatches {
          mismatches: mismatches.clone(),
          interaction_id: interaction_id.clone()
        }
      }
      crate::MismatchResult::Error(error, interaction_id) => {
        MismatchResult::Error {
          error: error.clone(),
          interaction_id: interaction_id.clone()
        }
      }
    }
  }
}
