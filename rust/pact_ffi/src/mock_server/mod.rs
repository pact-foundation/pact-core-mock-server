//! The `mock_server` module provides a number of exported functions using C bindings for
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

use std::{ptr, str};
use std::any::Any;
use std::ffi::CStr;
use std::ffi::CString;
use std::panic::catch_unwind;
use std::str::from_utf8;

use chrono::Local;
use libc::c_char;
use log::*;
use onig::Regex;
use pact_models::pact::Pact;
use pact_models::time_utils::{parse_pattern, to_chrono_pattern};
use rand::prelude::*;
use serde_json::json;
use uuid::Uuid;

use pact_matching::logging::fetch_buffer_contents;
use pact_mock_server::{MANAGER, MockServerError, tls::TlsConfigBuilder, WritePactFileErr};
use pact_mock_server::mock_server::MockServerConfig;
use pact_mock_server::server_manager::ServerManager;

use crate::convert_cstr;
use crate::mock_server::handles::path_from_dir;

pub mod handles;
pub mod bodies;
mod xml;

/// External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
/// as well as the port for the mock server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// * `pact_str` - Pact JSON
/// * `addr_str` - Address to bind to in the form name:port (i.e. 127.0.0.1:0)
/// * `tls` - boolean flag to indicate of the mock server should use TLS (using a self-signed certificate)
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
pub extern fn pactffi_create_mock_server(pact_str: *const c_char, addr_str: *const c_char, tls: bool) -> i32 {
  let result = catch_unwind(|| {
    let c_str = unsafe {
      if pact_str.is_null() {
        log::error!("Got a null pointer instead of pact json");
        return -1;
      }
      CStr::from_ptr(pact_str)
    };

    let addr_c_str = unsafe {
      if addr_str.is_null() {
        log::error!("Got a null pointer instead of listener address");
        return -1;
      }
      CStr::from_ptr(addr_str)
    };

    let tls_config = if tls {
      let key = include_str!("self-signed.key");
      let cert = include_str!("self-signed.crt");
      match TlsConfigBuilder::new()
        .key(key.as_bytes())
        .cert(cert.as_bytes())
        .build() {
        Ok(tls_config) => Some(tls_config),
        Err(err) => {
          error!("Failed to build TLS configuration - {}", err);
          return -6;
        }
      }
    } else {
      None
    };

    if let Ok(Ok(addr)) = str::from_utf8(addr_c_str.to_bytes()).map(|s| s.parse::<std::net::SocketAddr>()) {
      let server_result = match tls_config {
        Some(tls_config) => pact_mock_server::create_tls_mock_server(str::from_utf8(c_str.to_bytes()).unwrap(), addr, &tls_config),
        None => pact_mock_server::create_mock_server(str::from_utf8(c_str.to_bytes()).unwrap(), addr)
      };
      match server_result {
        Ok(ms_port) => ms_port,
        Err(err) => match err.downcast_ref::<MockServerError>() {
          Some(err) => match err {
            MockServerError::InvalidPactJson => -2,
            MockServerError::MockServerFailedToStart => -3
          },
          None => -3
        }
      }
    }
    else {
      -5
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      -4
    }
  }
}

/// Fetch the CA Certificate used to generate the self-signed certificate for the TLS mock server.
///
/// **NOTE:** The string for the result is allocated on the heap, and will have to be freed
/// by the caller using free_string
///
/// # Errors
///
/// An empty string indicates an error reading the pem file
#[no_mangle]
pub extern fn pactffi_get_tls_ca_certificate() -> *mut c_char  {
  let cert_file = include_str!("ca.pem");
  let cert_str = CString::new(cert_file).unwrap_or_default();

  cert_str.into_raw()
}

/// External interface to create a mock server. A Pact handle is passed in,
/// as well as the port for the mock server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// * `pact` - Handle to a Pact model
/// * `addr_str` - Address to bind to in the form name:port (i.e. 127.0.0.1:0)
/// * `tls` - boolean flag to indicate of the mock server should use TLS (using a self-signed certificate)
///
/// # Errors
///
/// Errors are returned as negative values.
///
/// | Error | Description |
/// |-------|-------------|
/// | -1 | An invalid handle was received |
/// | -3 | The mock server could not be started |
/// | -4 | The method panicked |
/// | -5 | The address is not valid |
/// | -6 | Could not create the TLS configuration with the self-signed certificate |
///
#[no_mangle]
pub extern fn pactffi_create_mock_server_for_pact(pact: handles::PactHandle, addr_str: *const c_char, tls: bool) -> i32 {
  let result = catch_unwind(|| {
    let addr_c_str = unsafe {
      if addr_str.is_null() {
        log::error!("Got a null pointer instead of listener address");
        return -5;
      }
      CStr::from_ptr(addr_str)
    };

    let tls_config = if tls {
      let key = include_str!("self-signed.key");
      let cert = include_str!("self-signed.crt");
      match TlsConfigBuilder::new()
        .key(key.as_bytes())
        .cert(cert.as_bytes())
        .build() {
        Ok(tls_config) => Some(tls_config),
        Err(err) => {
          error!("Failed to build TLS configuration - {}", err);
          return -6;
        }
      }
    } else {
      None
    };

    if let Ok(Ok(addr)) = str::from_utf8(addr_c_str.to_bytes()).map(|s| s.parse::<std::net::SocketAddr>()) {
      pact.with_pact(&move |_, inner| {
        let config = MockServerConfig { cors_preflight: true, pact_specification: inner.specification_version };
        let server_result = match &tls_config {
          Some(tls_config) => pact_mock_server::start_tls_mock_server_with_config(
            Uuid::new_v4().to_string(), inner.pact.boxed(), addr, tls_config, config),
          None => pact_mock_server::start_mock_server_with_config(Uuid::new_v4().to_string(),
            inner.pact.boxed(), addr, config)
        };
        match server_result {
          Ok(ms_port) => {
            inner.mock_server_started = true;
            ms_port
          },
          Err(err) => {
            error!("Failed to start mock server - {}", err);
            -3
          }
        }
      }).unwrap_or(-1)
    }
    else {
      -5
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      -4
    }
  }
}

/// External interface to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched, or
/// the method panics.
#[no_mangle]
pub extern fn pactffi_mock_server_matched(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    pact_mock_server::mock_server_matched(mock_server_port)
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      error!("Caught a general panic: {:?}", cause);
      false
    }
  }
}

/// External interface to get all the mismatches from a mock server. The port number of the mock
/// server is passed in, and a pointer to a C string with the mismatches in JSON format is
/// returned.
///
/// **NOTE:** The JSON string for the result is allocated on the heap, and will have to be freed
/// once the code using the mock server is complete. The [`cleanup_mock_server`](fn.cleanup_mock_server.html) function is
/// provided for this purpose.
///
/// # Errors
///
/// If there is no mock server with the provided port number, or the function panics, a NULL
/// pointer will be returned. Don't try to dereference it, it will not end well for you.
///
#[no_mangle]
pub extern fn pactffi_mock_server_mismatches(mock_server_port: i32) -> *mut c_char {
  let result = catch_unwind(|| {
    let result = MANAGER.lock().unwrap()
      .get_or_insert_with(ServerManager::new)
      .find_mock_server_by_port_mut(mock_server_port as u16, &|ref mut mock_server| {
        let mismatches = mock_server.mismatches().iter()
          .map(|mismatch| mismatch.to_json() )
          .collect::<Vec<serde_json::Value>>();
        let json = json!(mismatches);
        let s = CString::new(json.to_string()).unwrap();
        let p = s.as_ptr();
        mock_server.resources.push(s);
        p
      });
    match result {
      Some(p) => p as *mut _,
      None => std::ptr::null_mut()
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      error!("{}", error_message(cause, "mock_server_mismatches"));
      std::ptr::null_mut()
    }
  }
}

/// External interface to cleanup a mock server. This function will try terminate the mock server
/// with the given port number and cleanup any memory allocated for it. Returns true, unless a
/// mock server with the given port number does not exist, or the function panics.
#[no_mangle]
pub extern fn pactffi_cleanup_mock_server(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    pact_mock_server::shutdown_mock_server(mock_server_port)
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      false
    }
  }
}

/// External interface to trigger a mock server to write out its pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If a NULL pointer is passed, the current working directory is used.
///
/// If overwrite is true, the file will be overwritten with the contents of the current pact.
/// Otherwise, it will be merged with any existing pact file.
///
/// Returns 0 if the pact file was successfully written. Returns a positive code if the file can
/// not be written, or there is no mock server running on that port or the function panics.
///
/// # Errors
///
/// Errors are returned as positive values.
///
/// | Error | Description |
/// |-------|-------------|
/// | 1 | A general panic was caught |
/// | 2 | The pact file was not able to be written |
/// | 3 | A mock server with the provided port was not found |
#[no_mangle]
pub extern fn pactffi_write_pact_file(mock_server_port: i32, directory: *const c_char, overwrite: bool) -> i32 {
  let result = catch_unwind(|| {
    let dir = path_from_dir(directory, None);
    let path = dir.map(|path| path.into_os_string().into_string().unwrap_or_default());

    pact_mock_server::write_pact_file(mock_server_port, path, overwrite)
  });

  match result {
    Ok(val) => match val {
      Ok(_) => 0,
      Err(err) => match err {
        WritePactFileErr::IOError => 2,
        WritePactFileErr::NoMockServer => 3
      }
    },
    Err(cause) => {
      log::error!("Caught a general panic: {:?}", cause);
      1
    }
  }
}

/// Fetch the logs for the mock server. This needs the memory buffer log sink to be setup before
/// the mock server is started. Returned string will be freed with the `cleanup_mock_server`
/// function call.
///
/// Will return a NULL pointer if the logs for the mock server can not be retrieved.
#[no_mangle]
pub extern fn pactffi_mock_server_logs(mock_server_port: i32) -> *const c_char {
  let result = catch_unwind(|| {
    MANAGER.lock().unwrap()
      .get_or_insert_with(ServerManager::new)
      .find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
        match from_utf8(&fetch_buffer_contents(&mock_server.id)) {
          Ok(contents) => match CString::new(contents.to_string()) {
            Ok(c_str) => {
              let p = c_str.as_ptr();
              mock_server.resources.push(c_str);
              p
            },
            Err(err) => {
              error!("Failed to copy in-memory log buffer - {}", err);
              ptr::null()
            }
          }
          Err(err) => {
            error!("Failed to convert in-memory log buffer to UTF-8 = {}", err);
            ptr::null()
          }
        }
      })
  });

  match result {
    Ok(val) => val.unwrap_or_else(ptr::null),
    Err(cause) => {
      error!("Caught a general panic: {:?}", cause);
      ptr::null()
    }
  }
}

fn error_message(err: Box<dyn Any>, method: &str) -> String {
  if let Some(err) = err.downcast_ref::<&str>() {
    format!("{} failed with an error - {}", method, err)
  } else if let Some(err) = err.downcast_ref::<String>() {
    format!("{} failed with an error - {}", method, err)
  } else {
    format!("{} failed with an unknown error", method)
  }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Result of wrapping a string value
pub enum StringResult {
  /// Was generated OK
  Ok(*mut c_char),
  /// There was an error generating the string
  Failed(*mut c_char)
}

/// Generates a datetime value from the provided format string, using the current system date and time
/// NOTE: The memory for the returned string needs to be freed with the free_string function
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub unsafe extern fn pactffi_generate_datetime_string(format: *const c_char) -> StringResult {
  if format.is_null() {
    let error = CString::new("generate_datetime_string: format is NULL").unwrap();
    StringResult::Failed(error.into_raw())
  } else {
    let c_str = CStr::from_ptr(format);
    match c_str.to_str() {
      Ok(s) => match parse_pattern(s) {
        Ok(pattern_tokens) => {
          let result = Local::now().format(to_chrono_pattern(&pattern_tokens).as_str()).to_string();
          let result_str = CString::new(result.as_str()).unwrap();
          StringResult::Ok(result_str.into_raw())
        },
        Err(err) => {
          let error = format!("Error parsing '{}': {:?}", s, err);
          let error_str = CString::new(error.as_str()).unwrap();
          StringResult::Failed(error_str.into_raw())
        }
      },
      Err(err) => {
        let error = format!("generate_datetime_string: format is not a valid UTF-8 string: {:?}", err);
        let error_str = CString::new(error.as_str()).unwrap();
        StringResult::Failed(error_str.into_raw())
      }
    }
  }
}

/// Checks that the example string matches the given regex
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub unsafe extern fn pactffi_check_regex(regex: *const c_char, example: *const c_char) -> bool {
  if regex.is_null() {
    false
  } else {
    let c_str = CStr::from_ptr(regex);
    match c_str.to_str() {
      Ok(regex) => {
        let example = convert_cstr("example", example).unwrap_or_default();
        match Regex::new(regex) {
          Ok(re) => re.is_match(example),
          Err(err) => {
            error!("check_regex: '{}' is not a valid regular expression - {}", regex, err);
            false
          }
        }
      },
      Err(err) => {
        error!("check_regex: regex is not a valid UTF-8 string: {:?}", err);
        false
      }
    }
  }
}

/// Generates an example string based on the provided regex.
pub fn generate_regex_value_internal(regex: &str) -> Result<String, String> {
  let mut parser = regex_syntax::ParserBuilder::new().unicode(false).build();
  match parser.parse(regex) {
    Ok(hir) => {
      let mut rnd = rand::thread_rng();
      let gen = rand_regex::Regex::with_hir(hir, 20).unwrap();
      let result: String = rnd.sample(gen);
      Ok(result)
    },
    Err(err) => {
      let error = format!("generate_regex_value: '{}' is not a valid regular expression - {}", regex, err);
      Err(error)
    }
  }
}

/// Generates an example string based on the provided regex.
/// NOTE: The memory for the returned string needs to be freed with the free_string function
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
pub unsafe extern fn pactffi_generate_regex_value(regex: *const c_char) -> StringResult {
  if regex.is_null() {
    let error = CString::new("generate_regex_value: regex is NULL").unwrap();
    StringResult::Failed(error.into_raw())
  } else {
    let c_str = CStr::from_ptr(regex);
    match c_str.to_str() {
      Ok(regex) => match generate_regex_value_internal(regex) {
        Ok(val) => {
          let result_str = CString::new(val.as_str()).unwrap();
          StringResult::Ok(result_str.into_raw())
        },
        Err(err) => {
          let error = CString::new(err).unwrap();
          StringResult::Failed(error.into_raw())
        }
      },
      Err(err) => {
        let error = CString::new(format!("generate_regex_value: regex is not a valid UTF-8 string: {:?}", err)).unwrap();
        StringResult::Failed(error.into_raw())
      }
    }
  }
}

/// [DEPRECATED] Frees the memory allocated to a string by another function
///
/// This function is deprecated. Use pactffi_string_delete instead.
///
/// # Safety
///
/// Exported functions are inherently unsafe.
#[no_mangle]
#[deprecated(since = "0.1.0", note = "Use pactffi_string_delete instead")]
pub unsafe extern fn pactffi_free_string(s: *mut c_char) {
  if s.is_null() {
    return;
  }
  CString::from_raw(s);
}

