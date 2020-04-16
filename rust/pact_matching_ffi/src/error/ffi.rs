//! The FFI functions exposed for getting the last error.

use crate::error::last_error::get_error_msg;
use crate::error::status::Status;
use crate::util::write::write_to_c_buf;
use libc::{c_char, c_int};
use std::slice;

/// Provide the error message from `LAST_ERROR` to the calling C code.
///
/// # Params
///
/// * `buffer`: a pointer to an array of `char` of sufficient length to hold the error message.
/// * `length`: an int providing the length of the `buffer`.
///
/// # Return Codes
///
/// * The number of bytes written to the provided buffer, which may be zero if there is no last error.
/// * `-1` if the provided buffer is a null pointer.
/// * `-2` if the provided buffer length is too small for the error message.
/// * `-3` if the write failed for some other reason.
/// * `-4` if the error message had an interior NULL
///
/// # Notes
///
/// Note that this function zeroes out any excess in the provided buffer.
#[no_mangle]
pub extern "C" fn get_error_message(
    buffer: *mut c_char,
    length: c_int,
) -> c_int {
    // Make sure the buffer isn't null.
    if buffer.is_null() {
        return Status::NullBuffer as c_int;
    }

    // Convert the buffer raw pointer into a byte slice.
    let buffer = unsafe {
        slice::from_raw_parts_mut(buffer as *mut u8, length as usize)
    };

    // Get the last error, possibly empty if there isn't one.
    let last_err = get_error_msg().unwrap_or(String::new());

    // Try to write the error to the buffer.
    let status = match write_to_c_buf(&last_err, buffer) {
        Ok(_) => Status::Success,
        Err(err) => Status::from(err),
    };

    status as c_int
}
