//! FFI wrapper code for pact_matching::models::Provider
use libc::c_char;

use anyhow::anyhow;

pub use pact_matching::models::Provider;

use crate::util::*;
use crate::{as_ref, ffi};

/// Get a copy of this provider's name.
/// The copy must be deleted with `string_delete`.
///
/// # Errors
///
/// This function will fail if it is passed a NULL pointer,
/// or the Rust string contains an embedded NULL byte.
/// In the case of error, a NULL pointer will be returned.
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn provider_get_name(
    provider: *const Provider,
) -> *const c_char {
    ffi! {
        name: "provider_get_name",
        params: [provider],
        op: {
            let provider = as_ref!(provider);
            Ok(string::to_c(&provider.name)? as *const c_char)
        },
        fail: {
            ptr::null_to::<c_char>()
        }
    }
}
