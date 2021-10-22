//! WASM bindings for Pact models

use anyhow::anyhow;
use log::debug;
use pact_models::pact::load_pact_from_json;
use pact_models::prelude::v4::V4Pact;
use wasm_bindgen::prelude::*;

/// Library version
#[wasm_bindgen(js_name = libVersion)]
pub fn lib_version() -> String {
  option_env!("CARGO_PKG_VERSION").unwrap_or_default().to_string()
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
