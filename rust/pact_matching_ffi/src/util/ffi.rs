//! Provides a convenience macro for wrapping FFI code.

#![allow(unused)]

/// Makes sure FFI code is always wrapped in `catch_unwind` and sets its error.
///
/// This convenience macro is intended to make it easier to write _correct_ FFI code
/// which catches panics before they cross the language boundary, and reports its error
/// out for the C caller to read if they want.
macro_rules! ffi {
    ( op: $op:block, fail: $fail:block ) => {{
        $crate::error::catch_panic(|| $op).unwrap_or($fail)
    }};
}
