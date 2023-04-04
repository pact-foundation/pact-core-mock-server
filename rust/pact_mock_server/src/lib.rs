//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the V3 Pact specification
//! (https://github.com/pact-foundation/pact-specification/tree/version-3).
//!
//! The exported functions using C bindings for controlling the mock server now live in the `pact_mock_server_ffi`
//! crate.

#![warn(missing_docs)]

use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::anyhow;
use itertools::{Either, Itertools};
use lazy_static::*;
use maplit::hashmap;
use pact_models::pact::{load_pact_from_json, Pact, ReadWritePact, write_pact};
use pact_models::PactSpecification;
use pact_plugin_driver::catalogue_manager;
use pact_plugin_driver::catalogue_manager::{
  CatalogueEntry,
  CatalogueEntryProviderType,
  CatalogueEntryType,
  register_core_entries
};
use pact_plugin_driver::plugin_manager::get_mock_server_results;
use rustls::ServerConfig;
use serde_json::json;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::mock_server::{MockServer, MockServerConfig};
use crate::server_manager::{PluginMockServer, ServerManager};

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod hyper_server;
pub mod tls;
mod utils;

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

  /// Mock server entries to add to the plugin catalogue
  static ref MOCK_SERVER_CATALOGUE_ENTRIES: Vec<CatalogueEntry> = {
    let mut entries = vec![];
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::TRANSPORT,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "http".to_string(),
      values: hashmap!{}
    });
    entries.push(CatalogueEntry {
      entry_type: CatalogueEntryType::TRANSPORT,
      provider_type: CatalogueEntryProviderType::CORE,
      plugin: None,
      key: "https".to_string(),
      values: hashmap!{}
    });
    entries
  };
}

/// Sets up all the core catalogue entries for mock servers
pub fn configure_core_catalogue() {
  register_core_entries(MOCK_SERVER_CATALOGUE_ENTRIES.as_ref());
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
  pact: Box<dyn Pact + Send + Sync>,
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
  pact: Box<dyn Pact + Send + Sync>,
  addr: std::net::SocketAddr,
  config: MockServerConfig
) -> Result<i32, String> {
  configure_core_catalogue();
  pact_matching::matchers::configure_core_catalogue();

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
  pact: Box<dyn Pact + Send + Sync>,
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
  pact: Box<dyn Pact + Send + Sync>,
  addr: std::net::SocketAddr,
  tls: &ServerConfig,
  config: MockServerConfig
) -> Result<i32, String> {
  configure_core_catalogue();
  pact_matching::matchers::configure_core_catalogue();

  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .start_tls_mock_server_with_addr(id, pact, addr, tls, config)
    .map(|addr| addr.port() as i32)
}

/// Starts a mock server for the provided transport. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// * `id` - Unique ID for the mock server.
/// * `pact` - Pact model to use for the mock server.
/// * `addr` - Socket address that the server should listen on.
/// * `transport` - Transport to use for the mock server.
/// * `config` - Configuration for the mock server. Transport specific configuration must be specified in the `transport_config` field.
///
/// # Errors
///
/// An error will be returned if the mock server is not able to be started or the transport is not known.
pub fn start_mock_server_for_transport(
  id: String,
  pact: Box<dyn Pact + Send + Sync>,
  addr: std::net::SocketAddr,
  transport: &str,
  config: MockServerConfig
) -> anyhow::Result<i32> {
  configure_core_catalogue();
  pact_matching::matchers::configure_core_catalogue();

  let key = format!("transport/{}", transport);
  let transport_entry = catalogue_manager::lookup_entry(key.as_str())
    .ok_or_else(|| anyhow!("Transport '{}' is not a known transport", transport))?;

  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .start_mock_server_for_transport(id, pact, addr, &transport_entry, config)
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
  configure_core_catalogue();
  pact_matching::matchers::configure_core_catalogue();

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
///
/// Note that for mock servers provided by plugins, if the call to the plugin fails, a value of false
/// will also be returned.
pub fn mock_server_matched(mock_server_port: i32) -> bool {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .find_mock_server_by_port(mock_server_port as u16, &|server_manager, _, mock_server| {
      match mock_server {
        Either::Left(mock_server) => mock_server.mismatches().is_empty(),
        Either::Right(plugin_mock_server) => {
          let results = server_manager.exec_async(get_mock_server_results(&plugin_mock_server.mock_server_details));
          match results {
            Ok(results) => results.is_empty(),
            Err(err) => {
              error!("Request to plugin to get matching results failed - {}", err);
              false
            }
          }
        }
      }
    })
    .unwrap_or(false)
}

/// Gets all the mismatches from a mock server in JSON format. The port number of the mock
/// server is passed in, and the results are returned in JSON format as a String.
///
/// If there is no mock server with the provided port number, `None` is returned.
///
/// For mock servers provided by plugins, if the call to the plugin fails, a JSON value with an
/// error attribute will be returned.
pub fn mock_server_mismatches(mock_server_port: i32) -> Option<String> {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .find_mock_server_by_port(mock_server_port as u16, &|manager, _, mock_server| {
      match mock_server {
        Either::Left(mock_server) => {
          let mismatches = mock_server.mismatches().iter()
            .map(|mismatch| mismatch.to_json() )
            .collect::<Vec<serde_json::Value>>();
          json!(mismatches).to_string()
        }
        Either::Right(plugin_mock_server) => {
          let results = manager.exec_async(get_mock_server_results(&plugin_mock_server.mock_server_details));
          match results {
            Ok(results) => {
              json!(results.iter().map(|item| {
                json!({
                  "path": item.path,
                  "error": item.error,
                  "mismatches": item.mismatches.iter().map(|mismatch| {
                    json!({
                      "expected": mismatch.expected,
                      "actual": mismatch.actual,
                      "mismatch": mismatch.mismatch,
                      "path": mismatch.path,
                      "diff": mismatch.diff.clone().unwrap_or_default()
                    })
                  }).collect_vec()
                })
              }).collect_vec())
            },
            Err(err) => {
              error!("Request to plugin to get matching results failed - {}", err);
              json!({ "error": format!("Request to plugin to get matching results failed - {}", err) })
            }
          }.to_string()
        }
      }
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
        .find_mock_server_by_port(mock_server_port as u16, &|_, _, ms| {
          match ms {
            Either::Left(mock_server) => {
              mock_server.write_pact(&directory, overwrite)
                .map(|_| ())
                .map_err(|err| {
                  error!("Failed to write pact to file - {}", err);
                  WritePactFileErr::IOError
                })
            }
            Either::Right(plugin_mock_server) => {
              let mut pact = plugin_mock_server.pact.clone();
              pact.add_md_version("mockserver", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
              let pact_file_name = pact.default_file_name();
              let filename = match directory {
                Some(ref path) => {
                  let mut path = PathBuf::from(path);
                  path.push(pact_file_name);
                  path
                },
                None => PathBuf::from(pact_file_name)
              };

              info!("Writing pact out to '{}'", filename.display());
              match write_pact(pact.boxed(), filename.as_path(), PactSpecification::V4, overwrite) {
                Ok(_) => Ok(()),
                Err(err) => {
                  warn!("Failed to write pact to file - {}", err);
                  Err(WritePactFileErr::IOError)
                }
              }
            }
          }
        });

    match opt_result {
        Some(result) => result,
        None => {
            error!("No mock server running on port {}", mock_server_port);
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

/// Find a mock server by port number and and map it using supplied function if found. Returns the
/// result of the function call wrapped in a Some. Returns a None if the mock server was not found.
pub fn find_mock_server_by_port<R>(
  port: u16,
  f: &dyn Fn(&ServerManager, &String, Either<&MockServer, &PluginMockServer>) -> R
) -> Option<R> {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .find_mock_server_by_port(port, f)
}

/// Shuts down the mock server with the provided ID. Returns a boolean value to indicate if
/// the mock server was successfully shut down.
pub fn shutdown_mock_server_by_id(id: &str) -> bool {
  MANAGER.lock().unwrap()
    .get_or_insert_with(ServerManager::new)
    .shutdown_mock_server_by_id(id.to_string())
}

#[cfg(test)]
mod tests;
