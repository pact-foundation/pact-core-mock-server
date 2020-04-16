//! Handles wrapping Rust models

use pact_matching::models::Pact;
use lazy_static::*;
use std::sync::Mutex;

lazy_static! {
  static ref PACT_HANDLES: Mutex<Vec<Box<Pact>>> = Mutex::new(vec![]);
}

#[repr(C)]
#[derive(Debug, Clone)]
/// Wraps a Pact model struct
pub struct PactHandle {
  pact: usize
}

impl PactHandle {
  /// Creates a new handle to a Pact model
  pub fn new() -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.push(Box::new(Pact::default()));
    PactHandle {
      pact: handles.len() + 1
    }
  }
}
