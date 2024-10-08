//!
//! This module defines the external interface for controlling one particular
//! instance of a mock server.
//!

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, mpsc, Mutex};
use std::time::Duration;

use anyhow::anyhow;
use pact_models::generators::generate_hexadecimal;
use pact_models::json_utils::json_to_string;
use pact_models::pact::{Pact, ReadWritePact, write_pact};
use pact_models::PactSpecification;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::pact::V4Pact;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager::CatalogueEntry;
#[cfg(feature = "tls")] use rustls::ServerConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info, trace, warn};

use crate::hyper_server::create_and_bind;
#[cfg(feature = "tls")] use crate::hyper_server::create_and_bind_https;
use crate::matching::MatchResult;
use crate::utils::json_to_bool;

/// Mock server configuration
#[derive(Debug, Clone)]
pub struct MockServerConfig {
  /// If CORS Pre-Flight requests should be responded to
  pub cors_preflight: bool,
  /// Pact specification to use
  pub pact_specification: PactSpecification,
  /// Configuration required for the transport used
  pub transport_config: HashMap<String, Value>,
  /// Address to bind to
  pub address: String,
  /// Unique mock server ID to assign
  pub mockserver_id: Option<String>,
  /// TLS configuration
  #[cfg(feature = "tls")]
  pub tls_config: Option<ServerConfig>,
  /// Transport entry for the mock server
  #[cfg(feature = "plugins")]
  pub transport_entry: Option<CatalogueEntry>,
  /// If connection keep alive should be enabled
  pub keep_alive: bool
}

impl MockServerConfig {
  /// Convert a JSON value into a config struct. This method is tolerant of invalid JSON formats.
  pub fn from_json(value: &Value) -> MockServerConfig {
    let mut config = MockServerConfig::default();

    if let Value::Object(map) = value {
      for (k, v) in map {
        if k == "corsPreflight" {
          config.cors_preflight = json_to_bool(v).unwrap_or_default();
        } else if k == "pactSpecification" {
          config.pact_specification = PactSpecification::from(json_to_string(v));
        } else if k == "keepAlive" {
          config.keep_alive = json_to_bool(v).unwrap_or_default();
        } else {
          config.transport_config.insert(k.clone(), v.clone());
        }
      }
    }

    config
  }

  /// Return default config with keep alive enabled
  pub fn with_keep_alive(keep_alive: bool) -> Self {
    MockServerConfig {
      keep_alive,
      .. MockServerConfig::default()
    }
  }
}

impl Default for MockServerConfig {
  #[cfg(all(feature = "tls", feature = "plugins"))]
  fn default() -> Self {
    MockServerConfig {
      cors_preflight: false,
      pact_specification: Default::default(),
      transport_config: Default::default(),
      address: "".to_string(),
      mockserver_id: None,
      tls_config: None,
      transport_entry: None,
      keep_alive: true
    }
  }

  #[cfg(all(feature = "tls", not(feature = "plugins")))]
  fn default() -> Self {
    MockServerConfig {
      cors_preflight: false,
      pact_specification: Default::default(),
      transport_config: Default::default(),
      address: "".to_string(),
      mockserver_id: None,
      tls_config: None,
      keep_alive: true
    }
  }

  #[cfg(all(not(feature = "tls"), feature = "plugins"))]
  fn default() -> Self {
    MockServerConfig {
      cors_preflight: false,
      pact_specification: Default::default(),
      transport_config: Default::default(),
      address: "".to_string(),
      mockserver_id: None,
      transport_entry: None,
      keep_alive: true
    }
  }

  #[cfg(not(any(feature = "tls", feature = "plugins")))]
  fn default() -> Self {
    MockServerConfig {
      cors_preflight: false,
      pact_specification: Default::default(),
      transport_config: Default::default(),
      address: "".to_string(),
      mockserver_id: None,
      keep_alive: true
    }
  }
}

// Need to implement this, as we can't compare TLS configuration.
impl PartialEq for MockServerConfig {
  fn eq(&self, other: &Self) -> bool {
    let ok = self.cors_preflight == other.cors_preflight
      && self.pact_specification == other.pact_specification
      && self.transport_config == other.transport_config
      && self.address == other.address
      && self.mockserver_id == other.mockserver_id
      && self.keep_alive == other.keep_alive;

    #[cfg(feature = "plugins")]
    {
      ok && self.transport_entry == other.transport_entry
    }

    #[cfg(not(feature = "plugins"))]
    {
      ok
    }
  }
}

/// Mock server scheme
#[derive(Debug, Clone)]
pub enum MockServerScheme {
  /// HTTP
  HTTP,
  /// HTTPS
  HTTPS
}

impl Default for MockServerScheme {
  fn default() -> Self {
    MockServerScheme::HTTP
  }
}

impl Display for MockServerScheme {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      MockServerScheme::HTTP => write!(f, "http"),
      MockServerScheme::HTTPS => write!(f, "https")
    }
  }
}

/// Metrics for the mock server
#[derive(Debug, Default, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct MockServerMetrics {
  /// Total requests
  pub requests: usize,
  /// Total requests by path
  pub requests_by_path: HashMap<String, usize>
}

impl MockServerMetrics {
  pub(crate) fn add_path(&mut self, path: String) {
    self.requests += 1;
    *self.requests_by_path
      .entry(path)
      .or_insert(0)
      += 1;
  }
}

/// Events sent from the mock server task to be consumed by the mock server event loop.
#[derive(Debug, Clone, PartialEq)]
pub enum MockServerEvent {
  /// Connection failed with error
  ConnectionFailed(String),
  /// Request received with path
  RequestReceived(String),
  /// Result of matching a request
  RequestMatch(MatchResult),
  /// Server is shutting down
  ServerShutdown
}

/// Struct to represent the "foreground" part of mock server. Note that while Clone has been
/// implemented, clones of the mock server are detached from the background tasks, and so
/// should only be used to extract data at a point in time and then discarded.
#[derive(Debug)]
pub struct MockServer {
  /// Mock server unique ID
  pub id: String,
  /// Scheme the mock server is using
  pub scheme: MockServerScheme,
  /// Address the mock server is bound to
  pub address: SocketAddr,
  /// Pact that this mock server is based on
  pub pact: V4Pact,
  /// Receiver of match results
  matches: Arc<Mutex<Vec<MatchResult>>>,
  /// Sender to signal main server to shutdown
  shutdown_tx: RefCell<Option<tokio::sync::oneshot::Sender<()>>>,
  /// Mock server config
  pub config: MockServerConfig,
  /// Metrics collected by the mock server
  pub metrics: Arc<Mutex<MockServerMetrics>>,
  /// Pact spec version to use
  pub spec_version: PactSpecification,
  /// Event loop shutdown signal receiver. Message will be sent when the event loop has terminated.
  pub event_loop_rx: Option<mpsc::Receiver<()>>
}

impl Clone for MockServer {
  fn clone(&self) -> Self {
    MockServer {
      id: self.id.clone(),
      scheme: self.scheme.clone(),
      address: self.address.clone(),
      pact: self.pact.clone(),
      matches: self.matches.clone(),
      shutdown_tx: RefCell::new(None),
      config: self.config.clone(),
      metrics: self.metrics.clone(),
      spec_version: self.spec_version.clone(),
      event_loop_rx: None
    }
  }
}

impl Default for MockServer {
  fn default() -> Self {
    MockServer {
      id: generate_hexadecimal(8),
      scheme: MockServerScheme::HTTP,
      address: SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0),
      pact: Default::default(),
      matches: Arc::new(Mutex::new(vec![])),
      shutdown_tx: RefCell::new(None),
      config: Default::default(),
      metrics: Arc::new(Mutex::new(Default::default())),
      spec_version: Default::default(),
      event_loop_rx: None
    }
  }
}

impl MockServer {
  /// Create a new mock server, spawn its execution loop onto the tokio runtime and return the
  /// mock server instance.
  pub async fn create(
    pact: V4Pact,
    config: MockServerConfig
  ) -> anyhow::Result<MockServer> {
    let server_id = config.mockserver_id
      .clone()
      .unwrap_or_else(|| generate_hexadecimal(8));

    let address = if config.address.is_empty() {
      SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0)
    } else {
      config.address.parse()?
    };

    trace!(%server_id, %address, "Starting mock server");
    let (address, shutdown_send, event_recv, _task_handle) = create_and_bind(server_id.clone(), pact.clone(), address, config.clone()).await?;
    trace!(%server_id, %address, "Mock server started");

    let mut mock_server = MockServer {
      id: server_id,
      scheme: Default::default(),
      address,
      pact,
      matches: Default::default(),
      shutdown_tx: RefCell::new(Some(shutdown_send)),
      config: config.clone(),
      metrics: Default::default(),
      spec_version: config.pact_specification,
      event_loop_rx: None
    };

    mock_server.start_event_loop(event_recv);

    Ok(mock_server)
  }

  /// Create a new mock server serving over HTTPS, spawn its execution loop onto the tokio runtime and return the
  /// mock server instance. If no TLS configuration has been supplied, the mock server will use
  /// a new self-signed certificate.
  #[cfg(feature = "tls")]
  pub async fn create_https(
    pact: V4Pact,
    config: MockServerConfig
  ) -> anyhow::Result<MockServer> {
    let server_id = generate_hexadecimal(8);

    let address = if config.address.is_empty() {
      SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0)
    } else {
      config.address.parse()?
    };

    trace!(%server_id, %address, "Starting TLS mock server");
    let (address, shutdown_send, event_recv, _task_handle) = create_and_bind_https(server_id.clone(), pact.clone(), address, config.clone()).await?;
    trace!(%server_id, %address, "TLS mock server started");

    let mut mock_server = MockServer {
      id: server_id,
      scheme: MockServerScheme::HTTPS,
      address,
      pact,
      matches: Default::default(),
      shutdown_tx: RefCell::new(Some(shutdown_send)),
      config: config.clone(),
      metrics: Default::default(),
      spec_version: Default::default(),
      event_loop_rx: None
    };

    mock_server.start_event_loop(event_recv);

    Ok(mock_server)
  }

  /// Send the shutdown signal to the server and wait for tasks to complete.
  pub fn shutdown(&mut self) -> anyhow::Result<()> {
    trace!(server_id = %self.id, address = %self.address, "Shutting mock server down");
    match self.shutdown_tx.take() {
      Some(sender) => {
        match sender.send(()) {
          Ok(()) => {
            trace!(server_id = %self.id, address = %self.address, "Shutdown event sent, waiting for tasks to complete");
            if let Some(recv) = self.event_loop_rx.take() {
              let _ = recv.recv_timeout(Duration::from_millis(100));
            }
            let metrics = {
              let guard = self.metrics.lock().unwrap();
              guard.clone()
            };
            debug!("Mock server {} shutdown - {:?}", self.id, metrics);
            Ok(())
          },
          Err(_err) => Err(anyhow!("Problem sending shutdown signal to mock server"))
        }
      },
      _ => Err(anyhow!("Mock server already shut down"))
    }
  }

  /// Start the event loop for the mock server
  fn start_event_loop(&mut self, mut event_recv: Receiver<MockServerEvent>) {
    let server_id = self.id.clone();
    let metrics = self.metrics.clone();
    let matches = self.matches.clone();
    let (sender, receiver) = mpsc::channel();
    self.event_loop_rx = Some(receiver);

    tokio::spawn(async move {
      trace!(%server_id, "Starting mock server event loop");

      let mut total_events = 0;
      let metrics = metrics.clone();
      let matches = matches.clone();
      while let Some(event) = event_recv.recv().await {
        trace!(%server_id, ?event, "Received event");
        total_events += 1;

        match event {
          MockServerEvent::ConnectionFailed(_err) => {}
          MockServerEvent::RequestReceived(path) => {
            let mut guard = metrics.lock().unwrap();
            guard.add_path(path);
          }
          MockServerEvent::RequestMatch(result) => {
            let mut guard = matches.lock().unwrap();
            guard.push(result.clone());
          }
          MockServerEvent::ServerShutdown => {
            trace!(%server_id, total_events, "Exiting mock server event loop");
            break;
          }
        }
      }

      trace!(%server_id, total_events, "Mock server event loop done");
      let _ = sender.send(());
    });
  }

  /// Converts this mock server to a `Value` struct
  pub fn to_json(&self) -> Value {
    let metrics = {
      let guard = self.metrics.lock().unwrap();
      guard.clone()
    };
    json!({
      "id" : self.id.clone(),
      "port" : self.address.port(),
      "address" : self.address.to_string(),
      "scheme" : self.scheme.to_string(),
      "provider" : self.pact.provider().name.clone(),
      "status" : if self.mismatches().is_empty() { "ok" } else { "error" },
      "metrics" : metrics
    })
  }

  /// Returns all collected matches
  pub fn matches(&self) -> Vec<MatchResult> {
    let guard = self.matches.lock().unwrap();
    guard.clone()
  }

  /// If all requests to the mock server matched correctly
  pub fn all_matched(&self) -> bool {
    self.mismatches().is_empty()
  }

    /// Returns all the mismatches that have occurred with this mock server
    pub fn mismatches(&self) -> Vec<MatchResult> {
      let matches = self.matches();
      let mismatches = matches.iter()
        .filter(|m| !m.matched() && !m.cors_preflight())
        .cloned();
      let requests: Vec<HttpRequest> = matches.iter().map(|m| {
        match m {
          MatchResult::RequestMatch(request, _, _) => Some(request),
          MatchResult::RequestMismatch(request, _, _) => Some(request),
          MatchResult::RequestNotFound(_) => None,
          MatchResult::MissingRequest(_) => None
        }
      }).filter(|o| o.is_some())
        .map(|o| o.unwrap().clone())
        .collect();

      let interactions = self.pact.interactions();
      let missing = interactions.iter()
        .map(|i| i.as_v4_http().unwrap().request)
        .filter(|req| !requests.contains(req))
        .map(|req| MatchResult::MissingRequest(req.clone()));
      mismatches.chain(missing).collect()
    }

  /// Mock server writes its pact out to the provided directory
  pub fn write_pact(&self, output_path: &Option<String>, overwrite: bool) -> anyhow::Result<()> {
    trace!("write_pact: output_path = {:?}, overwrite = {}", output_path, overwrite);
    let mut v4_pact = self.pact.clone();
    v4_pact.add_md_version("mockserver", option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
    for interaction in &mut v4_pact.interactions {
      interaction.set_transport(Some("http".to_string()));
    }

    let pact_file_name = v4_pact.default_file_name();
    let filename = match *output_path {
      Some(ref path) => {
        let mut path = PathBuf::from(path);
        path.push(pact_file_name);
        path
      },
      None => PathBuf::from(pact_file_name)
    };

    info!("Writing pact out to '{}'", filename.display());
    let specification = match self.spec_version {
      PactSpecification::Unknown => self.pact.specification_version(),
      _ => self.spec_version
    };
    match write_pact(v4_pact.boxed(), filename.as_path(), specification, overwrite) {
      Ok(_) => Ok(()),
      Err(err) => {
        warn!("Failed to write pact to file - {}", err);
        Err(err)
      }
    }
  }

  /// Returns the URL of the mock server
  pub fn url(&self) -> String {
    if self.address.ip().is_unspecified() {
      if self.address.is_ipv4() {
        format!("{}://{}:{}", self.scheme, Ipv4Addr::LOCALHOST, self.address.port())
      } else {
        format!("{}://[{}]:{}", self.scheme, Ipv6Addr::LOCALHOST, self.address.port())
      }
    } else {
      format!("{}://{}", self.scheme, self.address)
    }
  }

  /// Returns the port the mock server is running on. Returns None when the mock server
  /// has not started yet.
  pub fn port(&self) -> u16 {
    self.address.port()
  }
}

#[cfg(test)]
mod tests {
  use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
  use std::str::FromStr;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::PactSpecification;
  use serde_json::{json, Value};

  use crate::mock_server::{MockServer, MockServerConfig};

  #[test]
  fn test_mock_server_config_from_json() {
    expect!(MockServerConfig::from_json(&Value::Null)).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::Array(vec![]))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::String("s".into()))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::Bool(true))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&json!(12334))).to(be_equal_to(MockServerConfig::default()));

    let config = MockServerConfig {
      cors_preflight: true,
      pact_specification: PactSpecification::V4,
      transport_config: hashmap! {
        "tlsKey".to_string() => json!("key"),
        "tlsCertificate".to_string() => json!("cert")
      },
      address: "".to_string(),
      mockserver_id: None,
      .. MockServerConfig::default()
    };
    expect!(MockServerConfig::from_json(&json!({
      "corsPreflight": true,
      "pactSpecification": "V4",
      "tlsKey": "key",
      "tlsCertificate": "cert"
    }))).to(be_equal_to(config));

    let config = MockServerConfig {
      keep_alive: true,
      .. MockServerConfig::default()
    };
    expect!(MockServerConfig::from_json(&json!({
      "keepAlive": true
    }))).to(be_equal_to(config));
  }

  #[test]
  fn mock_server_url() {
    let ms = MockServer {
      address: SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://[::1]:0"));
    expect!(ms.port()).to(be_equal_to(0));

    let ms = MockServer {
      address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 0),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://127.0.0.1:0"));
    expect!(ms.port()).to(be_equal_to(0));

    let ms = MockServer {
      address: SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://[::1]:0"));
    expect!(ms.port()).to(be_equal_to(0));

    let ms = MockServer {
      address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://127.0.0.1:0"));
    expect!(ms.port()).to(be_equal_to(0));

    let ms = MockServer {
      address: SocketAddr::new(Ipv6Addr::from_str("fe80::42:31ff:fe22:6d4b").unwrap().into(), 80),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://[fe80::42:31ff:fe22:6d4b]:80"));
    expect!(ms.port()).to(be_equal_to(80));

    let ms = MockServer {
      address: SocketAddr::new(Ipv4Addr::from([10, 0, 0, 1]).into(), 1025),
      .. MockServer::default()
    };
    expect!(ms.url()).to(be_equal_to("http://10.0.0.1:1025"));
    expect!(ms.port()).to(be_equal_to(1025));
  }
}
