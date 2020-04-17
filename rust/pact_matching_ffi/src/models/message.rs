//! The Pact `Message` type, including associated matching rules and provider states.

use std::ffi::CStr;
use anyhow::Context;
use libc::{c_char, c_int, c_uint, EXIT_FAILURE, EXIT_SUCCESS};

use crate::ffi;
use crate::util::*;
use crate::models::pact_specification::PactSpecification;

// Necessary to make 'cbindgen' generate an opaque struct on the C side.
pub use pact_matching::models::message::Message;

/// Get a mutable pointer to a newly-created default message on the heap.
#[no_mangle]
pub extern "C" fn message_new() -> *mut Message {
    ffi! {
        name: "message_new",
        op: {
            Ok(ptr::raw_to(Message::default()))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}

/// Destroy the `Message` being pointed to.
#[no_mangle]
pub extern "C" fn message_delete(message: *mut Message) -> c_int {
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

/// Constructs a `Message` from the JSON string
#[no_mangle]
pub extern "C" fn message_from_json(
    index: c_uint,
    json_str: *const c_char,
    spec_version: PactSpecification,
) -> *mut Message {
    ffi! {
        name: "message_from_json",
        op: {
            if json_str.is_null() {
                anyhow::bail!("json_str is null");
            }

            let json_str = unsafe { CStr::from_ptr(json_str) };
            let json_str = json_str
                .to_str()
                .context("Error parsing json_str as UTF-8")?;

            let json_value: serde_json::Value =
                serde_json::from_str(json_str)
                .context("Error parsing json_str as JSON")?;

            let message = Message::from_json(
                index as usize,
                &json_value,
                &spec_version.into())
                .map_err(|e| anyhow::anyhow!("Pact error: {}", e))?;

            Ok(ptr::raw_to(message))
        },
        fail: {
            ptr::null_mut_to::<Message>()
        }
    }
}
