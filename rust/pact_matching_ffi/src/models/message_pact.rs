//! FFI wrapper for `MessagePact` from pact_matching.

use anyhow::{anyhow, Context};
use libc::{c_char, c_int, EXIT_FAILURE, EXIT_SUCCESS};

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message_pact::MessagePact;
use pact_matching::models::Consumer;
use pact_matching::models::Provider;

use crate::util::*;
use crate::{as_mut, cstr, ffi, safe_str};

/// Construct a new `MessagePact` from the JSON string.
/// The provided file name is used when generating error messages.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_pact_new_from_json(
    file_name: *const c_char,
    json_str: *const c_char,
) -> *mut MessagePact {
    ffi! {
        name: "message_pact_new_from_json",
        params: [file_name, json_str],
        op: {
            let file_name = safe_str!(file_name);
            let json_str = safe_str!(json_str);

            let json_value: serde_json::Value =
                serde_json::from_str(json_str)
                .context("error parsing json_str as JSON")?;

            let message_pact = MessagePact::from_json(
                &file_name.to_string(),
                &json_value,
            ).map_err(|e| anyhow!("{}", e))?;

            Ok(ptr::raw_to(message_pact))
        },
        fail: {
            ptr::null_mut_to::<MessagePact>()
        }
    }
}

/// Delete the `MessagePact` being pointed to.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_pact_delete(
    message_pact: *mut MessagePact,
) -> c_int {
    ffi! {
        name: "message_pact_delete",
        params: [message_pact],
        op: {
            ptr::drop_raw(message_pact);
            Ok(EXIT_SUCCESS)
        },
        fail: {
            EXIT_FAILURE
        }
    }
}

/// Get a pointer to the Consumer struct inside the MessagePact.
/// This is a mutable borrow: The caller may mutate the Consumer
/// through this pointer.
///
/// # Errors
///
/// This function will only fail if it is passed a NULL pointer.
/// In the case of error, a NULL pointer will be returned.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_pact_get_consumer(
    message_pact: *mut MessagePact,
) -> *mut Consumer {
    ffi! {
        name: "message_pact_get_consumer",
        params: [message_pact],
        op: {
            let message_pact = as_mut!(message_pact);
            let consumer = &mut message_pact.consumer;
            Ok(consumer as *mut Consumer)
        },
        fail: {
            ptr::null_mut_to::<Consumer>()
        }
    }
}

/// Get a pointer to the Provider struct inside the MessagePact.
/// This is a mutable borrow: The caller may mutate the Provider
/// through this pointer.
///
/// # Errors
///
/// This function will only fail if it is passed a NULL pointer.
/// In the case of error, a NULL pointer will be returned.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn message_pact_get_provider(
    message_pact: *mut MessagePact,
) -> *mut Provider {
    ffi! {
        name: "message_pact_get_provider",
        params: [message_pact],
        op: {
            let message_pact = as_mut!(message_pact);
            let provider = &mut message_pact.provider;
            Ok(provider as *mut Provider)
        },
        fail: {
            ptr::null_mut_to::<Provider>()
        }
    }
}
