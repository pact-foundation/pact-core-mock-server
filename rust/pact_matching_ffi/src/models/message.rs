//! The Pact `Message` type, including associated matching rules and provider states.

/*===============================================================================================
 * # Imports
 *---------------------------------------------------------------------------------------------*/

use crate::models::pact_specification::PactSpecification;
use crate::util::*;
use crate::{as_mut, as_ref, cstr, ffi, safe_str};
use anyhow::{anyhow, Context};
use libc::{c_char, c_int, c_uint, EXIT_FAILURE, EXIT_SUCCESS};
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

/// Get a mutable pointer to a newly-created default message on the heap.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_new() -> *mut Message {
    ffi! {
        name: "message_new",
        params: [],
        op: {
            Ok(ptr::raw_to(Message::default()))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}

/// Constructs a `Message` from the JSON string
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_new_from_json(
    index: c_uint,
    json_str: *const c_char,
    spec_version: PactSpecification,
) -> *mut Message {
    ffi! {
        name: "message_new_from_json",
        params: [index, json_str, spec_version],
        op: {
            let json_str = safe_str!(json_str);

            let json_value: serde_json::Value =
                serde_json::from_str(json_str)
                .context("error parsing json_str as JSON")?;

            let message = Message::from_json(
                index as usize,
                &json_value,
                &spec_version.into())
                .map_err(|e| anyhow::anyhow!("{}", e))?;

            Ok(ptr::raw_to(message))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}

/// Destroy the `Message` being pointed to.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_delete(message: *mut Message) -> c_int {
    ffi! {
        name: "message_delete",
        params: [message],
        op: {
            ptr::drop_raw(message);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Description
 */

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
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_get_description(
    message: *const Message,
) -> *const c_char {
    ffi! {
        name: "message_get_description",
        params: [message],
        op: {
            let message = as_ref!(message);
            Ok(string::to_c(&message.description)? as *const c_char)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}

/// Write the `description` field on the `Message`.
///
/// `description` must contain valid UTF-8. Invalid UTF-8
/// will be replaced with U+FFFD REPLACEMENT CHARACTER.
///
/// This function will only reallocate if the new string
/// does not fit in the existing buffer.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_set_description(
    message: *mut Message,
    description: *const c_char,
) -> c_int {
    ffi! {
        name: "message_set_description",
        params: [message, description],
        op: {
            let message = as_mut!(message);
            let description = safe_str!(description);

            // Wipe out the previous contents of the string, without
            // deallocating, and set the new description.
            message.description.clear();
            message.description.push_str(description);

            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/*-----------------------------------------------------------------------------------------------
 * ## Provider States
 */

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
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_get_provider_state(
    message: *const Message,
    index: usize,
) -> *const ProviderState {
    ffi! {
        name: "message_get_provider_state",
        params: [message, index],
        op: {
            let message = as_ref!(message);
            // Get a raw pointer directly, rather than boxing it, as its owned by the `Message`
            // and will be cleaned up when the `Message` is cleaned up.
            let provider_state = message
                .provider_states
                .get(index)
                .ok_or(anyhow!("index is out of bounds"))?
                as *const ProviderState;
            Ok(provider_state)
        },
        fail: {
            ptr::null_to::<ProviderState>()
        }
    }
}

/// Get an iterator over provider states.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_get_provider_state_iter(
    message: *mut Message,
) -> *mut ProviderStateIterator {
    ffi! {
        name: "message_get_provider_state_iter",
        params: [message],
        op: {
            let message = as_mut!(message);

            let iter = ProviderStateIterator {
                current: 0,
                message,
            };

            Ok(ptr::raw_to(iter))
        },
        fail: {
            ptr::null_mut_to::<ProviderStateIterator>()
        }
    }
}

/// Get the next value from the iterator.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn provider_state_iter_next(
    iter: *mut ProviderStateIterator,
) -> *mut ProviderState {
    ffi! {
        name: "provider_state_iter_next",
        params: [iter],
        op: {
            // Reconstitute the iterator.
            let iter = as_mut!(iter);

            // Reconstitute the message.
            let message = as_mut!(iter.message);

            // Get the current index from the iterator.
            let index = iter.next();

            // Get the value for the current index.
            let provider_state = message.provider_states.get_mut(index).ok_or(anyhow::anyhow!("iter past the end of provider states"))?;

            // Leak the value out to the C-side.
            Ok(provider_state as *mut ProviderState)
        },
        fail: {
            ptr::null_mut_to::<ProviderState>()
        }
    }
}

/// Delete the iterator.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn provider_state_iter_delete(
    iter: *mut ProviderStateIterator,
) -> c_int {
    ffi! {
        name: "provider_state_iter_delete",
        params: [iter],
        op: {
            ptr::drop_raw(iter);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
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
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_find_metadata(
    message: *const Message,
    key: *const c_char,
) -> *const c_char {
    ffi! {
        name: "message_find_metadata",
        params: [message, key],
        op: {
            // Reconstitute the message.
            let message = as_ref!(message);
            // Safely get a Rust String out of the key.
            let key = safe_str!(key);
            // Get the value, if present, for that key.
            let value = message.metadata.get(key).ok_or(anyhow::anyhow!("invalid metadata key"))?;
            // Leak the string to the C-side.
            Ok(string::to_c(value)? as *const c_char)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}

/// Insert the (`key`, `value`) pair into this Message's
/// `metadata` HashMap.
/// This function returns an enum indicating the result;
/// see the comments on HashMapInsertStatus for details.
///
/// # Errors
///
/// This function may fail if the provided `key` or `value` strings
/// contain invalid UTF-8.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_insert_metadata(
    message: *mut Message,
    key: *const c_char,
    value: *const c_char,
) -> c_int {
    use HashMapInsertStatus as Status;

    ffi! {
        name: "message_insert_metadata",
        params: [message, key, value],
        op: {
            let message = as_mut!(message);
            let key = safe_str!(key);
            let value = safe_str!(value);

            match message.metadata.insert(key.to_string(), value.to_string()) {
                None => Ok(Status::SuccessNew as c_int),
                Some(_) => Ok(Status::SuccessOverwrite as c_int),
            }
        },
        fail: {
            Status::Error as c_int
        }
    }
}

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
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn message_get_metadata_iter(
    message: *mut Message,
) -> *mut MetadataIterator {
    ffi! {
        name: "message_get_metadata_iter",
        params: [message],
        op: {
            let message = as_mut!(message);

            let iter = MetadataIterator {
                keys:  message.metadata.keys().cloned().collect(),
                current: 0,
                message: message as *const Message,
            };

            Ok(ptr::raw_to(iter))
        },
        fail: {
            ptr::null_mut_to::<MetadataIterator>()
        }
    }
}

/// Get the next key and value out of the iterator, if possible
///
/// Returns a pointer to a heap allocated array of 2 elements, the pointer to the
/// key string on the heap, and the pointer to the value string on the heap.
///
/// The user needs to free both the contained strings and the array.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::or_fun_call)]
pub unsafe extern "C" fn metadata_iter_next(
    iter: *mut MetadataIterator,
) -> *mut MetadataPair {
    ffi! {
        name: "metadata_iter_next",
        params: [iter],
        op: {
            // Reconstitute the iterator.
            let iter = as_mut!(iter);

            // Reconstitute the message.
            let message = as_ref!(iter.message);

            // Get the current key from the iterator.
            let key = iter.next().ok_or(anyhow::anyhow!("iter past the end of metadata"))?;

            // Get the value for the current key.
            let (key, value) = message.metadata.get_key_value(key).ok_or(anyhow::anyhow!("iter provided invalid metadata key"))?;

            // Package up for return.
            let pair = MetadataPair::new(key, value)?;

            // Leak the value out to the C-side.
            Ok(ptr::raw_to(pair))
        },
        fail: {
            ptr::null_mut_to::<MetadataPair>()
        }
    }
}

/// Free the metadata iterator when you're done using it.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn metadata_iter_delete(
    iter: *mut MetadataIterator,
) -> c_int {
    ffi! {
        name: "metadata_iter_delete",
        params: [iter],
        op: {
            ptr::drop_raw(iter);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// Free a pair of key and value returned from `message_next_metadata_iter`.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn metadata_pair_delete(
    pair: *mut MetadataPair,
) -> c_int {
    ffi! {
        name: "metadata_pair_delete",
        params: [pair],
        op: {
            ptr::drop_raw(pair);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// An iterator that enables FFI iteration over metadata by putting all the keys on the heap
/// and tracking which one we're currently at.
///
/// This assumes no mutation of the underlying metadata happens while the iterator is live.
#[derive(Debug)]
pub struct MetadataIterator {
    /// The metadata keys
    keys: Vec<String>,
    /// The current key
    current: usize,
    /// Pointer to the message.
    message: *const Message,
}

impl MetadataIterator {
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
pub struct MetadataPair {
    key: *const c_char,
    value: *const c_char,
}

impl MetadataPair {
    fn new(key: &str, value: &str) -> anyhow::Result<MetadataPair> {
        Ok(MetadataPair {
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
impl Drop for MetadataPair {
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
