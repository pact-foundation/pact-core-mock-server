//! Handles wrapping Rust models

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::*;
use maplit::*;

use pact_models::{Consumer, Provider};
use pact_models::message::Message;
use pact_models::message_pact::MessagePact;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;

#[derive(Debug, Clone)]
/// Pact handle inner struct
pub struct PactHandleInner {
  pub(crate) pact: RequestResponsePact,
  pub(crate) mock_server_started: bool
}

lazy_static! {
  static ref PACT_HANDLES: Mutex<HashMap<usize, RefCell<PactHandleInner>>> = Mutex::new(hashmap![]);
  static ref MESSAGE_PACT_HANDLES: Mutex<HashMap<usize, RefCell<MessagePact>>> = Mutex::new(hashmap![]);
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
    handles.insert(id, RefCell::new(PactHandleInner {
      pact: RequestResponsePact {
        consumer: Consumer { name: consumer.to_string() },
        provider: Provider { name: provider.to_string() },
        .. RequestResponsePact::default()
      },
      mock_server_started: false
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
  pub fn with_interaction<R>(&self, f: &dyn Fn(usize, bool, &mut RequestResponseInteraction) -> R) -> Option<R> {
    let mut handles = PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      let inner_mut = &mut *inner.borrow_mut();
      let interactions = &mut inner_mut.pact.interactions;
      match interactions.get_mut(self.interaction - 1) {
        Some(inner_i) => Some(f(self.interaction - 1, inner_mut.mock_server_started, inner_i)),
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
    let mut handles = MESSAGE_PACT_HANDLES.lock().unwrap();
    let id = handles.len() + 1;
    handles.insert(id, RefCell::new(MessagePact {
      consumer: Consumer { name: consumer.to_string() },
      provider: Provider { name: provider.to_string() },
      .. MessagePact::default()
    }));
    MessagePactHandle {
      pact: id
    }
  }

  /// Invokes the closure with the inner MessagePact model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut MessagePact) -> R) -> Option<R> {
    let mut handles = MESSAGE_PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
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

  /// Invokes the closure with the inner MessagePact model
  pub fn with_pact<R>(&self, f: &dyn Fn(usize, &mut MessagePact) -> R) -> Option<R> {
    let mut handles = MESSAGE_PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| f(self.pact - 1, &mut inner.borrow_mut()))
  }

  /// Invokes the closure with the inner Interaction model
  pub fn with_message<R>(&self, f: &dyn Fn(usize, &mut Message) -> R) -> Option<R> {
    let mut handles = MESSAGE_PACT_HANDLES.lock().unwrap();
    handles.get_mut(&self.pact).map(|inner| {
      inner.borrow_mut().messages.get_mut(self.message - 1)
        .map(|inner_i| f(self.message - 1, inner_i))
    }).flatten()
  }
}
