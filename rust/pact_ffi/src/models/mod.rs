//! FFI functions to support Pact models.

use std::collections::BTreeMap;
use std::sync::Mutex;

use libc::c_char;
use maplit::btreemap;
use serde_json::Value;
use tracing::{error, warn};

use pact_models::{Consumer, Provider};
use pact_models::plugins::PluginData;
use pact_models::v4::interaction::interactions_from_json;
use pact_models::v4::pact::V4Pact;

use crate::{ffi_fn, safe_str};
use crate::util::ptr;

pub mod consumer;
pub mod message;
pub mod message_pact;
pub mod pact_specification;
pub mod provider;
pub mod provider_state;
pub mod iterators;
pub mod sync_message;
pub mod http_interaction;
pub mod expressions;
pub mod matching_rules;
pub mod generators;

/// Opaque type for use as a pointer to a Pact model
pub struct Pact {
  inner: Mutex<V4Pact>
}

impl Pact {
  pub fn new(pact: V4Pact) -> Self {
    Pact {
      inner: Mutex::new(pact)
    }
  }
}

ffi_fn! {
  /// Parses the provided JSON into a Pact model. The returned Pact model must be freed with the
  /// `pactffi_pact_model_delete` function when no longer needed.
  ///
  /// # Error Handling
  ///
  /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
  fn pactffi_parse_pact_json(json: *const c_char) -> *mut Pact {
    let json_str = safe_str!(json);
    match serde_json::from_str::<Value>(json_str) {
      Ok(pact_json) => {
        // TODO: when pact_models 1.1.0 is released, update to this
        // let pact = V4Pact::pact_from_json(&pact_json, "<FFI>")?;
        let pact = pact_from_json(&pact_json, "<FFI>")?;
        ptr::raw_to(Pact::new(pact))
      },
      Err(err) => {
        error!("Failed to parse the Pact JSON - {}", err);
        ptr::null_mut_to::<Pact>()
      }
    }
  } {
    ptr::null_mut_to::<Pact>()
  }
}

ffi_fn! {
  /// Frees the memory used by the Pact model
  fn pactffi_pact_model_delete(pact: *mut Pact) {
    ptr::drop_raw(pact);
  }
}

// TODO: remove when pact_models 1.1.0 is released
fn pact_from_json(json: &Value, source: &str) -> anyhow::Result<V4Pact> {
  let mut metadata = meta_data_from_json(&json);

  let consumer = match json.get("consumer") {
    Some(v) => Consumer::from_json(v),
    None => Consumer { name: "consumer".into() }
  };
  let provider = match json.get("provider") {
    Some(v) => Provider::from_json(v),
    None => Provider { name: "provider".into() }
  };

  let plugin_data = extract_plugin_data(&mut metadata);

  Ok(V4Pact {
    consumer,
    provider,
    interactions: interactions_from_json(&json, source),
    metadata,
    plugin_data
  })
}

// TODO: remove when pact_models 1.1.0 is released
fn meta_data_from_json(pact_json: &Value) -> BTreeMap<String, Value> {
  match pact_json.get("metadata") {
    Some(Value::Object(ref obj)) => {
      obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
    _ => btreemap!{}
  }
}

// TODO: remove when pact_models 1.1.0 is released
fn extract_plugin_data(metadata: &mut BTreeMap<String, Value>) -> Vec<PluginData> {
  if let Some(plugin_data) = metadata.remove("plugins") {
    match plugin_data {
      Value::Array(items) => {
        let mut v = vec![];

        for item in &items {
          match serde_json::from_value::<PluginData>(item.clone()) {
            Ok(data) => v.push(data),
            Err(err) => warn!("Could not convert '{}' into PluginData format - {}", item, err)
          };
        }

        v
      }
      _ => {
        warn!("'{}' is not valid plugin data", plugin_data);
        vec![]
      }
    }
  } else {
    vec![]
  }
}

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;

  use crate::models::{pactffi_pact_model_delete, pactffi_parse_pact_json};

  #[test]
  fn load_pact_from_json() {
    let json = CString::new("{}").unwrap();
    let pact = pactffi_parse_pact_json(json.as_ptr());
    expect!(pact.is_null()).to(be_false());

    pactffi_pact_model_delete(pact);
  }
}
