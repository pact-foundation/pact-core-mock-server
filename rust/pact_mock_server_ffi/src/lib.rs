//! The `pact_mock_server` crate provides a number of exported functions using C bindings for
//! controlling the mock server. These can be used in any language that supports C bindings.
//!
//! ## [create_mock_server_ffi](fn.create_mock_server_ffi.html)
//!
//! External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
//! as well as the port for the mock server to run on. A value of 0 for the port will result in a
//! port being allocated by the operating system. The port of the mock server is returned.
//!
//! ## [mock_server_matched_ffi](fn.mock_server_matched_ffi.html)
//!
//! Simple function that returns a boolean value given the port number of the mock service. This value will be true if all
//! the expectations of the pact that the mock server was created with have been met. It will return false if any request did
//! not match, an un-recognised request was received or an expected request was not received.
//!
//! ## [mock_server_mismatches_ffi](fn.mock_server_mismatches_ffi.html)
//!
//! This returns all the mismatches, un-expected requests and missing requests in JSON format, given the port number of the
//! mock server.
//!
//! **IMPORTANT NOTE:** The JSON string for the result is allocated on the rust heap, and will have to be freed once the
//! code using the mock server is complete. The [`cleanup_mock_server_ffi`](fn.cleanup_mock_server_ffi.html) function is provided for this purpose. If the mock
//! server is not cleaned up properly, this will result in memory leaks as the rust heap will not be reclaimed.
//!
//! ## [cleanup_mock_server_ffi](fn.cleanup_mock_server_ffi.html)
//!
//! This function will try terminate the mock server with the given port number and cleanup any memory allocated for it by
//! the [`mock_server_mismatches_ffi`](fn.mock_server_mismatches_ffi.html) function. Returns `true`, unless a mock server with the given port number does not exist,
//! or the function fails in some way.
//!
//! **NOTE:** Although `close()` on the listerner for the mock server is called, this does not currently work and the
//! listerner will continue handling requests. In this case, it will always return a 501 once the mock server has been
//! cleaned up.
//!
//! ## [write_pact_file_ffi](fn.write_pact_file_ffi.html)
//!
//! External interface to trigger a mock server to write out its pact file. This function should
//! be called if all the consumer tests have passed. The directory to write the file to is passed
//! as the second parameter. If a NULL pointer is passed, the current working directory is used.
//!
//! Returns 0 if the pact file was successfully written. Returns a positive code if the file can
//! not be written, or there is no mock server running on that port or the function panics.

#![warn(missing_docs)]

extern crate pact_mock_server;
extern crate libc;
extern crate serde_json;
extern crate env_logger;
#[macro_use] extern crate log;

use std::panic::catch_unwind;
use libc::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use serde_json::json;
use pact_mock_server::{
  create_mock_server,
  MockServerError,
  mock_server_matched,
  write_pact_file,
  WritePactFileErr,
  MANAGER
};
use pact_mock_server::server_manager::ServerManager;

/// External interface to create a mock server. A pointer to the pact JSON as a C string is passed in,
/// as well as the port for the mock server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
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
///
#[no_mangle]
pub extern fn create_mock_server_ffi(pact_str: *const c_char, port: i32) -> i32 {
  env_logger::init();

  let result = catch_unwind(|| {
    let c_str = unsafe {
      if pact_str.is_null() {
        error!("Got a null pointer instead of pact json");
        return -1;
      }
      CStr::from_ptr(pact_str)
    };

    match create_mock_server(str::from_utf8(c_str.to_bytes()).unwrap(), port) {
      Ok(ms_port) => ms_port,
      Err(err) => match err {
        MockServerError::InvalidPactJson => -2,
        MockServerError::MockServerFailedToStart => -3
      }
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      error!("Caught a general panic: {:?}", cause);
      -4
    }
  }
}

/// External interface to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched, or
/// the method panics.
#[no_mangle]
pub extern fn mock_server_matched_ffi(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    mock_server_matched(mock_server_port)
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
pub extern fn mock_server_mismatches_ffi(mock_server_port: i32) -> *mut c_char {
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
      None => 0 as *mut _
    }
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      error!("Caught a general panic: {:?}", cause);
      0 as *mut _
    }
  }
}

/// External interface to cleanup a mock server. This function will try terminate the mock server
/// with the given port number and cleanup any memory allocated for it. Returns true, unless a
/// mock server with the given port number does not exist, or the function panics.
///
/// **NOTE:** Although `close()` on the listener for the mock server is called, this does not
/// currently work and the listener will continue handling requests. In this
/// case, it will always return a 404 once the mock server has been cleaned up.
#[no_mangle]
pub extern fn cleanup_mock_server_ffi(mock_server_port: i32) -> bool {
  let result = catch_unwind(|| {
    MANAGER.lock().unwrap()
      .get_or_insert_with(ServerManager::new)
      .shutdown_mock_server_by_port(mock_server_port as u16)
  });

  match result {
    Ok(val) => val,
    Err(cause) => {
      error!("Caught a general panic: {:?}", cause);
      false
    }
  }
}

/// External interface to trigger a mock server to write out its pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If a NULL pointer is passed, the current working directory is used.
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
pub extern fn write_pact_file_ffi(mock_server_port: i32, directory: *const c_char) -> i32 {
  let result = catch_unwind(|| {
    let dir = unsafe {
      if directory.is_null() {
        warn!("Directory to write to is NULL, defaulting to the current working directory");
        None
      } else {
        let c_str = CStr::from_ptr(directory);
        let dir_str = str::from_utf8(c_str.to_bytes()).unwrap();
        if dir_str.is_empty() {
          None
        } else {
          Some(dir_str.to_string())
        }
      }
    };

    write_pact_file(mock_server_port, dir)
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
      error!("Caught a general panic: {:?}", cause);
      1
    }
  }
}
