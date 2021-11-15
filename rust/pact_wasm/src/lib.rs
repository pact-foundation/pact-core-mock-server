//! WASM bindings for Pact models

use std::rc::Rc;

use anyhow::anyhow;
use log::debug;
use pact_models::pact::load_pact_from_json;
use pact_models::plugins::PluginData;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::synch_http::SynchronousHttp;
use wasm_bindgen::prelude::*;

/// Library version
#[wasm_bindgen(js_name = libVersion)]
pub fn lib_version() -> String {
  option_env!("CARGO_PKG_VERSION").unwrap_or_default().to_string()
}

/// Struct for a Pact Interaction (request/response or message)
#[wasm_bindgen]
#[derive(Debug)]
pub struct PactInteraction {
  interaction: Box<dyn pact_models::interaction::Interaction + Send + Sync>
}

#[wasm_bindgen]
impl PactInteraction {
  /// Interaction key
  #[wasm_bindgen(getter)]
  pub fn key(&self) -> String {
    self.interaction.as_v4().map(|i| i.key().unwrap_or_default())
      .unwrap_or_default()
  }

  /// Description
  #[wasm_bindgen(getter)]
  pub fn description(&self) -> String {
    self.interaction.description().clone()
  }

  /// Interaction Type
  #[wasm_bindgen(getter, js_name=type)]
  pub fn interaction_type(&self) -> String {
    if let Some(v4) = self.interaction.as_v4() {
      v4.type_of()
    } else if self.interaction.is_message() {
      "Message".to_string()
    } else {
      "HTTP".to_string()
    }
  }

  /// Interaction Markup
  #[wasm_bindgen(getter, js_name=markup)]
  pub fn interaction_markup(&self) -> Option<String> {
    if let Some(v4) = self.interaction.as_v4() {
      Some(v4.interaction_markup().markup)
    } else {
      None
    }
  }

  /// Interaction Markup Type
  #[wasm_bindgen(getter, js_name=markupType)]
  pub fn interaction_markup_type(&self) -> Option<String> {
    if let Some(v4) = self.interaction.as_v4() {
      Some(v4.interaction_markup().markup_type)
    } else {
      None
    }
  }
}

impl PartialEq for PactInteraction {
  fn eq(&self, other: &Self) -> bool {
    if let Some(interaction) = self.interaction.as_v4() {
      if let Some(ref other) = other.interaction.as_v4() {
        interaction.eq(other)
      } else {
        false
      }
    } else if let Some(interaction) = self.interaction.as_request_response() {
      if let Some(ref other) = other.interaction.as_request_response() {
        interaction.eq(other)
      } else {
        false
      }
    } else if let Some(interaction) = self.interaction.as_message() {
      if let Some(ref other) = other.interaction.as_message() {
        interaction.eq(other)
      } else {
        false
      }
    } else {
      false
    }
  }
}

impl Clone for PactInteraction {
  fn clone(&self) -> Self {
    PactInteraction { interaction: self.interaction.boxed() }
  }
}

impl Default for PactInteraction {
  fn default() -> Self {
    PactInteraction { interaction: pact_models::interaction::Interaction::boxed(&SynchronousHttp::default()) }
  }
}

/// Details of plugins used with the Pact
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct PactPlugin {
  plugin_data: Rc<PluginData>
}

impl PactPlugin {
  pub fn new(plugin_data: &PluginData) -> Self {
    PactPlugin {
      plugin_data: Rc::new(plugin_data.clone())
    }
  }
}

#[wasm_bindgen]
impl PactPlugin {
  /// Plugin name
  #[wasm_bindgen(getter)]
  pub fn name(&self) -> String {
    self.plugin_data.name.clone()
  }

  /// Plugin version
  #[wasm_bindgen(getter)]
  pub fn version(&self) -> String {
    self.plugin_data.version.clone()
  }
}

/// Struct for a Pact (request/response or message)
#[wasm_bindgen]
#[derive(Debug)]
pub struct Pact {
  pact: Box<dyn pact_models::pact::Pact + Send + Sync>,
}

#[wasm_bindgen]
impl Pact {
  /// Consumer side of the pact
  #[wasm_bindgen(getter)]
  pub fn consumer(&self) -> String {
    self.pact.consumer().name
  }

  /// Provider side of the pact
  #[wasm_bindgen(getter)]
  pub fn provider(&self) -> String {
    self.pact.provider().name
  }

  /// Provider side of the pact
  #[wasm_bindgen(getter, js_name = specificationVersion)]
  pub fn specification_version(&self) -> String {
    self.pact.specification_version().to_string()
  }

  /// Parse a Pact from JSON
  pub fn parse(json: String) -> Result<Pact, JsValue> {
    match serde_json::from_str(&json) {
      Ok(json) => load_pact_from_json("<JSON>", &json).map(|pact| Pact { pact }),
      Err(err) => Err(anyhow!("Failed to parse Pact JSON: {}", err))
    }.map_err(|err| {
      JsValue::from(err.to_string())
    })
  }

  /// Interactions in the Pact
  #[wasm_bindgen(js_name = numInteractions)]
  pub fn num_interactions(&self) -> usize {
    self.pact.interactions().len()
  }

  /// Interactions in the Pact
  #[wasm_bindgen]
  pub fn interaction(&self, index: usize) -> Option<PactInteraction> {
    self.pact.interactions().get(index).map(|interaction| {
      PactInteraction { interaction: interaction.boxed() }
    })
  }

  /// Plugin data in the pact
  #[wasm_bindgen(js_name = hasPlugins)]
  pub fn has_plugins(&self) -> bool {
    !self.pact.plugin_data().is_empty()
  }

  /// Plugin data in the pact
  #[wasm_bindgen(js_name = numPlugins)]
  pub fn num_plugins(&self) -> usize {
    self.pact.plugin_data().len()
  }

  /// Plugin data in the pact
  #[wasm_bindgen]
  pub fn plugin(&self, index: usize) -> Option<PactPlugin> {
    self.pact.plugin_data().get(index).map(|plugin| {
      PactPlugin::new(plugin)
    })
  }
}

impl PartialEq for Pact {
  fn eq(&self, other: &Self) -> bool {
    if let Ok(pact) = self.pact.as_v4_pact() {
      if let Ok(ref other) = other.pact.as_v4_pact() {
        pact.eq(other)
      } else {
        false
      }
    } else if let Ok(pact) = self.pact.as_request_response_pact() {
      if let Ok(ref other) = other.pact.as_request_response_pact() {
        pact.eq(other)
      } else {
        false
      }
    } else if let Ok(pact) = self.pact.as_message_pact() {
      if let Ok(ref other) = other.pact.as_message_pact() {
        pact.eq(other)
      } else {
        false
      }
    } else {
      false
    }
  }
}

impl Clone for Pact {
  fn clone(&self) -> Self {
    Pact { pact: self.pact.boxed() }
  }
}

impl Default for Pact {
  fn default() -> Self {
    Pact { pact: pact_models::pact::Pact::boxed(&V4Pact::default()) }
  }
}
