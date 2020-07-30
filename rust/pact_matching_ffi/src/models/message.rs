//! The Pact `Message` type, including associated matching rules and provider states.

/*===============================================================================================
 * # Imports
 *---------------------------------------------------------------------------------------------*/

use crate::models::pact_specification::PactSpecification;
use crate::util::*;
use crate::{as_mut, as_ref, cstr, ffi, safe_str};
use anyhow::{anyhow, Context};
use libc::{c_char, c_int, c_uint, EXIT_FAILURE, EXIT_SUCCESS};

/*===============================================================================================
 * # Re-Exports
 *---------------------------------------------------------------------------------------------*/

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message::Message;
pub use pact_matching::models::provider_states::ProviderState;

/*===============================================================================================
 * # FFI Functions
 *---------------------------------------------------------------------------------------------*/

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
            let description = string::into_leaked_cstring(&message.description)?;
            Ok(description)
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
            let message = as_ref!(message);
            let key = safe_str!(key);

            match message.metadata.get(key) {
                None => Ok(ptr::null_to::<c_char>()),
                Some(value) => {
                    Ok(string::into_leaked_cstring(value)?)
                },
            }
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

/*
/// Get a copy of the metadata list from this message.
/// It is in the form of a list of (key, value) pairs,
/// in an unspecified order.
/// The returned structure must be deleted with `metadata_list_delete`.
///
/// Since it is a copy, the returned structure may safely outlive
/// the `Message`.
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
pub unsafe extern "C" fn message_get_metadata_list(
    message: *mut Message,
) -> *mut MetadataIterator {
    ffi! {
        name: "message_get_metadata_list",
        params: [message],
        op: {
            let message = as_mut!(message);

            todo!()
        },
        fail: {
            ptr::null_to::<MetadataIterator>()
        }
    }
}
*/

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
