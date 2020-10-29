//! FFI wrapper code for pact_matching::models::Provider
use libc::c_char;

pub use pact_matching::models::Provider;

use crate::util::*;
use crate::{as_ref, ffi_fn};

ffi_fn! {
    /// Get a copy of this provider's name.
    /// The copy must be deleted with `string_delete`.
    ///
    /// # Errors
    ///
    /// This function will fail if it is passed a NULL pointer,
    /// or the Rust string contains an embedded NULL byte.
    /// In the case of error, a NULL pointer will be returned.
    fn provider_get_name(provider: *const Provider) -> *const c_char {
        let provider = as_ref!(provider);
        string::to_c(&provider.name)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}
