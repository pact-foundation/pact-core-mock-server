//! Structs for shared parts of message interactions

use pact_models::bodies::OptionalBody;
use std::collections::HashMap;
use serde_json::Value;
use crate::models::{matchingrules, generators};

/// Contents of a message interaction
#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct MessageContents {
  /// The contents of the message
  pub contents: OptionalBody,
  /// Metadata associated with this message.
  pub metadata: HashMap<String, Value>,
  /// Matching rules
  pub matching_rules: matchingrules::MatchingRules,
  /// Generators
  pub generators: generators::Generators,
}
