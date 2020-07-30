//! Provides a convenience macro for wrapping FFI code.

/// Makes sure FFI code is always wrapped in `catch_unwind` and sets its error.
///
/// This convenience macro is intended to make it easier to write _correct_ FFI code
/// which catches panics before they cross the language boundary, and reports its error
/// out for the C caller to read if they want.
#[doc(hidden)]
#[macro_export]
macro_rules! ffi {
    ( op: $op:block, fail: $fail:block ) => {{
        compile_error!("the ffi macro must include a name and a list of params");
    }};

    ( name: $name:literal, op: $op:block, fail: $fail:block ) => {{
        compile_error!("the ffi macro must include a list of params");
    }};

    ( params: [ $( $params:ident ),* ], op: $op:block, fail: $fail:block ) => {{
        compile_error!("the ffi macro must include a name");
    }};

    ( name: $name:literal, params: [ $( $params:ident ),* ], op: $op:block, fail: $fail:block ) => {{
        use $crate::log::TARGET;

        log::debug!(target: TARGET, "{}::{}", module_path!(), $name);

        $(
            log::trace!(target: TARGET, "@param $params = {:?}", $params);
        )*

        let output = $crate::error::catch_panic(|| $op).unwrap_or($fail);

        log::trace!(target: TARGET, "@return {:?}", output);

        output
    }};
}
