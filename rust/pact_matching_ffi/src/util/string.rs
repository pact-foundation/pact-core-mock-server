use crate::ffi_fn;
use libc::c_char;
use std::ffi::CString;
use std::ops::Not;

/// Converts the string into a C-compatible null terminated string,
/// then forgets the container while returning a pointer to the
/// underlying buffer.
///
/// The returned pointer must be passed to CString::from_raw to
/// prevent leaking memory.
pub(crate) fn to_c(t: &str) -> anyhow::Result<*mut c_char> {
    Ok(CString::new(t)?.into_raw())
}

ffi_fn! {
    /// Delete a string previously returned by this FFI.
    ///
    /// It is explicitly allowed to pass a null pointer to this function;
    /// in that case the function will do nothing.
    fn string_delete(string: *mut c_char) {
        if string.is_null().not() {
            let string = unsafe { CString::from_raw(string) };
            std::mem::drop(string);
        }
    }
}

/// Construct a CStr safely with null checks.
#[macro_export]
macro_rules! cstr {
    ( $name:ident ) => {{
        use std::ffi::CStr;

        if $name.is_null() {
            anyhow::bail!(concat!(stringify!($name), " is null"));
        }

        unsafe { CStr::from_ptr($name) }
    }};
}

/// Construct a `&str` safely with null checks.
#[macro_export]
macro_rules! safe_str {
    ( $name:ident ) => {{
        cstr!($name).to_str().context(concat!(
            "error parsing ",
            stringify!($name),
            " as UTF-8"
        ))?
    }};
}
