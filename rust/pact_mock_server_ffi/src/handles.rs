//! Handles wrapping Rust models

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::*;
use maplit::*;

use pact_matching::models::{Interaction, Pact, RequestResponseInteraction, RequestResponsePact};
use pact_models::{Consumer, Provider};

lazy_static! {
  static ref PACT_HANDLES: Mutex<HashMap<usize, RefCell<RequestResponsePact>>> = Mutex::new(hashmap![]);
}

#[repr(C)]
#[derive(Debug, Clone)]
/// Wraps a Pact model struct
pub struct PactHandle {
  /// Pact reference
  pub pact: usize
}

#[repr(C)]
#[derive(Debug, Clone)]
/// Wraps a Pact model struct
pub struct InteractionHandle {
  /// Pact reference
  pub pact: usize,
  /// Interaction reference
  pub interaction: usize
}

#[repr(C)]
#[derive(Debug, Clone)]
/// Request or Response enum
pub enum InteractionPart {
  /// Request part
  Request,
  /// Response part
  Response
}

impl PactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let id = handles.len() + 1;
    handles.insert(id, RefCell::new(RequestResponsePact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      .. RequestResponsePact::default()
    }));
    PactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner Pact model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut RequestResponsePact) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }
}

impl InteractionHandle {
  /// Creates a new handle to an Interaction
  pub fn new(pact: PactHandle, interaction: usize) -> InteractionHandle {
    InteractionHandle {
      pact: pact.pact,
      interaction
    }
  }

  /// Invokes the closure with the inner Pact model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut RequestResponsePact) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_interaction<R>(&self, f: &dyn Fn(usize, &mut RequestResponseInteraction) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      match inner.borrow_mut().interactions.get_mut(self.interaction - 1) {
        Some(inner_i) => Some(f(self.interaction - 1, inner_i)),
        None => None
      }
    }).flatten()
  }
}
