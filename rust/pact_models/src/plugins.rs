//! Models to support plugins

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Plugin configuration persisted in the pact file metadata
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PluginData {
  /// Plugin name
  pub name: String,
  /// Plugin version
  pub version: String,
  /// Any configuration supplied by the plugin
  #[serde(default)]
  pub configuration: HashMap<String, Value>
}
