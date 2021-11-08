//! Handles wrapping Rust models

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::*;
use maplit::*;
use log::*;

use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::prelude::Pact;
use pact_models::prelude::v4::V4Pact;
use pact_models::v4::interaction::V4Interaction;

#[derive(Debug, Clone)]
/// Pact handle inner struct
pub struct PactHandleInner {
  pub(crate) pact: V4Pact,
  pub(crate) mock_server_started: bool,
  pub(crate) specification_version: PactSpecification
}

lazy_static! {
  static ref PACT_HANDLES: Mutex<HashMap<usize, RefCell<PactHandleInner>>> = Mutex::new(hashmap![]);
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct PactHandle {
  /// Pact reference
  pub pact: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct InteractionHandle {
  /// Pact reference
  pub pact: usize,
  /// Interaction reference
  pub interaction: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
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
    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    PactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner Pact model
  pub(crate) fn with_pact<R>(&self, f: &dyn Fn(usize, &mut PactHandleInner) -> R) -> Option<R> {
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
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut PactHandleInner) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_interaction<R>(&self, f: &dyn Fn(usize, bool, &mut dyn V4Interaction) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let inner_mut = &mut *inner.borrow_mut();
      let interactions = &mut inner_mut.pact.interactions;
      match interactions.get_mut(self.interaction - 1) {
        Some(inner_i) => {
          Some(f(self.interaction - 1, inner_mut.mock_server_started, inner_i.as_mut()))
        },
        None => None
      }
    }).flatten()
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessagePactHandle {
  /// Pact reference
  pub pact: usize
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Wraps a Pact model struct
pub struct MessageHandle {
  /// Message Pact reference
  pub pact: usize,
  /// Interaction reference
  pub message: usize
}

impl MessagePactHandle {
  /// Creates a new handle to a Pact model
  pub fn new(consumer: &str, provider: &str) -> Self {
    let mut handles = PACT_HANDLES.lock().unwrap();
    let id = handles.len() + 1;
    let mut pact = V4Pact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      ..V4Pact::default()
    };
    pact.add_md_version("ffi", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    handles.insert(id, RefCell::new(PactHandleInner {
      pact,
      mock_server_started: false,
      specification_version: PactSpecification::V3
    }));
    MessagePactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(self.pact - 1, &mut ref_mut.pact, specification)
    })
  }
}

impl MessageHandle {
  /// Creates a new handle to a message
  pub fn new(pact: MessagePactHandle, message: usize) -> MessageHandle {
    MessageHandle {
      pact: pact.pact,
      message
    }
  }

  /// Invokes the closure with the inner model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut V4Pact, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      f(self.pact - 1, & mut ref_mut.pact, specification)
    })
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_message<R>(&self, f: &dyn Fn(usize, &mut dyn V4Interaction, PactSpecification) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let mut ref_mut = inner.borrow_mut();
      let specification = ref_mut.specification_version;
      ref_mut.pact.interactions.get_mut(self.message - 1)
        .map(|inner_i| {
          if inner_i.is_message() {
            Some(f(self.message - 1, inner_i.as_mut(), specification))
          } else {
            error!("Interaction {} is not a message interaction, it is {}", self.message, inner_i.type_of());
            None
          }
        }).flatten()
    }).flatten()
  }
}
