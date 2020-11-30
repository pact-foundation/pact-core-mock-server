//! The `pact_mock_server` crate provides a number of exported functions using C bindings for
//! controlling the mock server. These can be used in any language that supports C bindings.
//!
//! ## [create_mock_server](fn.create_mock_server_ffi.html)
//!
//! External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
//! as well as the port for the mock server to run on. A value of 0 for the port will result in a
//! port being allocated by the operating system. The port of the mock server is returned.
//!
//! ## [mock_server_matched](fn.mock_server_matched_ffi.html)
//!
//! Simple function that returns a boolean value given the port number of the mock service. This value will be true if all
//! the expectations of the pact that the mock server was created with have been met. It will return false if any request did
//! not match, an un-recognised request was received or an expected request was not received.
//!
//! ## [mock_server_mismatches](fn.mock_server_mismatches_ffi.html)
//!
//! This returns all the mismatches, un-expected requests and missing requests in JSON format, given the port number of the
//! mock server.
//!
//! **IMPORTANT NOTE:** The JSON string for the result is allocated on the rust heap, and will have to be freed once the
//! code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is provided for this purpose. If the mock
//! server is not cleaned up properly, this will result in memory leaks as the rust heap will not be reclaimed.
//!
//! ## [cleanup_mock_server](fn.cleanup_mock_server.html)
//!
//! This function will try terminate the mock server with the given port number and cleanup any memory allocated for it by
//! the [`mock_server_mismatches`](fn.mock_server_mismatches.html) function. Returns `true`, unless a mock server with the given port number does not exist,
//! or the function fails in some way.
//!
//! **NOTE:** Although `close()` on the listerner for the mock server is called, this does not currently work and the
//! listerner will continue handling requests. In this case, it will always return a 501 once the mock server has been
//! cleaned up.
//!
//! ## [write_pact_file](fn.write_pact_file.html)
//!
//! External interface to trigger a mock server to write out its pact file. This function should
//! be called if all the consumer tests have passed. The directory to write the file to is passed
//! as the second parameter. If a NULL pointer is passed, the current working directory is used.
//!
//! Returns 0 if the pact file was successfully written. Returns a positive code if the file can
//! not be written, or there is no mock server running on that port or the function panics.

#![warn(missing_docs)]

use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use std::panic::catch_unwind;
use std::env;
use libc::{c_char};
use log::*;
use env_logger::Builder;

mod args;
mod verifier;


const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// Get the current library version
///
/// **NOTE:** The string for the result is allocated on the heap, and will have to be freed
/// by the caller using free_string
///
/// # Errors
///
/// An empty string indicates an error determining the current crate version
#[no_mangle]
pub extern fn version() -> *mut c_char {
  let s = CString::new(VERSION).unwrap_or_default();
  s.into_raw()
}

/// Initialise the mock server library, can provide an environment variable name to use to
/// set the log levels.
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub unsafe extern fn init(log_env_var: *const c_char) {
  let log_env_var = if !log_env_var.is_null() {
    let c_str = CStr::from_ptr(log_env_var);
    match c_str.to_str() {
      Ok(str) => str,
      Err(err) => {
        warn!("Failed to parse the environment variable name as a UTF-8 string: {}", err);
        "LOG_LEVEL"
      }
    }
  } else {
    "LOG_LEVEL"
  };

  let env = env_logger::Env::new().filter(log_env_var);
  let mut builder = Builder::from_env(env);
  builder.try_init().unwrap_or(());
}

/// Frees the memory allocated to a string by another function
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub unsafe extern fn free_string(s: *mut c_char) {
  if s.is_null() {
    return;
  }
  CString::from_raw(s);
}


/// Runs the verification process
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]

/// External interface to verifier a provider
///
/// * `args` - the same as the CLI interface, except newline delimited
///
/// # Errors
///
/// Errors are returned as negative values.
///
/// | Error | Description |
/// |-------|-------------|
/// | -1 | A null pointer was received |
/// | -2 | The pact JSON could not be parsed |
/// | -3 | The mock server could not be started |
/// | -4 | The method panicked |
/// | -5 | The address is not valid |
/// | -6 | Could not create the TLS configuration with the self-signed certificate |
///
#[no_mangle]
pub unsafe extern fn verify(args: *const c_char) -> i32 {
  let mut runtime = tokio::runtime::Builder::new()
    .basic_scheduler()
    .enable_all()
    .build()
    .expect("new runtime");

  if args.is_null() {
    return -1;
  }

  if args.is_null() {
    log::error!("Got a null pointer instead of verifier arguments");
    return -1;
  }

  let args_raw = CStr::from_ptr(args).to_string_lossy().into_owned();
  let args: Vec<String> = args_raw.lines().map(|s| s.to_string()).collect();
  log::info!("args {:?}", args);

  let result = runtime.block_on(verifier::handle_args(args));

  match result {
    Ok(_) => 0,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      cause
    }
  }
}


// Option 1: lots of function flags -> this will be painful, because no rust safety for things like optionals, argument ordering will be messy
// Option 2: array of strings to pass to clap -> aka treat the interface the same as the CLI, except via an FFI? potential re-use of CLI code, or at least logic
// Option 3: pass in big JSON string key/value -> marshal/unmarshalling from JSON should be fairly easy
