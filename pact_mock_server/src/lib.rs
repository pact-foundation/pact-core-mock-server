//! The `pact_mock_server` crate provides the in-process mock server for mocking HTTP requests
//! and generating responses based on a pact file. It implements the
//! [V3 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-3)
//! and [V4 Pact specification](https://github.com/pact-foundation/pact-specification/tree/version-4).
//!
//! ## Crate features
//! All features are enabled by default
//!
//! * `datetime`: Enables support of date and time expressions and generators.
//! * `xml`: Enables support for parsing XML documents.
//! * `plugins`: Enables support for using plugins.
//! * `multipart`: Enables support for MIME multipart bodies.
//! * `tls`: Enables support for mock servers using TLS. This will add the following dependencies: hyper-rustls, rustls, rustls-pemfile, tokio-rustls.

#![warn(missing_docs)]

use std::sync::Mutex;

#[cfg(feature = "plugins")] use maplit::hashmap;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::{
  CatalogueEntry,
  CatalogueEntryProviderType,
  CatalogueEntryType,
  register_core_entries
};
#[allow(unused_imports)] use tracing::{error, info, warn};

use crate::server_manager::ServerManager;

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod utils;
pub mod legacy;
pub mod builder;
pub mod hyper_server;

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

lazy_static::lazy_static! {
  ///
  /// A global thread-safe, "init-on-demand" reference to a server manager.
  /// When the server manager is initialized, it starts a separate thread on which
  /// to serve requests.
  ///
  pub static ref MANAGER: Mutex<Option<ServerManager>> = Mutex::new(None);
}

#[cfg(feature = "plugins")]
lazy_static::lazy_static! {
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
  #[cfg(feature = "plugins")] register_core_entries(MOCK_SERVER_CATALOGUE_ENTRIES.as_ref());
}

/// Write Pact File Errors
pub enum WritePactFileErr {
  /// IO Error occurred
  IOError,
  /// No mock server was running on the port
  NoMockServer
}

#[cfg(test)]
mod tests;
