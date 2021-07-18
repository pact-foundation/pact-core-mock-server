//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! The exported functions using C bindings for controlling the mock server now live in the `pact_mock_server_ffi`
//! crate.

#![warn(missing_docs)]

use std::sync::Mutex;

use lazy_static::*;
use log::*;
use rustls::ServerConfig;
use serde_json::json;
use uuid::Uuid;

use pact_models::pact::{load_pact_from_json, Pact};

use crate::mock_server::MockServerConfig;
use crate::server_manager::ServerManager;

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod hyper_server;
pub mod tls;

/// Mock server errors
#[derive(thiserror::Error, Debug)]
pub enum MockServerError {
  /// Invalid Pact Json
  #[error("Invalid Pact JSON")]
  InvalidPactJson,
  /// Failed to start the mock server
  #[error("Failed to start the mock server")]
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
/// * `id` - Unique ID for the mock server.
/// * `pact` - Pact model to use for the mock server.
/// * `addr` - Socket address that the server should listen on.
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_mock_server(
  id: String,
  pact: Box<dyn Pact>,
  addr: std::net::SocketAddr
) -> Result<i32, String> {
  start_mock_server_with_config(id, pact, addr, MockServerConfig::default())
}

/// Starts a mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// * `id` - Unique ID for the mock server.
/// * `pact` - Pact model to use for the mock server.
/// * `addr` - Socket address that the server should listen on.
/// * `config` - Configuration for the mock server
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_mock_server_with_config(
  id: String,
  pact: Box<dyn Pact>,
  addr: std::net::SocketAddr,
  config: MockServerConfig
) -> Result<i32, String> {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .start_mock_server_with_addr(id, pact, addr, config)
    .map(|addr| addr.port() as i32)
}

/// Starts a TLS mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// * `id` - Unique ID for the mock server.
/// * `pact` - Pact model to use for the mock server.
/// * `addr` - Socket address that the server should listen on.
/// * `tls` - TLS config.
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_tls_mock_server(
  id: String,
  pact: Box<dyn Pact>,
  addr: std::net::SocketAddr,
  tls: &ServerConfig
) -> Result<i32, String> {
  start_tls_mock_server_with_config(id, pact, addr, tls, MockServerConfig::default())
}

/// Starts a TLS mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// * `id` - Unique ID for the mock server.
/// * `pact` - Pact model to use for the mock server.
/// * `addr` - Socket address that the server should listen on.
/// * `tls` - TLS config.
/// * `config` - Configuration for the mock server
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_tls_mock_server_with_config(
  id: String,
  pact: Box<dyn Pact>,
  addr: std::net::SocketAddr,
  tls: &ServerConfig,
  config: MockServerConfig
) -> Result<i32, String> {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .start_tls_mock_server_with_addr(id, pact, addr, tls, config)
    .map(|addr| addr.port() as i32)
}

/// Creates a mock server. Requires the pact JSON as a string as well as the port for the mock
/// server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// * `pact_json` - Pact in JSON format
/// * `addr` - Socket address to listen on
pub fn create_mock_server(
  pact_json: &str,
  addr: std::net::SocketAddr
) -> anyhow::Result<i32> {
  match serde_json::from_str(pact_json) {
    Ok(pact_json) => {
      let pact = load_pact_from_json("<create_mock_server>", &pact_json)?;
      start_mock_server(Uuid::new_v4().to_string(), pact, addr)
        .map_err(|err| {
          error!("Could not start mock server: {}", err);
          MockServerError::MockServerFailedToStart.into()
        })
    },
    Err(err) => {
      error!("Could not parse pact json: {}", err);
      Err(MockServerError::InvalidPactJson.into())
    }
  }
}

/// Creates a TLS mock server. Requires the pact JSON as a string as well as the port for the mock
/// server to run on. A value of 0 for the port will result in a
/// port being allocated by the operating system. The port of the mock server is returned.
///
/// * `pact_json` - Pact in JSON format
/// * `addr` - Socket address to listen on
/// * `tls` - TLS config
pub fn create_tls_mock_server(
  pact_json: &str,
  addr: std::net::SocketAddr,
  tls: &ServerConfig
) -> anyhow::Result<i32> {
  match serde_json::from_str(pact_json) {
    Ok(pact_json) => {
      let pact = load_pact_from_json("<create_mock_server>", &pact_json)?;
      start_tls_mock_server(Uuid::new_v4().to_string(), pact, addr, tls)
        .map_err(|err| {
          error!("Could not start mock server: {}", err);
          MockServerError::MockServerFailedToStart.into()
        })
    },
    Err(err) => {
      error!("Could not parse pact json: {}", err);
      Err(MockServerError::InvalidPactJson.into())
    }
  }
}

/// Function to check if a mock server has matched all its requests. The port number is
/// passed in, and if all requests have been matched, true is returned. False is returned if there
/// is no mock server on the given port, or if any request has not been successfully matched.
pub fn mock_server_matched(mock_server_port: i32) -> bool {
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
pub fn mock_server_mismatches(mock_server_port: i32) -> Option<std::string::String> {
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
/// If overwrite is true, the file will be overwritten with the contents of the current pact.
/// Otherwise it will be merged with any existing pact file.
///
/// Returns `Ok` if the pact file was successfully written. Returns an `Err` if the file can
/// not be written, or there is no mock server running on that port.
pub fn write_pact_file(
  mock_server_port: i32,
  directory: Option<String>,
  overwrite: bool
) -> Result<(), WritePactFileErr> {
    let opt_result = MANAGER.lock().unwrap()
        .get_or_insert_with(ServerManager::new)
        .find_mock_server_by_port_mut(mock_server_port as u16, &|mock_server| {
            mock_server.write_pact(&directory, overwrite)
                .map(|_| ())
                .map_err(|err| {
                    log::error!("Failed to write pact to file - {}", err);
                    WritePactFileErr::IOError
                })
        });

    match opt_result {
        Some(result) => result,
        None => {
            log::error!("No mock server running on port {}", mock_server_port);
            Err(WritePactFileErr::NoMockServer)
        }
    }
}

/// Shuts down the mock server with the provided port. Returns a boolean value to indicate if
/// the mock server was successfully shut down.
pub fn shutdown_mock_server(mock_server_port: i32) -> bool {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .shutdown_mock_server_by_port(mock_server_port as u16)
}

#[cfg(test)]
mod tests;
