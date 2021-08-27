//! The `pact_verifier` crate provides a number of exported functions using C bindings for
//! controlling the pact verification process. These can be used in any language that supports C bindings.

#![warn(missing_docs)]

use std::ffi::{CStr, CString, OsStr};
use std::panic::catch_unwind;

use anyhow::Context;
use libc::{c_char, c_int, c_ushort, EXIT_FAILURE};
use log::*;
use std::env;

use crate::{as_mut, ffi_fn, safe_str};
use crate::util::*;
use crate::util::string::if_null;
use serde::{Serialize, Deserialize};
use std::any::Any;
use clap::ArgSettings;

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

#[derive(Serialize, Deserialize)]
pub struct Argument {
    long: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    short: Option<String>,
    help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    possible_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<String>,
    multiple: Option<bool>
}

#[derive(Serialize, Deserialize)]
pub struct OptionsFlags {
    pub options: Vec<Argument>,
    pub flags: Vec<Argument>
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
/// {
///   "options": [
///     {
///       "long": "scheme",
///       "help": "Provider URI scheme (defaults to http)",
///       "possible_values": [
///         "http",
///         "https"
///       ],
///       "default_value": "http"
///       "multiple": false,
///     },
///     {
///       "long": "file",
///       "short": "f",
///       "help": "Pact file to verify (can be repeated)",
///       "multiple": true
///     },
///     {
///       "long": "user",
///       "help": "Username to use when fetching pacts from URLS",
///       "multiple": false
///     }
///   ],
///   "flags": [
///     {
///       "long": "disable-ssl-verification",
///       "help": "Disables validation of SSL certificates",
///       "multiple": false
///     }
///   ]
/// }
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
    let mut options: Vec<Argument> = Vec::new();
    let mut flags: Vec<Argument> = Vec::new();

    for opt in app.p.opts.iter() {
        let arg = parse_argument(opt.s.long, opt.s.short, opt.b.help, opt.v.possible_vals.clone(),opt.v.default_val, opt.b.settings.is_set(ArgSettings::Multiple));
        options.push(arg);
    }

    for opt in app.p.flags.iter() {
        let arg = parse_argument(opt.s.long, opt.s.short, opt.b.help, None, None,opt.b.settings.is_set(ArgSettings::Multiple));
        flags.push(arg);
    }


    let opts_flags = OptionsFlags { options:options, flags: flags};
    let json = serde_json::to_string(&opts_flags).unwrap();
    let c_str = CString::new(json).unwrap();
    c_str.into_raw() as *const c_char
}

fn parse_argument(long: Option<&str>, short: Option<char>, help: Option<&str>, possible_values: Option<Vec<&str>>, default_value: Option<&OsStr>, multiple: bool) -> Argument {
    let mut arg = Argument { short: None, long: None, help: None, possible_values: None, default_value: None, multiple: Some(false) };

    // Long
    match long {
        None => {}
        Some(_val) => {
            let c_str = CString::new(_val.to_string()).unwrap();
            let long = c_str.to_str().unwrap();
            arg.long = Some(long.to_string());
        }
    }

    // Short
    match short {
        None => {}
        Some(_val) => {
            let c_str = CString::new(_val.to_string()).unwrap();
            let short = c_str.to_str().unwrap();
            arg.short = Some(short.to_string());
        }
    }

    // Help
    match help {
        None => {}
        Some(_val) => {
            let c_str = CString::new(_val.to_string()).unwrap();
            let help = c_str.to_str().unwrap();
            arg.help = Some(help.to_string());
        }
    }

    // Possible values
    match possible_values {
        None => {}
        Some(_) => {
            let mut possible_vals: Vec<String> = Vec::new();
            for possible_val in possible_values.unwrap().iter() {
                possible_vals.push(possible_val.to_string())
            }
            arg.possible_values = Some(possible_vals);
        }
    }

    // Default value
    match default_value {
        None => {}
        Some(_val) =>
            {
                let x = _val.to_os_string().into_string().unwrap();
                let c_str = CString::new(x).unwrap();
                let default_val = c_str.to_str().unwrap();
                arg.default_value = Some(default_val.to_string());
            }
    }

    // Multiple
    if multiple {
        arg.multiple = Some(true);
    }

    arg
}
