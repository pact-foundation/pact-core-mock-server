//!
//! This module defines a manager for holding multiple instances of mock servers.
//!

use std::collections::BTreeMap;
use std::ffi::CString;
#[cfg(feature = "plugins")] use std::future::Future;
use std::net::SocketAddr;
#[cfg(feature = "plugins")] use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};

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
  mock_server: Either<Arc<Mutex<MockServer>>, PluginMockServer>,
  /// Port the mock server is running on
  port: u16,
  /// List of resources that need to be cleaned up when the mock server completes
  pub resources: Vec<CString>,
  join_handle: Option<tokio::task::JoinHandle<()>>
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
        .worker_threads(1)
        .enable_all()
        .build()
        .unwrap(),
      mock_servers: BTreeMap::new()
    }
  }

    /// Start a new server on the runtime
    pub fn start_mock_server_with_addr(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      addr: SocketAddr,
      config: MockServerConfig
    ) -> Result<SocketAddr, String> {
      // let (mock_server, future) =
      //   self.runtime.block_on(MockServer::new(id.clone(), pact, addr, config))?;
      //
      // let port = { mock_server.lock().unwrap().address.map(|addr| addr.port()) };
      // self.mock_servers.insert(
      //   id,
      //   ServerEntry {
      //     mock_server: Either::Left(mock_server),
      //     port: port.unwrap_or_else(|| addr.port()),
      //     resources: vec![],
      //     join_handle: Some(self.runtime.spawn(future))
      //   },
      // );
      //
      // match port {
      //   Some(port) => Ok(SocketAddr::new(addr.ip(), port)),
      //   None => Ok(addr)
      // }
      todo!()
    }

    /// Start a new TLS server on the runtime
    #[cfg(feature = "tls")]
    pub fn start_tls_mock_server_with_addr(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      addr: SocketAddr,
      tls_config: &ServerConfig,
      config: MockServerConfig
    ) -> Result<SocketAddr, String> {
      // let (mock_server, future) =
      //   self.runtime.block_on(MockServer::new_tls(id.clone(), pact, addr, tls_config, config))?;
      //
      // let port = { mock_server.lock().unwrap().address.map(|addr| addr.port()) };
      // self.mock_servers.insert(
      //   id,
      //   ServerEntry {
      //     mock_server: Either::Left(mock_server),
      //     port: port.unwrap_or_else(|| addr.port()),
      //     resources: vec![],
      //     join_handle: Some(self.runtime.spawn(future))
      //   }
      // );
      //
      // match port {
      //   Some(port) => Ok(SocketAddr::new(addr.ip(), port)),
      //   None => Ok(addr)
      // }
      todo!()
    }

    /// Start a new server on the runtime
    pub fn start_mock_server(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      port: u16,
      config: MockServerConfig
    ) -> Result<u16, String> {
        self.start_mock_server_with_addr(id, pact, ([0, 0, 0, 0], port as u16).into(), config)
            .map(|addr| addr.port())
    }

  /// Start a new server on the runtime, returning the future
  pub async fn start_mock_server_nonblocking(
    &mut self,
    id: String,
    pact: Box<dyn Pact + Send + Sync>,
    port: u16,
    config: MockServerConfig
  ) -> Result<u16, String> {
    // let addr= ([0, 0, 0, 0], port as u16).into();
    // let (mock_server, future) = MockServer::new(id.clone(), pact, addr, config).await?;
    //
    // let port = { mock_server.lock().unwrap().address.map(|addr| addr.port()) };
    // self.mock_servers.insert(
    //   id,
    //   ServerEntry {
    //     mock_server: Either::Left(mock_server),
    //     port: port.unwrap_or_else(|| addr.port()),
    //     resources: vec![],
    //     join_handle: Some(self.runtime.spawn(future))
    //   },
    // );
    //
    // port.ok_or_else(|| "Started mock server has no port".to_string())
    todo!()
  }

    /// Start a new TLS server on the runtime
    #[cfg(feature = "tls")]
    pub fn start_tls_mock_server(
      &mut self,
      id: String,
      pact: Box<dyn Pact + Send + Sync>,
      port: u16,
      tls: &ServerConfig,
      config: MockServerConfig
    ) -> Result<u16, String> {
        self.start_tls_mock_server_with_addr(id, pact, ([0, 0, 0, 0], port as u16).into(), tls, config)
          .map(|addr| addr.port())
    }

  /// Start a new mock server for the provided transport on the runtime. Returns the socket address
  /// that the server is running on.
  #[allow(unused_variables)]
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
            resources: vec![],
            join_handle: None
          }
        );

        let url = Url::parse(&result.base_url)?;
        (url.host_str().unwrap_or_default(), result.port as u16).to_socket_addrs()?.next()
          .ok_or_else(|| anyhow!("Could not parse the result from the plugin as a socket address"))
      } else {
        self.start_mock_server_with_addr(id, pact, addr, config)
          .map_err(|err| anyhow!(err))
      }
    }
    #[cfg(not(feature = "plugins"))]
    {
      self.start_mock_server_with_addr(id, pact, addr, config)
        .map_err(|err| anyhow!(err))
    }
  }

  /// Shut down a server by its id. This function will only shut down a local mock server, not one
  /// provided by a plugin.
  pub fn shutdown_mock_server_by_id(&mut self, id: String) -> bool {
    match self.mock_servers.remove(&id) {
      Some(entry) => match entry.mock_server {
        Either::Left(mock_server) => {
          let mut ms = mock_server.lock().unwrap();
          debug!("Shutting down mock server with ID {} - {:?}", id, ms.metrics);
          match ms.shutdown() {
            Ok(()) => {
              self.runtime.block_on(entry.join_handle.unwrap()).unwrap();
              true
            }
            Err(_) => false,
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
            error!("Plugins require the plugin feature to be enabled");
            false
          }
        }
      },
      None => false,
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
    id: &String,
    f: &dyn Fn(&ServerManager, Either<&MockServer, &PluginMockServer>) -> R
  ) -> Option<R> {
    match self.mock_servers.get(id) {
      Some(entry) => match &entry.mock_server {
        Either::Left(mock_server) => {
          let inner = mock_server.lock().unwrap();
          Some(f(self, Either::Left(&inner)))
        }
        Either::Right(plugin_mock_server) => Some(f(self, Either::Right(plugin_mock_server)))
      }
      None => None,
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
          let inner = mock_server.lock().unwrap();
          Some(f(self, &id, Either::Left(&inner)))
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
          Some(f(&mut mock_server.lock().unwrap()))
        }
        Either::Right(_) => None
      }
      None => None,
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
        trace!(?id, "Waiting on lock for mock server");
        let guard = mock_server.lock().unwrap();
        trace!(?id, "Got access to mock server, invoking callback");
        results.push(f(&guard));
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
