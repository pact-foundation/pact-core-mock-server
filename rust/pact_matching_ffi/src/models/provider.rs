//! FFI wrapper code for pact_matching::models::Provider
use libc::c_char;

pub use pact_matching::models::Provider;

use crate::util::*;
use crate::{as_ref, ffi_fn};

ffi_fn! {
    /// Get a copy of this provider's name.
    ///
    /// The copy must be deleted with `string_delete`.
    ///
    /// # Usage
    ///
    /// ```c
    /// // Assuming `file_name` and `json_str` are already defined.
    ///
    /// MessagePact *message_pact = message_pact_new_from_json(file_name, json_str);
    /// if (message_pact == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// Provider *provider = message_pact_get_provider(message_pact);
    /// if (provider == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// char *name = provider_get_name(provider);
    /// if (name == NULL) {
    ///     // handle error.
    /// }
    ///
    /// printf("%s\n", name);
    ///
    /// string_delete(name);
    /// ```
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
