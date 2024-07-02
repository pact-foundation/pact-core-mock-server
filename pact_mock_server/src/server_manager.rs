//!
//! This module defines a manager for holding multiple instances of mock servers.
//!

use std::collections::BTreeMap;
use std::ffi::CString;
#[cfg(feature = "plugins")] use std::future::Future;
use std::net::SocketAddr;
#[cfg(feature = "plugins")] use std::net::ToSocketAddrs;

use anyhow::anyhow;
use itertools::Either;
#[cfg(feature = "plugins")] use maplit::hashmap;
use pact_models::pact::Pact;
#[cfg(feature = "plugins")] use pact_models::prelude::v4::V4Pact;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::{CatalogueEntry, CatalogueEntryProviderType};
#[cfg(feature = "plugins")] use pact_plugin_driver::mock_server::MockServerDetails;
#[cfg(feature = "tls")] use rustls::ServerConfig;
#[cfg(not(feature = "plugins"))] use serde::{Deserialize, Serialize};
use tracing::{debug, error, trace};
#[cfg(feature = "plugins")] use url::Url;
use crate::builder::MockServerBuilder;

use crate::mock_server::{MockServer, MockServerConfig};

/// Mock server that has been provided by a plugin
#[derive(Debug, Clone)]
#[cfg(feature = "plugins")]
pub struct PluginMockServer {
  /// Details of the running mock server
  pub mock_server_details: MockServerDetails,
  /// Catalogue entry for the transport
  pub catalogue_entry: CatalogueEntry,
  /// Pact for this mock server
  pub pact: V4Pact
}

/// Mock server that has been provided by a plugin (dummy struct)
#[derive(Debug, Clone)]
#[cfg(not(feature = "plugins"))]
pub struct PluginMockServer {}

/// Dummy Catalogue entry
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[cfg(not(feature = "plugins"))]
pub struct CatalogueEntry {}

struct ServerEntry {
  /// Either a local mock server or a plugin provided one
  mock_server: Either<MockServer, PluginMockServer>,
  /// Port the mock server is running on
  port: u16,
  /// List of resources that need to be cleaned up when the mock server completes
  pub resources: Vec<CString>
}

/// Struct to represent many mock servers running in a background thread
pub struct ServerManager {
    runtime: tokio::runtime::Runtime,
    mock_servers: BTreeMap<String, ServerEntry>,
}

impl ServerManager {
  /// Construct a new ServerManager for scheduling several instances of mock servers
  /// on one tokio runtime.
  pub fn new() -> ServerManager {
    ServerManager {
      runtime: tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap(),
      mock_servers: BTreeMap::new()
    }
  }

  /// Consumes the mock server builder, and then spawns the resulting mock server on the server
  /// manager's runtime. Note that this function will block the current calling thread.
  pub fn spawn_mock_server(&mut self, builder: MockServerBuilder) -> anyhow::Result<MockServer> {
    #[allow(unused_assignments)]
    let mut mock_server = MockServer::default();

    #[cfg(feature = "tls")]
    {
      mock_server = if builder.tls_configured() {
        self.runtime.block_on(builder.start_https())
      } else {
        self.runtime.block_on(builder.start())
      }?;
    }

    #[cfg(not(feature = "tls"))]
    {
      mock_server = self.runtime.block_on(builder.start())?;
    }

    let mock_server_id = mock_server.id.clone();
    let port = mock_server.port();
    let ms_clone = mock_server.clone();
    self.mock_servers.insert(
      mock_server_id,
      ServerEntry {
        mock_server: Either::Left(mock_server),
        port,
        resources: vec![]
      },
    );

    Ok(ms_clone)
  }

    /// Start a new server on the runtime
    #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
    pub fn start_mock_server_with_addr(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      addr: SocketAddr,
      config: MockServerConfig
    ) -> anyhow::Result<SocketAddr> {
      let mock_server = self.runtime.block_on(MockServerBuilder::new()
          .with_pact(pact)
          .with_config(config)
          .bind_to(addr.to_string())
          .with_id(id.as_str())
          .start())?;

      let port = mock_server.address.port();
      self.mock_servers.insert(
        id,
        ServerEntry {
          mock_server: Either::Left(mock_server),
          port,
          resources: vec![]
        },
      );

      Ok(SocketAddr::new(addr.ip(), port))
    }

    /// Start a new TLS server on the runtime
    #[cfg(feature = "tls")]
    #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
    pub fn start_tls_mock_server_with_addr(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      addr: SocketAddr,
      tls_config: &ServerConfig,
      config: MockServerConfig
    ) -> anyhow::Result<SocketAddr> {
      let mock_server = self.runtime.block_on(MockServerBuilder::new()
        .with_pact(pact)
        .with_config(config)
        .with_tls_config(tls_config)
        .bind_to(addr.to_string())
        .with_id(id.as_str())
        .start_https())?;

      let port = mock_server.address.port();
      self.mock_servers.insert(
        id,
        ServerEntry {
          mock_server: Either::Left(mock_server),
          port,
          resources: vec![]
        },
      );

      Ok(SocketAddr::new(addr.ip(), port))
    }

    /// Start a new server on the runtime
    #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
    pub fn start_mock_server(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      port: u16,
      config: MockServerConfig
    ) -> anyhow::Result<u16> {
      #[allow(deprecated)]
      self.start_mock_server_with_addr(id, pact, ([0, 0, 0, 0], port as u16).into(), config)
            .map(|addr| addr.port())
    }

  /// Start a new server on the runtime, returning the bound port the mock server is running on
  #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
  pub async fn start_mock_server_nonblocking(
    &mut self,
    id: String,
    pact: Box<dyn Pact + Send + Sync>,
    port: u16,
    config: MockServerConfig
  ) -> Result<u16, String> {
    let mock_server = MockServerBuilder::new()
        .with_pact(pact)
        .with_config(config)
        .bind_to_port(port)
        .with_id(id.as_str())
        .start()
        .await
        .map_err(|err| err.to_string())?;

    let port = mock_server.address.port();
    self.mock_servers.insert(
      id,
      ServerEntry {
        mock_server: Either::Left(mock_server),
        port,
        resources: vec![]
      }
    );

    Ok(port)
  }

    /// Start a new TLS server on the runtime
    #[cfg(feature = "tls")]
    #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
    pub fn start_tls_mock_server(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      port: u16,
      tls: &ServerConfig,
      config: MockServerConfig
    ) -> anyhow::Result<u16> {
      #[allow(deprecated)]
      self.start_tls_mock_server_with_addr(id, pact, ([0, 0, 0, 0], port as u16).into(), tls, config)
          .map(|addr| addr.port())
    }

  /// Start a new mock server for the provided transport on the runtime. Returns the socket address
  /// that the server is running on.
  #[allow(unused_variables)]
  #[deprecated(since = "2.0.0-beta.0", note = "Use the mock server builder (MockServerBuilder)")]
  pub fn start_mock_server_for_transport(
    &mut self,
    id: String,
    pact: Box<dyn Pact + Send + Sync>,
    addr: SocketAddr,
    transport: &CatalogueEntry,
    config: MockServerConfig
  ) -> anyhow::Result<SocketAddr> {
    #[cfg(feature = "plugins")]
    {
      if transport.provider_type == CatalogueEntryProviderType::PLUGIN {
        let mut v4_pact = pact.as_v4_pact()?;
        for interaction in v4_pact.interactions.iter_mut() {
          if let None = interaction.transport() {
            interaction.set_transport(transport.key.split("/").last().map(|i| i.to_string()));
          }
        }
        let mock_server_config = pact_plugin_driver::mock_server::MockServerConfig {
          output_path: None,
          host_interface: Some(addr.ip().to_string()),
          port: addr.port() as u32,
          tls: false
        };
        let test_context = hashmap! {};
        let result = self.runtime.block_on(
          pact_plugin_driver::plugin_manager::start_mock_server_v2(transport, v4_pact.boxed(),
                                                                   mock_server_config, test_context)
        )?;
        self.mock_servers.insert(
          id,
          ServerEntry {
            mock_server: Either::Right(PluginMockServer {
              mock_server_details: result.clone(),
              catalogue_entry: transport.clone(),
              pact: v4_pact
            }),
            port: result.port as u16,
            resources: vec![]
          }
        );

        let url = Url::parse(&result.base_url)?;
        (url.host_str().unwrap_or_default(), result.port as u16).to_socket_addrs()?.next()
          .ok_or_else(|| anyhow!("Could not parse the result from the plugin as a socket address"))
      } else {
        #[allow(deprecated)]
        self.start_mock_server_with_addr(id, pact, addr, config)
          .map_err(|err| anyhow!(err))
      }
    }
    #[cfg(not(feature = "plugins"))]
    {
      #[allow(deprecated)]
      self.start_mock_server_with_addr(id, pact, addr, config)
        .map_err(|err| anyhow!(err))
    }
  }

  /// Shut down a server by its id. This function will only shut down a local mock server, not one
  /// provided by a plugin.
  pub fn shutdown_mock_server_by_id<S: Into<String>>(&mut self, id: S) -> bool {
    let id = id.into();
    match self.mock_servers.remove(&id) {
      Some(entry) => match entry.mock_server {
        Either::Left(mut mock_server) => {
          match mock_server.shutdown() {
            Ok(()) => {
              let metrics = {
                let guard = mock_server.metrics.lock().unwrap();
                guard.clone()
              };
              debug!("Shutting down mock server with ID {} - {:?}", id, metrics);
              true
            },
            Err(err) => {
              error!("Failed to shutdown the mock server with ID {}: {}", id, err);
              false
            }
          }
        }
        Either::Right(_plugin_mock_server) => {
          #[cfg(feature = "plugins")]
          {
            match self.runtime.block_on(pact_plugin_driver::plugin_manager::shutdown_mock_server(&_plugin_mock_server.mock_server_details)) {
              Ok(_) => true,
              Err(err) => {
                error!("Failed to shutdown plugin mock server with ID {} - {}", id, err);
                false
              }
            }
          }
          #[cfg(not(feature = "plugins"))]
          {
            error!("Mockserver has been provided by a plugin. Plugins require the plugin feature to be enabled");
            false
          }
        }
      },
      None => false
    }
  }

  /// Shut down a server by its local port number
  pub fn shutdown_mock_server_by_port(&mut self, port: u16) -> bool {
    debug!("Shutting down mock server with port {}", port);
    let result = self
      .mock_servers
      .iter()
      .find_map(|(id, entry)| {
        if entry.port == port {
          Some(id.clone())
        } else {
          None
        }
      });

    if let Some(id) = result {
      self.shutdown_mock_server_by_id(id)
    } else {
      false
    }
  }

  /// Find mock server by id, and map it using supplied function if found.
  pub fn find_mock_server_by_id<R>(
    &self,
    id: &str,
    f: &dyn Fn(&ServerManager, Either<&MockServer, &PluginMockServer>) -> R
  ) -> Option<R> {
    match self.mock_servers.get(id) {
      Some(entry) => match &entry.mock_server {
        Either::Left(mock_server) => {
          Some(f(self, Either::Left(mock_server)))
        }
        Either::Right(plugin_mock_server) => Some(f(self, Either::Right(plugin_mock_server)))
      }
      None => None
    }
  }

  /// Find a mock server by port number and map it using supplied function if found.
  pub fn find_mock_server_by_port<R>(
    &mut self,
    port: u16,
    f: &dyn Fn(&ServerManager, &String, Either<&MockServer, &PluginMockServer>) -> R
  ) -> Option<R> {
    let entry = {
      self.mock_servers
        .iter()
        .find(|(_id, entry)| entry.port == port)
        .map(|(id, entry)| (id.clone(), &entry.mock_server))
    };
    match entry {
      Some((id, entry)) => match entry {
        Either::Left(mock_server) => {
          Some(f(self, &id, Either::Left(mock_server)))
        }
        Either::Right(plugin_mock_server) => Some(f(self, &id, Either::Right(plugin_mock_server)))
      }
      None => None,
    }
  }

  /// Find a mock server by port number and apply a mutating operation on it if successful. This will
  /// only work for locally managed mock servers, not mock servers provided by plugins.
  pub fn find_mock_server_by_port_mut<R>(
    &mut self,
    port: u16,
    f: &dyn Fn(&mut MockServer) -> R,
  ) -> Option<R> {
    match self
      .mock_servers
      .iter_mut()
      .find(|(_id, entry)| entry.port == port)
    {
      Some((_id, entry)) => match &mut entry.mock_server {
        Either::Left(mock_server) => {
          Some(f(mock_server))
        }
        Either::Right(_) => None
      }
      None => None
    }
  }

  /// Map all the running mock servers This will only work for locally managed mock servers,
  /// not mock servers provided by plugins.
  pub fn map_mock_servers<R, F>(&self, f: F) -> Vec<R>
    where F: Fn(&MockServer) -> R {
    let mut results = vec![];
    for (id, entry) in self.mock_servers.iter() {
      trace!(?id, "mock server entry");
      if let Either::Left(mock_server) = &entry.mock_server {
        results.push(f(mock_server));
      }
    }
    trace!("returning results");
    return results;
  }

  /// Execute a future on the Tokio runtime for the service manager
  #[cfg(feature = "plugins")]
  pub(crate) fn exec_async<OUT>(&self, future: impl Future<Output=OUT>) -> OUT {
    self.runtime.block_on(future)
  }

  /// Store a string that needs to be cleaned up when the mock server terminates
  pub fn store_mock_server_resource(&mut self, port: u16, s: CString) -> bool {
    if let Some((_, entry)) = self.mock_servers
      .iter_mut()
      .find(|(_id, entry)| entry.port == port) {
      entry.resources.push(s);
      true
    } else {
      false
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{thread, time};
  use std::net::TcpStream;

  use env_logger;
  use pact_models::sync_pact::RequestResponsePact;

  use super::*;

  #[test]
    #[cfg(not(target_os = "windows"))]
    fn manager_should_start_and_shutdown_mock_server() {
        let _ = env_logger::builder().is_test(true).try_init();
        let mut manager = ServerManager::new();
        #[allow(deprecated)]
        let start_result = manager.start_mock_server("foobar".into(),
                                                     RequestResponsePact::default().boxed(),
                                                     0, MockServerConfig::default());

        assert!(start_result.is_ok());
        let server_port = start_result.unwrap();

        // Server should be up
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_ok());

        // Should be able to read matches without blocking
        let matches =
            manager.find_mock_server_by_port_mut(server_port, &|mock_server| mock_server.matches());
        assert_eq!(matches, Some(vec![]));

        let stopped = manager.shutdown_mock_server_by_port(server_port);
        assert!(stopped);

        // The tokio runtime is now out of tasks
        drop(manager);

        let millis = time::Duration::from_millis(100);
        thread::sleep(millis);

        // Server should be down
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_err());
    }
}
