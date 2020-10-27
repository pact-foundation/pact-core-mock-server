//! FFI wrapper code for pact_matching::models::Consumer
use libc::c_char;

use anyhow::anyhow;

pub use pact_matching::models::Consumer;

use crate::util::*;
use crate::{as_ref, ffi};

/// Get a copy of this consumer's name.
/// The copy must be deleted with `string_delete`.
///
/// # Errors
///
/// This function will fail if it is passed a NULL pointer,
/// or the Rust string contains an embedded NULL byte.
/// In the case of error, a NULL pointer will be returned.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn consumer_get_name(
    consumer: *const Consumer,
) -> *const c_char {
    ffi! {
        name: "consumer_get_name",
        params: [consumer],
        op: {
            let consumer = as_ref!(consumer);
            Ok(string::to_c(&consumer.name)? as *const c_char)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}
