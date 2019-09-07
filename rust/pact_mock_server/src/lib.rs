//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! The exported functions using C bindings for controlling the mock server now live in the `pact_mock_server_ffi`
//! crate.

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

#[cfg(test)]
#[macro_use]
extern crate maplit;

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod hyper_server;

use pact_matching::models::Pact;
use pact_matching::s;
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
  pub static ref MANAGER: Mutex<Option<ServerManager>> = Mutex::new(Option::None);
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
pub fn start_mock_server(id: String, pact: Pact, port: i32) -> Result<i32, String> {
    MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .start_mock_server(id, pact, port as u16)
        .map(|port| port as i32)
}

/// Creates a mock server. Requires the pact JSON as a string as well as the port for the mock
/// server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
pub extern fn create_mock_server(pact_json: &str, port: i32) -> Result<i32, MockServerError> {
  match serde_json::from_str(pact_json) {
    Ok(pact_json) => {
      let pact = Pact::from_json(&s!("<create_mock_server>"), &pact_json);
      start_mock_server(Uuid::new_v4().simple().to_string(), pact, port)
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

/// Write Pact File Errors
pub enum WritePactFileErr {
  /// IO Error occurred
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

#[cfg(test)]
#[macro_use(expect)]
extern crate expectest;

#[cfg(test)]
extern crate quickcheck;

#[cfg(test)]
mod tests;
