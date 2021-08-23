//! The `pact_verifier` crate provides a number of exported functions using C bindings for
//! controlling the pact verification process. These can be used in any language that supports C bindings.

#![warn(missing_docs)]

use std::ffi::{CStr, CString};
use std::panic::catch_unwind;

use anyhow::Context;
use libc::{c_char, c_int, c_ushort, EXIT_FAILURE};
use log::*;
use std::env;

use crate::{as_mut, ffi_fn, safe_str};
use crate::util::*;
use crate::util::string::if_null;
use serde::Serialize;

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

#[derive(Serialize)]
struct Argument {
    name: Option<String>,
    short: Option<String>,
    long: Option<String>,
    help: Option<String>,
    possible_values: Option<Vec<String>>
}

/// External interface to retrieve the options and arguments available when calling the CLI interface,
/// returning them as a JSON string.
///
/// The purpose is to then be able to use in other languages which wrap the FFI library, to implement
/// the same CLI functionality automatically without manual maintenance of arguments, help descriptions
/// etc.
///
/// # Example structure
/// ```json
/// [
///     {
///         "name": "broker-url",
///         "short": "b",
///         "long": "broker-url",
///         "help": "URL of the pact broker to fetch pacts from to verify (requires the provider name parameter)"
///     },
///     {
///         "name": "port",
///         "short": "p",
///         "long": "port",
///         "help": "Provider port (defaults to protocol default 80/443)"
///     },
///     {
///         "name": "user",
///         "short": null,
///         "long": "user",
///         "help": "Username to use when fetching pacts from URLS"
///     }
/// ]
/// ```
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub extern "C" fn pactffi_verifier_cli_args() -> *const c_char {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let app = args::setup_app(program, clap::crate_version!());

    // Iterate through the args, extracting info from each to then add to a Vector of args
    let mut args: Vec<Argument> = Vec::new();
    for opt in app.p.opts.iter() {
        let mut arg = Argument { name: None, short: None, long: None, help: None, possible_values: None };

        // Name
        // TODO: Maybe superfluous as this is always the same as the long
        let arg_name = opt.b.name;
        arg.name = Some(arg_name.to_string());

        // Short
        match opt.s.short {
            None => {}
            Some(_val) => {
                let c_str = CString::new(_val.to_string()).unwrap();
                let short = c_str.to_str().unwrap();
                arg.short = Some(short.to_string());
            }
        }

        // Long
        match opt.s.long {
            None => {}
            Some(_val) => {
                let c_str = CString::new(_val.to_string()).unwrap();
                let long = c_str.to_str().unwrap();
                arg.long = Some(long.to_string());
            }
        }

        // Help
        match opt.b.help {
            None => {}
            Some(_val) => {
                let c_str = CString::new(_val.to_string()).unwrap();
                let help = c_str.to_str().unwrap();
                arg.help = Some(help.to_string());
            }
        }

        // Possible Values
        match opt.v.possible_vals {
            None => {}
            Some(_) => {
                let mut possible_vals: Vec<String> = Vec::new();
                let possible_values = opt.v.possible_vals.clone().unwrap();
                for possible_val in possible_values.iter() {
                    possible_vals.push(possible_val.to_string())
                }
                arg.possible_values = Some(possible_vals);
            }
        }


        args.push(arg);
    }

    let json = serde_json::to_string(&args).unwrap();
    let c_str = CString::new(json).unwrap();
    c_str.into_raw() as *const c_char
}