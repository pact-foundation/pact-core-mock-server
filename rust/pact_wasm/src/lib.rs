//! WASM bindings for Pact models

use anyhow::anyhow;
use wasm_bindgen::prelude::*;
use log::debug;

use pact_models::pact::load_pact_from_json;

/// Struct for a Pact (request/response or message)
#[wasm_bindgen]
#[derive(Debug, Clone, Default)]
pub struct Pact {
  pact: Box<dyn pact_models::pact::Pact>,
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
  #[wasm_bindgen(getter)]
  pub fn specification_version(&self) -> String {
    debug!("{:?}", self.pact.specification_version());
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
    self.pact == other.pact.clone()
  }
}
