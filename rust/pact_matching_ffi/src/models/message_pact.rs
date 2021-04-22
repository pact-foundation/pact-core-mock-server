//! FFI wrapper for `MessagePact` from pact_matching.

use crate::util::*;
use crate::{as_mut, as_ref, ffi_fn, safe_str};
use anyhow::{anyhow, Context};
use libc::c_char;
use std::iter::{self, Iterator};

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
use crate::models::message::Message;
pub use pact_matching::models::message_pact::MessagePact;
use pact_matching::models::Consumer;
use pact_matching::models::Provider;

ffi_fn! {
    /// Construct a new `MessagePact` from the JSON string.
    /// The provided file name is used when generating error messages.
    ///
    /// # Safety
    ///
    /// The `file_name` and `json_str` parameters must both be valid UTF-8
    /// encoded strings.
    ///
    /// # Error Handling
    ///
    /// On error, this function will return a null pointer.
    fn message_pact_new_from_json(
        file_name: *const c_char,
        json_str: *const c_char
    ) -> *mut MessagePact {
        let file_name = safe_str!(file_name);
        let json_str = safe_str!(json_str);

        let json_value: serde_json::Value =
            serde_json::from_str(json_str)
            .context("error parsing json_str as JSON")?;

        let message_pact = MessagePact::from_json(
            &file_name.to_string(),
            &json_value,
        ).map_err(|e| anyhow!("{}", e))?;

        ptr::raw_to(message_pact)
    } {
        ptr::null_mut_to::<MessagePact>()
    }
}

ffi_fn! {
    /// Delete the `MessagePact` being pointed to.
    fn message_pact_delete(message_pact: *mut MessagePact) {
        ptr::drop_raw(message_pact);
    }
}

ffi_fn! {
    /// Get a pointer to the Consumer struct inside the MessagePact.
    /// This is a mutable borrow: The caller may mutate the Consumer
    /// through this pointer.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// This function will only fail if it is passed a NULL pointer.
    /// In the case of error, a NULL pointer will be returned.
    fn message_pact_get_consumer(message_pact: *mut MessagePact) -> *mut Consumer {
        let message_pact = as_mut!(message_pact);
        let consumer = &mut message_pact.consumer;
        consumer as *mut Consumer
    } {
        ptr::null_mut_to::<Consumer>()
    }
}

ffi_fn! {
    /// Get a pointer to the Provider struct inside the MessagePact.
    /// This is a mutable borrow: The caller may mutate the Provider
    /// through this pointer.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// This function will only fail if it is passed a NULL pointer.
    /// In the case of error, a NULL pointer will be returned.
    fn message_pact_get_provider(message_pact: *mut MessagePact) -> *mut Provider {
        let message_pact = as_mut!(message_pact);
        let provider = &mut message_pact.provider;
        provider as *mut Provider
    } {
        ptr::null_mut_to::<Provider>()
    }
}

ffi_fn! {
    /// Get an iterator over the messages of a message pact.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the message pact, and must
    /// not outlive the message pact.
    ///
    /// The message pact messages also must not be modified during iteration.
    /// If they are, the old iterator must be deleted and a new iterator created.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain embedded
    /// null ('\0') bytes.
    fn message_pact_get_message_iter(message_pact: *mut MessagePact) -> *mut MessagePactMessageIterator {
        let message_pact = as_mut!(message_pact);
        let iter = MessagePactMessageIterator { current: 0, message_pact };
        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<MessagePactMessageIterator>()
    }
}

ffi_fn! {
    /// Get the next message from the message pact.
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// This function will return a NULL pointer if passed a NULL pointer or if an error occurs.
    fn message_pact_message_iter_next(iter: *mut MessagePactMessageIterator) -> *mut Message {
        let iter = as_mut!(iter);
        let message_pact = as_mut!(iter.message_pact);
        let index = iter.next();
        let message = message_pact
            .messages
            .get_mut(index)
            .ok_or(anyhow::anyhow!("iter past the end of messages"))?;
        message as *mut Message
    } {
        ptr::null_mut_to::<Message>()
    }
}

ffi_fn! {
    /// Delete the iterator.
    fn message_pact_message_iter_delete(iter: *mut MessagePactMessageIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Get a copy of the metadata value indexed by `key1` and `key2`.
    ///
    /// # Safety
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `Message`.
    ///
    /// The returned string must be deleted with `string_delete`.
    ///
    /// The returned pointer will be NULL if the metadata does not contain
    /// the given key, or if an error occurred.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the provided `key1` or `key2` strings contains
    /// invalid UTF-8, or if the Rust string contains embedded null ('\0')
    /// bytes.
    fn message_pact_find_metadata(message_pact: *const MessagePact, key1: *const c_char, key2: *const c_char) -> *const c_char {
        let message_pact = as_ref!(message_pact);
        let key1 = safe_str!(key1);
        let key2 = safe_str!(key2);
        let metadata = message_pact.metadata.get(key1).ok_or(anyhow::anyhow!("invalid metadata key (key 1)"))?;
        let value = metadata.get(key2).ok_or(anyhow::anyhow!("invalid metadata key (key 2)"))?;
        let value_ptr = string::to_c(value)?;
        value_ptr as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Get an iterator over the metadata of a message pact.
    ///
    /// # Safety
    ///
    /// This iterator carries a pointer to the message pact, and must
    /// not outlive the message pact.
    ///
    /// The message pact metadata also must not be modified during iteration. If it is,
    /// the old iterator must be deleted and a new iterator created.
    ///
    /// # Error Handling
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain
    /// embedded null ('\0') bytes.
    fn message_pact_get_metadata_iter(message_pact: *mut MessagePact) -> *mut MessagePactMetadataIterator {
        let message_pact = as_mut!(message_pact);

        let keys = message_pact
            .metadata
            .iter()
            .flat_map(|(outer_key, sub_tree)| {
                let outer_key_repeater = iter::repeat(outer_key.clone());
                let inner_keys = sub_tree.keys().cloned();

                Iterator::zip(outer_key_repeater, inner_keys)
            })
            .collect();

        let iter = MessagePactMetadataIterator {
            keys,
            current: 0,
            message_pact: message_pact as *const MessagePact,
        };

        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<MessagePactMetadataIterator>()
    }
}

ffi_fn! {
    /// Get the next triple out of the iterator, if possible
    ///
    /// # Safety
    ///
    /// This operation is invalid if the underlying data has been changed during iteration.
    ///
    /// # Error Handling
    ///
    /// Returns null if no next element is present.
    fn message_pact_metadata_iter_next(iter: *mut MessagePactMetadataIterator) -> *mut MessagePactMetadataTriple {
        let iter = as_mut!(iter);
        let message_pact = as_ref!(iter.message_pact);
        let (outer_key, inner_key) = iter.next().ok_or(anyhow::anyhow!("iter past the end of metadata"))?;

        let (outer_key, sub_tree) = message_pact
            .metadata
            .get_key_value(outer_key)
            .ok_or(anyhow::anyhow!("iter provided invalid metadata key"))?;

        let (inner_key, value) = sub_tree
            .get_key_value(inner_key)
            .ok_or(anyhow::anyhow!("iter provided invalid metadata key"))?;

        let triple = MessagePactMetadataTriple::new(outer_key, inner_key, value)?;

        ptr::raw_to(triple)
    } {
        ptr::null_mut_to::<MessagePactMetadataTriple>()
    }
}

ffi_fn! {
    /// Free the metadata iterator when you're done using it.
    fn message_pact_metadata_iter_delete(iter: *mut MessagePactMetadataIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Free a triple returned from `message_pact_metadata_iter_next`.
    fn message_pact_metadata_triple_delete(triple: *mut MessagePactMetadataTriple) {
        ptr::drop_raw(triple);
    }
}

/// An iterator over messages in a message pact.
#[derive(Debug)]
#[allow(missing_copy_implementations)]
pub struct MessagePactMessageIterator {
    current: usize,
    message_pact: *mut MessagePact,
}

impl MessagePactMessageIterator {
    /// Get the index of the next message in the message pact.
    fn next(&mut self) -> usize {
        let idx = self.current;
        self.current += 1;
        idx
    }
}

/// An iterator that enables FFI iteration over metadata by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying metadata happens while the iterator is live.
#[derive(Debug)]
pub struct MessagePactMetadataIterator {
    /// The metadata keys
    keys: Vec<(String, String)>,
    /// The current key
    current: usize,
    /// Pointer to the message pact.
    message_pact: *const MessagePact,
}

impl MessagePactMetadataIterator {
    fn next(&mut self) -> Option<(&str, &str)> {
        let idx = self.current;
        self.current += 1;
        self.keys.get(idx).map(|(outer_key, inner_key)| {
            (outer_key.as_ref(), inner_key.as_ref())
        })
    }
}

/// A triple, containing the outer key, inner key, and value, exported to the C-side.
#[derive(Debug)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct MessagePactMetadataTriple {
    /// The outer key of the `MessagePact` metadata.
    outer_key: *const c_char,
    /// The inner key of the `MessagePact` metadata.
    inner_key: *const c_char,
    /// The value of the `MessagePact` metadata.
    value: *const c_char,
}

impl MessagePactMetadataTriple {
    fn new(
        outer_key: &str,
        inner_key: &str,
        value: &str,
    ) -> anyhow::Result<MessagePactMetadataTriple> {
        // This constructor means each of these strings is an owned string.
        Ok(MessagePactMetadataTriple {
            outer_key: string::to_c(outer_key)? as *const c_char,
            inner_key: string::to_c(inner_key)? as *const c_char,
            value: string::to_c(value)? as *const c_char,
        })
    }
}

// Ensure that the owned strings are freed when the triple is dropped.
//
// Notice that we're casting from a `*const c_char` to a `*mut c_char`.
// This may seem wrong, but is safe so long as it doesn't violate Rust's
// guarantees around immutable references, which this doesn't. In this case,
// the underlying data came from `CString::into_raw` which takes ownership
// of the `CString` and hands it off via a `*mut pointer`. We cast that pointer
// back to `*const` to limit the C-side from doing any shenanigans, since the
// pointed-to values live inside of the `Message` metadata `HashMap`, but
// cast back to `*mut` here so we can free the memory.
//
// The discussion here helps explain: https://github.com/rust-lang/rust-clippy/issues/4774
impl Drop for MessagePactMetadataTriple {
    fn drop(&mut self) {
        string::string_delete(self.outer_key as *mut c_char);
        string::string_delete(self.inner_key as *mut c_char);
        string::string_delete(self.value as *mut c_char);
    }
}
