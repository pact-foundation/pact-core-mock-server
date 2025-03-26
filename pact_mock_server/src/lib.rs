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
//!
//! ## Creating a mock server
//! Mock servers can be created by using the mock server builder in the `builder` package. The
//! builder can create both standard HTTP and HTTPS servers.
//!
//! The following example loads a Pact file, starts the mock server and then shuts it down later.
//! ```rust
//! # tokio_test::block_on(async {
//! use pact_models::prelude::{Pact, RequestResponsePact};
//! use pact_mock_server::builder::MockServerBuilder;
//!
//! // Setup a Pact file for the mock server
//! let pact_json = r#"
//!     {
//!         "provider": {
//!             "name": "Example Provider"
//!         },
//!         "consumer": {
//!             "name": "Example Consumer"
//!         },
//!         "interactions": [
//!           {
//!               "description": "a GET request",
//!               "request": {
//!                 "method": "GET",
//!                 "path": "/path"
//!               },
//!               "response": {
//!                 "status": 200,
//!                 "headers": {
//!                   "Content-Type": "text/plain"
//!                 },
//!                 "body": "Hello from the mock server"
//!               }
//!           }
//!         ]
//!     }
//!     "#;
//!  let pact = RequestResponsePact::from_json(&"JSON sample".to_string(), &serde_json::from_str(pact_json)?)?;
//!
//! // Create the mock server. Note that the async version requires a Tokio runtime.
//! let mut mock_server = MockServerBuilder::new()
//!   .bind_to("127.0.0.1:0")
//!   .with_pact(pact.boxed())
//!   .start()
//!   .await?;
//!
//! // We can now make any requests to the mock server
//! let http_client = reqwest::Client::new();
//! let response = http_client.get(format!("http://127.0.0.1:{}/path", mock_server.port()).as_str())
//!   .send()
//!   .await?;
//! assert_eq!(response.text().await?, "Hello from the mock server");
//!
//! // Shut the mock server down. This will dispose of the running background tasks.
//! mock_server.shutdown()?;
//!
//! // Finally we can now check the status of the mock server.
//! assert_eq!(mock_server.all_matched(), true);
//!
//! # Ok::<(), anyhow::Error>(())
//! # });
//! ```

#![warn(missing_docs)]

#[doc = include_str!("../README.md")]
#[cfg(doctest)]
struct Readme;

use std::sync::Mutex;

#[cfg(feature = "plugins")] use maplit::hashmap;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::{
  CatalogueEntry,
  CatalogueEntryProviderType,
  CatalogueEntryType,
  register_core_entries
};
use tokio::task_local;
#[allow(unused_imports)] use tracing::{error, info, warn};

use crate::server_manager::ServerManager;

pub mod matching;
pub mod mock_server;
pub mod server_manager;
mod utils;
pub mod legacy;
pub mod builder;
pub mod hyper_server;

task_local! {
  /// Log ID to accumulate logs against
  #[allow(missing_docs)]
  #[deprecated(note = "This must be moved to the FFI crate")]
  pub static LOG_ID: String;
}

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
  #[deprecated(since = "2.0.0", note = "Crates that require a static manager should setup one themselves")]
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
