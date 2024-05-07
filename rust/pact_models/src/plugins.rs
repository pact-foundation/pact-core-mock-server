//! Models to support plugins

use std::collections::HashMap;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::json_utils::json_deep_merge;

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

impl PluginData {
  /// Deep merges the data with any existing data
  pub fn merge(&mut self, data: &HashMap<String, Value>) {
    for (key, value) in data {
      let value = if let Some(v) = self.configuration.get(key) {
        json_deep_merge(v, value)
      } else {
        value.clone()
      };
      self.configuration.insert(key.clone(), value);
    }
  }
}

impl PluginData {
  /// Convert this plugin data to a JSON value
  pub fn to_json(&self) -> anyhow::Result<Value> {
    serde_json::to_value(self)
      .map_err(|err| anyhow!("Could not convert plugin data to JSON - {}", err))
  }
}
