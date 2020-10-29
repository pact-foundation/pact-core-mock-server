//! FFI wrapper for `MessagePact` from pact_matching.

use crate::util::*;
use crate::{as_mut, as_ref, cstr, ffi_fn, safe_str};
use anyhow::{anyhow, Context};
use libc::c_char;

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message_pact::MessagePact;
use pact_matching::models::Consumer;
use pact_matching::models::Provider;

ffi_fn! {
    /// Construct a new `MessagePact` from the JSON string.
    /// The provided file name is used when generating error messages.
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
    /// # Errors
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
    /// # Errors
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
    /// Get a copy of the metadata value indexed by `key1` and `key2`.
    /// The returned string must be deleted with `string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `Message`.
    ///
    /// The returned pointer will be NULL if the metadata does not contain
    /// the given key, or if an error occurred.
    ///
    /// # Errors
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
