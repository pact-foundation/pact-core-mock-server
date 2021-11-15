//! FFI wrapper code for iterating over Pact interactions

use pact_models::message::Message;
use pact_models::message_pact::MessagePact;
use pact_models::v4::pact::V4Pact;
use pact_models::v4::sync_message::SynchronousMessage;
use pact_models::v4::V4InteractionType;

use crate::{as_mut, ffi_fn};
use crate::ptr;

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_message_iter_delete(iter: *mut PactMessageIterator) {
        ptr::drop_raw(iter);
    }
}

/// An iterator over messages in a pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactMessageIterator {
  current: usize,
  message_pact: MessagePact,
}

impl PactMessageIterator {
  /// Create a new iterator given a message pact
  pub fn new(message_pact: MessagePact) -> Self {
    PactMessageIterator {
      current: 0,
      message_pact
    }
  }

  /// Get the next message in the message pact.
  fn next(&mut self) -> Option<&mut Message> {
    let idx = self.current;
    self.current += 1;
    self.message_pact.messages.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next message from the message pact. As the messages returned are owned by the
    /// iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting a message returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_message_iter_next(iter: *mut PactMessageIterator) -> *mut Message {
        let iter = as_mut!(iter);
        let message = iter.next()
            .ok_or(anyhow::anyhow!("iter past the end of messages"))?;
        message as *mut Message
    } {
        ptr::null_mut_to::<Message>()
    }
}


/// An iterator over synchronous request/response messages in a pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct PactSyncMessageIterator {
  current: usize,
  messages: Vec<SynchronousMessage>
}

impl PactSyncMessageIterator {
  /// Create a new iterator over all synchronous request/response messages in the pact
  pub fn new(pact: V4Pact) -> Self {
    PactSyncMessageIterator {
      current: 0,
      messages: pact.filter_interactions(V4InteractionType::Synchronous_Messages)
        .iter()
        .map(|i| i.as_v4_sync_message().unwrap())
        .collect()
    }
  }

  /// Get the next message in the pact.
  fn next(&mut self) -> Option<&mut SynchronousMessage> {
    let idx = self.current;
    self.current += 1;
    self.messages.get_mut(idx)
  }
}

ffi_fn! {
    /// Get the next synchronous request/response message from the pact. As the messages returned are owned by the
    /// iterator, they do not need to be deleted but will be cleaned up when the iterator is
    /// deleted.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// Deleting a message returned by the iterator can lead to undefined behaviour.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn pactffi_pact_sync_message_iter_next(iter: *mut PactSyncMessageIterator) -> *mut SynchronousMessage {
        let iter = as_mut!(iter);
        let message = iter.next()
            .ok_or(anyhow::anyhow!("iter past the end of messages"))?;
        message as *mut SynchronousMessage
    } {
        ptr::null_mut_to::<SynchronousMessage>()
    }
}

ffi_fn! {
    /// Free the iterator when you're done using it.
    fn pactffi_pact_sync_message_iter_delete(iter: *mut PactSyncMessageIterator) {
        ptr::drop_raw(iter);
    }
}

