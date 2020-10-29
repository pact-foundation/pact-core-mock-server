//! FFI wrapper code for pact_matching::models::Consumer
use libc::c_char;

pub use pact_matching::models::Consumer;

use crate::util::*;
use crate::{as_ref, ffi_fn};

ffi_fn! {
    /// Get a copy of this consumer's name.
    /// The copy must be deleted with `string_delete`.
    ///
    /// # Errors
    ///
    /// This function will fail if it is passed a NULL pointer,
    /// or the Rust string contains an embedded NULL byte.
    /// In the case of error, a NULL pointer will be returned.
    fn consumer_get_name(consumer: *const Consumer) -> *const c_char {
        let consumer = as_ref!(consumer);
        string::to_c(&consumer.name)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}
