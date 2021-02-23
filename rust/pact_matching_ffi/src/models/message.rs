//! The Pact `Message` type, including associated matching rules and provider states.

/*===============================================================================================
 * # Imports
 *---------------------------------------------------------------------------------------------*/

use crate::models::pact_specification::PactSpecification;
use crate::util::*;
use crate::{as_mut, as_ref, cstr, ffi_fn, safe_str};
use anyhow::{anyhow, Context};
use libc::{c_char, c_int, c_uint, EXIT_FAILURE, EXIT_SUCCESS};
use pact_matching::models::{content_types::ContentType, OptionalBody};
use serde_json::from_str as from_json_str;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::ops::Drop;

/*===============================================================================================
 * # Re-Exports
 *---------------------------------------------------------------------------------------------*/

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message::Message;
pub use pact_matching::models::provider_states::ProviderState;

/*===============================================================================================
 * # Message
 *---------------------------------------------------------------------------------------------*/

/*-----------------------------------------------------------------------------------------------
 * ## Constructors / Destructor
 */

ffi_fn! {
    /// Get a mutable pointer to a newly-created default message on the heap.
    fn message_new() -> *mut Message {
        let message = Message::default();
        ptr::raw_to(message)
    } {
        ptr::null_mut_to::<Message>()
    }
}

ffi_fn! {
    /// Constructs a `Message` from the JSON string
    fn message_new_from_json(
        index: c_uint,
        json_str: *const c_char,
        spec_version: PactSpecification
    ) -> *mut Message {
        let message = {
            let index = index as usize;
            let json_value: JsonValue = from_json_str(safe_str!(json_str))
                .context("error parsing json_str as JSON")?;
            let spec_version = spec_version.into();

            Message::from_json(index, &json_value, &spec_version)
                .map_err(|e| anyhow::anyhow!("{}", e))?
        };

        ptr::raw_to(message)
    } {
        ptr::null_mut_to::<Message>()
    }
}

ffi_fn! {
    /// Constructs a `Message` from a body with a given content-type.
    fn message_new_from_body(body: *const c_char, content_type: *const c_char) -> *mut Message {
        // Get the body as a Vec<u8>.
        let body = cstr!(body)
            .to_bytes()
            .to_owned();

        // Parse the content type.
        let content_type = ContentType::parse(safe_str!(content_type))
            .map_err(|s| anyhow!("invalid content type '{}'", s))?;

        // Populate the Message metadata.
        let mut metadata = HashMap::new();
        metadata.insert(String::from("contentType"), content_type.to_string());

        // Populate the OptionalBody with our content and content type.
        let contents = OptionalBody::Present(body, Some(content_type));

        // Construct and return the message.
        let message = Message {
            contents,
            metadata,
            .. Message::default()
        };

        ptr::raw_to(message)
    } {
        ptr::null_mut_to::<Message>()
    }
}

ffi_fn! {
    /// Destroy the `Message` being pointed to.
    fn message_delete(message: *mut Message) {
        ptr::drop_raw(message);
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Contents
 */

ffi_fn! {
    /// Get the contents of a `Message`.
    fn message_get_contents(message: *const Message) -> *const c_char {
        let message = as_ref!(message);

        match message.contents {
            // If it's missing, return a null pointer.
            OptionalBody::Missing => ptr::null_to::<c_char>(),
            // If empty or null, return an empty string on the heap.
            OptionalBody::Empty | OptionalBody::Null => {
                let content = string::to_c("")?;
                content as *const c_char
            }
            // Otherwise, get the contents, possibly still empty.
            _ => {
                let content = string::to_c(message.contents.str_value())?;
                content as *const c_char
            }
        }
    } {
        ptr::null_to::<c_char>()
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Description
 */

ffi_fn! {
    /// Get a copy of the description.
    /// The returned string must be deleted with `string_delete`.
    ///
    /// Since it is a copy, the returned string may safely outlive
    /// the `Message`.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if the Rust string contains embedded
    /// null ('\0') bytes.
    fn message_get_description(message: *const Message) -> *const c_char {
        let message = as_ref!(message);
        let description = string::to_c(&message.description)?;
        description as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Write the `description` field on the `Message`.
    ///
    /// `description` must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    /// This function will only reallocate if the new string
    /// does not fit in the existing buffer.
    fn message_set_description(message: *mut Message, description: *const c_char) -> c_int {
        let message = as_mut!(message);
        let description = safe_str!(description);

        // Wipe out the previous contents of the string, without
        // deallocating, and set the new description.
        message.description.clear();
        message.description.push_str(description);

        EXIT_SUCCESS
    } {
        EXIT_FAILURE
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Provider States
 */

ffi_fn! {
    /// Get a copy of the provider state at the given index from this message.
    /// A pointer to the structure will be written to `out_provider_state`,
    /// only if no errors are encountered.
    ///
    /// The returned structure must be deleted with `provider_state_delete`.
    ///
    /// Since it is a copy, the returned structure may safely outlive
    /// the `Message`.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a variant other than Success.
    ///
    /// This function may fail if the index requested is out of bounds,
    /// or if any of the Rust strings contain embedded null ('\0') bytes.
    fn message_get_provider_state(message: *const Message, index: c_uint) -> *const ProviderState {
        let message = as_ref!(message);
        let index = index as usize;

        // Get a raw pointer directly, rather than boxing it, as its owned by the `Message`
        // and will be cleaned up when the `Message` is cleaned up.
        let provider_state = message
            .provider_states
            .get(index)
            .ok_or(anyhow!("index is out of bounds"))?;

        provider_state as *const ProviderState
    } {
        ptr::null_to::<ProviderState>()
    }
}

ffi_fn! {
    /// Get an iterator over provider states.
    fn message_get_provider_state_iter(message: *mut Message) -> *mut ProviderStateIterator {
        let message = as_mut!(message);
        let iter = ProviderStateIterator { current: 0, message };
        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<ProviderStateIterator>()
    }
}

ffi_fn! {
    /// Get the next value from the iterator.
    fn provider_state_iter_next(iter: *mut ProviderStateIterator) -> *mut ProviderState {
        let iter = as_mut!(iter);
        let message = as_mut!(iter.message);
        let index = iter.next();
        let provider_state = message
            .provider_states
            .get_mut(index)
            .ok_or(anyhow::anyhow!("iter past the end of provider states"))?;
       provider_state as *mut ProviderState
    } {
        ptr::null_mut_to::<ProviderState>()
    }
}

ffi_fn! {
    /// Delete the iterator.
    fn provider_state_iter_delete(iter: *mut ProviderStateIterator) {
        ptr::drop_raw(iter);
    }
}

/// Iterator over individual provider states.
#[allow(missing_copy_implementations)]
#[allow(missing_debug_implementations)]
pub struct ProviderStateIterator {
    current: usize,
    message: *mut Message,
}

impl ProviderStateIterator {
    fn next(&mut self) -> usize {
        let idx = self.current;
        self.current += 1;
        idx
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Metadata
 */

ffi_fn! {
    /// Get a copy of the metadata value indexed by `key`.
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
    /// This function may fail if the provided `key` string contains
    /// invalid UTF-8, or if the Rust string contains embedded null ('\0')
    /// bytes.
    fn message_find_metadata(message: *const Message, key: *const c_char) -> *const c_char {
        let message = as_ref!(message);
        let key = safe_str!(key);
        let value = message.metadata.get(key).ok_or(anyhow::anyhow!("invalid metadata key"))?;
        let value_ptr = string::to_c(value)?;
        value_ptr as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}

ffi_fn! {
    /// Insert the (`key`, `value`) pair into this Message's
    /// `metadata` HashMap.
    /// This function returns an enum indicating the result;
    /// see the comments on HashMapInsertStatus for details.
    ///
    /// # Errors
    ///
    /// This function may fail if the provided `key` or `value` strings
    /// contain invalid UTF-8.
    fn message_insert_metadata(
        message: *mut Message,
        key: *const c_char,
        value: *const c_char
    ) -> c_int {
        let message = as_mut!(message);
        let key = safe_str!(key);
        let value = safe_str!(value);

        match message.metadata.insert(key.to_string(), value.to_string()) {
            None => HashMapInsertStatus::SuccessNew as c_int,
            Some(_) => HashMapInsertStatus::SuccessOverwrite as c_int,
        }
    } {
        HashMapInsertStatus::Error as c_int
    }
}

ffi_fn! {
    /// Get an iterator over the metadata of a message.
    ///
    /// This iterator carries a pointer to the message, and must
    /// not outlive the message.
    ///
    /// The message metadata also must not be modified during iteration. If it is,
    /// the old iterator must be deleted and a new iterator created.
    ///
    /// # Errors
    ///
    /// On failure, this function will return a NULL pointer.
    ///
    /// This function may fail if any of the Rust strings contain
    /// embedded null ('\0') bytes.
    fn message_get_metadata_iter(message: *mut Message) -> *mut MessageMetadataIterator {
        let message = as_mut!(message);

        let iter = MessageMetadataIterator {
            keys:  message.metadata.keys().cloned().collect(),
            current: 0,
            message: message as *const Message,
        };

        ptr::raw_to(iter)
    } {
        ptr::null_mut_to::<MessageMetadataIterator>()
    }
}

ffi_fn! {
    /// Get the next key and value out of the iterator, if possible
    fn message_metadata_iter_next(iter: *mut MessageMetadataIterator) -> *mut MessageMetadataPair {
        let iter = as_mut!(iter);
        let message = as_ref!(iter.message);
        let key = iter.next().ok_or(anyhow::anyhow!("iter past the end of metadata"))?;
        let (key, value) = message
            .metadata
            .get_key_value(key)
            .ok_or(anyhow::anyhow!("iter provided invalid metadata key"))?;
        let pair = MessageMetadataPair::new(key, value)?;
        ptr::raw_to(pair)
    } {
        ptr::null_mut_to::<MessageMetadataPair>()
    }
}

ffi_fn! {
    /// Free the metadata iterator when you're done using it.
    fn message_metadata_iter_delete(iter: *mut MessageMetadataIterator) {
        ptr::drop_raw(iter);
    }
}

ffi_fn! {
    /// Free a pair of key and value returned from `message_metadata_iter_next`.
    fn message_metadata_pair_delete(pair: *mut MessageMetadataPair) {
        ptr::drop_raw(pair);
    }
}

/// An iterator that enables FFI iteration over metadata by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying metadata happens while the iterator is live.
#[derive(Debug)]
pub struct MessageMetadataIterator {
    /// The metadata keys
    keys: Vec<String>,
    /// The current key
    current: usize,
    /// Pointer to the message.
    message: *const Message,
}

impl MessageMetadataIterator {
    fn next(&mut self) -> Option<&String> {
        let idx = self.current;
        self.current += 1;
        self.keys.get(idx)
    }
}

/// A single key-value pair exported to the C-side.
#[derive(Debug)]
#[repr(C)]
#[allow(missing_copy_implementations)]
pub struct MessageMetadataPair {
    /// The metadata key.
    key: *const c_char,
    /// The metadata value.
    value: *const c_char,
}

impl MessageMetadataPair {
    fn new(
        key: &str,
        value: &str,
    ) -> anyhow::Result<MessageMetadataPair> {
        Ok(MessageMetadataPair {
            key: string::to_c(key)? as *const c_char,
            value: string::to_c(value)? as *const c_char,
        })
    }
}

// Ensure that the owned strings are freed when the pair is dropped.
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
impl Drop for MessageMetadataPair {
    fn drop(&mut self) {
        string::string_delete(self.key as *mut c_char);
        string::string_delete(self.value as *mut c_char);
    }
}

/*===============================================================================================
 * # Status Types
 *---------------------------------------------------------------------------------------------*/

/// Result from an attempt to insert into a HashMap
enum HashMapInsertStatus {
    /// The value was inserted, and the key was unset
    SuccessNew = 0,
    /// The value was inserted, and the key was previously set
    SuccessOverwrite = -1,
    /// An error occured, and the value was not inserted
    Error = -2,
}
