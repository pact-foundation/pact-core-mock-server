//! The `pact_verifier` crate provides a number of exported functions using C bindings for
//! controlling the pact verification process. These can be used in any language that supports C bindings.

#![warn(missing_docs)]

use std::ffi::CStr;
use std::panic::catch_unwind;

use anyhow::Context;
use libc::{c_char, c_int, c_ushort, EXIT_FAILURE};
use log::*;

use crate::{as_mut, ffi_fn, safe_str};
use crate::util::*;
use crate::util::string::if_null;

mod args;
pub mod verifier;
pub mod handle;

/// External interface to verifier a provider
///
/// * `args` - the same as the CLI interface, except newline delimited
///
/// # Errors
///
/// Errors are returned as non-zero numeric values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | The verification process failed, see output for errors |
/// | 2 | A null pointer was received |
/// | 3 | The method panicked |
/// | 4 | Invalid arguments were provided to the verification process |
///
/// # Safety
///
/// Exported functions are inherently unsafe. Deal.
#[no_mangle]
pub unsafe extern fn pactffi_verify(args: *const c_char) -> i32 {
  if args.is_null() {
    return 2;
  }

  let result = catch_unwind(|| {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
      let args_raw = CStr::from_ptr(args).to_string_lossy().into_owned();
      let args: Vec<String> = args_raw.lines().map(|s| s.to_string()).collect();
      let result = verifier::handle_args(args).await;

      match result {
        Ok(_) => 0,
        Err(e) => e
      }
    })
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      3
    }
  }
}

ffi_fn! {
    /// Get a Handle to a newly created verifier. You should call `pactffi_verifier_shutdown` when
    /// done with the verifier to free all allocated resources
    ///
    /// # Safety
    ///
    /// This function is safe.
    ///
    /// # Error Handling
    ///
    /// Returns NULL on error.
    fn pactffi_verifier_new() -> *mut handle::VerifierHandle {
        let handle = handle::VerifierHandle::new();
        ptr::raw_to(handle)
    } {
        ptr::null_mut_to::<handle::VerifierHandle>()
    }
}

ffi_fn! {
    /// Shutdown the verifier and release all resources
    fn pactffi_verifier_shutdown(handle: *mut handle::VerifierHandle) {
        ptr::drop_raw(handle);
    }
}

ffi_fn! {
    /// Set the provider details for the Pact verifier. Passing a NULL for any field will
    /// use the default value for that field.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_set_provider_info(
      handle: *mut handle::VerifierHandle,
      name: *const c_char,
      scheme: *const c_char,
      host: *const c_char,
      port: c_ushort,
      path: *const c_char
    ) {
      let handle = as_mut!(handle);
      let name = if_null(name, "provider");
      let scheme = if_null(scheme, "http");
      let host = if_null(host, "localhost");
      let path = if_null(path, "/");

      handle.update_provider_info(name, scheme, host, port as u16, path);
    }
}

ffi_fn! {
    /// Adds a Pact file as a source to verify.
    ///
    /// # Safety
    ///
    /// All string fields must contain valid UTF-8. Invalid UTF-8
    /// will be replaced with U+FFFD REPLACEMENT CHARACTER.
    ///
    fn pactffi_verifier_add_file_source(
      handle: *mut handle::VerifierHandle,
      file: *const c_char
    ) {
      let handle = as_mut!(handle);
      let file = safe_str!(file);

      handle.add_file_source(file);
    }
}

ffi_fn! {
    /// Runs the verification.
    ///
    /// # Error Handling
    ///
    /// Errors will be reported with a non-zero return value.
    fn pactffi_verifier_execute(handle: *mut handle::VerifierHandle) -> c_int {
      let handle = as_mut!(handle);

      handle.execute()
    } {
      EXIT_FAILURE
    }
}
