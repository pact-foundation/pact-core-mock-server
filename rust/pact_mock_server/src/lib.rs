//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! There are a number of exported functions using C bindings for controlling the mock server. These can be used in any
//! language that supports C bindings.
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

#[cfg_attr(test, macro_use)] extern crate pact_matching;
extern crate serde_json;
extern crate hyper;
extern crate futures;
extern crate tokio;
#[macro_use] extern crate log;
extern crate uuid;
extern crate itertools;
#[macro_use] extern crate lazy_static;
extern crate libc;

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod hyper_server;

use pact_matching::models::Pact;
use pact_matching::s;
use libc::{c_char};
use std::ffi::CStr;
use std::ffi::CString;
use std::str;
use std::panic::catch_unwind;
use std::sync::Mutex;
use serde_json::json;
use uuid::Uuid;
use server_manager::ServerManager;

/// Mock server errors
pub enum MockServerError {
  /// Invalid Pact Json
  InvalidPactJson,
  /// Failed to start the mock server
  MockServerFailedToStart
}

lazy_static! {
  ///
  /// A global thread-safe, "init-on-demand" reference to a server manager.
  /// When the server manager is initialized, it starts a separate thread on which
  /// to serve requests.
  ///
  static ref MANAGER: Mutex<Option<ServerManager>> = Mutex::new(Option::None);
}

/// Starts a mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_mock_server(id: String, pact: Pact, addr: std::net::SocketAddr) -> Result<i32, String> {
    MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .start_mock_server_with_addr(id, pact, addr)
        .map(|addr| addr.port() as i32)
}

/// Creates a mock server. Requires the pact JSON as a string as well as the port for the mock
/// server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
pub extern fn create_mock_server(pact_json: &str, addr: std::net::SocketAddr) -> Result<i32, MockServerError> {
  match serde_json::from_str(pact_json) {
    Ok(pact_json) => {
      let pact = Pact::from_json(&s!("<create_mock_server>"), &pact_json);
      start_mock_server(Uuid::new_v4().simple().to_string(), pact, addr)
        .map_err(|err| {
          error!("Could not start mock server: {}", err);
          MockServerError::MockServerFailedToStart
        })
    },
    Err(err) => {
      error!("Could not parse pact json: {}", err);
      Err(MockServerError::InvalidPactJson)
    }
  }
}

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
/// | -5 | The address is not valid |
///
#[no_mangle]
pub extern fn create_mock_server_ffi(pact_str: *const c_char, addr_str: *const c_char) -> i32 {
    env_logger::init().unwrap_or(());

    let result = catch_unwind(|| {
        let c_str = unsafe {
            if pact_str.is_null() {
                error!("Got a null pointer instead of pact json");
                return -1;
            }
            CStr::from_ptr(pact_str)
        };

        let addr_c_str = unsafe {
            if addr_str.is_null() {
                error!("Got a null pointer instead of listener address");
                return -1;
            }
            CStr::from_ptr(addr_str)
        };

        if let Ok(Ok(addr)) = str::from_utf8(addr_c_str.to_bytes()).map(|s| s.parse::<std::net::SocketAddr>()) {
          match create_mock_server(str::from_utf8(c_str.to_bytes()).unwrap(), addr) {
            Ok(ms_port) => ms_port,
            Err(err) => match err {
              MockServerError::InvalidPactJson => -2,
              MockServerError::MockServerFailedToStart => -3
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
            error!("Caught a general panic: {:?}", cause);
            -4
        }
    }
}

/// Function to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched.
pub extern fn mock_server_matched(mock_server_port: i32) -> bool {
    MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
            mock_server.mismatches().is_empty()
        })
        .unwrap_or(false)
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

/// Gets all the mismatches from a mock server. The port number of the mock
/// server is passed in, and the results are returned in JSON format as a String.
///
/// If there is no mock server with the provided port number, `None` is returned.
///
pub extern fn mock_server_mismatches(mock_server_port: i32) -> Option<std::string::String> {
    MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
            let mismatches = mock_server.mismatches().iter()
            .map(|mismatch| mismatch.to_json() )
            .collect::<Vec<serde_json::Value>>();
            json!(mismatches).to_string()
        })
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

/// Write Pact File Errors
pub enum WritePactFileErr {
  /// IO Error occured
  IOError,
  /// No mock server was running on the port
  NoMockServer
}

/// Trigger a mock server to write out its pact file. This function should
/// be called if all the consumer tests have passed. The directory to write the file to is passed
/// as the second parameter. If `None` is passed in, the current working directory is used.
///
/// Returns `Ok` if the pact file was successfully written. Returns an `Err` if the file can
/// not be written, or there is no mock server running on that port.
pub extern fn write_pact_file(mock_server_port: i32, directory: Option<String>) -> Result<(), WritePactFileErr> {
    let opt_result = MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
            mock_server.write_pact(&directory)
                .map(|_| ())
                .map_err(|err| {
                    error!("Failed to write pact to file - {}", err);
                    WritePactFileErr::IOError
                })
        });

    match opt_result {
        Some(result) => result,
        None => {
            error!("No mock server running on port {}", mock_server_port);
            Err(WritePactFileErr::NoMockServer)
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
          Some(s!(dir_str))
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

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
mod tests;
