//! FFI functions to support Pact models.

use std::sync::Mutex;

use libc::c_char;
use pact_models::pact::load_pact_from_json;
use serde_json::Value;
use tracing::error;

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
#[derive(Debug)]
pub struct Pact {
  inner: Mutex<Box<dyn pact_models::pact::Pact + Send + Sync>>
}

impl Pact {
  /// Create a new FFI Pact wrapper for the given Pact model
  pub fn new(pact: Box<dyn pact_models::pact::Pact + Send + Sync>) -> Self {
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
        let pact = load_pact_from_json("<FFI>", &pact_json)?;
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

#[cfg(test)]
mod tests {
  use std::ffi::CString;

  use expectest::prelude::*;
  use libc::c_char;

  use crate::models::{pactffi_pact_model_delete, pactffi_parse_pact_json};
  use crate::models::consumer::{pactffi_consumer_get_name, pactffi_pact_consumer_delete, pactffi_pact_get_consumer};
  use crate::models::provider::{pactffi_pact_get_provider, pactffi_pact_provider_delete, pactffi_provider_get_name};

  #[test]
  fn load_pact_from_json() {
    let json = CString::new(r#"{
      "provider": {
        "name": "load_pact_from_json Provider"
      },
      "consumer": {
        "name": "load_pact_from_json Consumer"
      }
    }"#).unwrap();
    let pact = pactffi_parse_pact_json(json.as_ptr());
    expect!(pact.is_null()).to(be_false());

    let consumer = pactffi_pact_get_consumer(pact);
    let consumer_name_ptr = pactffi_consumer_get_name(consumer);
    let consumer_name = unsafe { CString::from_raw(consumer_name_ptr as *mut c_char) };

    let provider = pactffi_pact_get_provider(pact);
    let provider_name_ptr = pactffi_provider_get_name(provider);
    let provider_name = unsafe { CString::from_raw(provider_name_ptr as *mut c_char) };

    pactffi_pact_consumer_delete(consumer);
    pactffi_pact_provider_delete(provider);
    pactffi_pact_model_delete(pact);

    expect!(consumer_name.to_string_lossy()).to(be_equal_to("load_pact_from_json Consumer"));
    expect!(provider_name.to_string_lossy()).to(be_equal_to("load_pact_from_json Provider"));
  }
}
