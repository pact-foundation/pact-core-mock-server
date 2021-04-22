//! FFI wrapper code for pact_matching::models::Consumer

use crate::util::*;
use crate::{as_ref, ffi_fn};
use libc::c_char;

pub use pact_matching::models::Consumer;

ffi_fn! {
    /// Get a copy of this consumer's name.
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
    /// Consumer *consumer = message_pact_get_consumer(message_pact);
    /// if (consumer == NULLPTR) {
    ///     // handle error.
    /// }
    ///
    /// char *name = consumer_get_name(consumer);
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
    fn consumer_get_name(consumer: *const Consumer) -> *const c_char {
        let consumer = as_ref!(consumer);
        string::to_c(&consumer.name)? as *const c_char
    } {
        ptr::null_to::<c_char>()
    }
}
