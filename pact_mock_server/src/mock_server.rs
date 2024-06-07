//!
//! This module defines the external interface for controlling one particular
//! instance of a mock server.
//!

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::net::{Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::anyhow;

use pact_models::json_utils::json_to_string;
use pact_models::pact::{Pact, ReadWritePact, write_pact};
use pact_models::PactSpecification;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::pact::V4Pact;
#[cfg(feature = "tls")] use rustls::ServerConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info, trace, warn};
use crate::hyper_server::create_and_bind;

use crate::matching::MatchResult;
use crate::utils::json_to_bool;

/// Mock server configuration
#[derive(Debug, Default, Clone, PartialEq)]
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
  pub mockserver_id: Option<String>
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
        } else {
          config.transport_config.insert(k.clone(), v.clone());
        }
      }
    }

    config
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
#[derive(Debug, Clone)]
pub enum MockServerEvent {
  /// Connection failed with error
  ConnectionFailed(String),
  /// Request received with path
  RequestReceived(String),
  /// Result of matching a request
  RequestMatch(MatchResult),
}

/// Struct to represent the "foreground" part of mock server
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
  /// Shutdown signal
  shutdown_tx: RefCell<Option<tokio::sync::oneshot::Sender<()>>>,
  /// Mock server config
  pub config: MockServerConfig,
  /// Metrics collected by the mock server
  pub metrics: Arc<Mutex<MockServerMetrics>>,
  /// Pact spec version to use
  pub spec_version: PactSpecification
}

impl MockServer {
  /// Create a new mock server, spawn its execution loop onto the tokio runtime and return the
  /// mock server instance.
  pub async fn create(
    pact: V4Pact,
    config: MockServerConfig
  ) -> anyhow::Result<MockServer> {
    let server_id = uuid::Uuid::new_v4().to_string();

    let address = if config.address.is_empty() {
      SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0)
    } else {
      config.address.parse()?
    };

    trace!(%server_id, %address, "Starting mock server");
    let (address, shutdown_send, event_recv) = create_and_bind(server_id.clone(), pact.clone(), address, config.clone()).await?;
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
      spec_version: Default::default()
    };

    mock_server.start_event_loop(event_recv).await;

    Ok(mock_server)
  }

  /// Create a new TLS mock server, consisting of its state (self) and its executable server future.
  // #[cfg(feature = "tls")]
  // #[deprecated(since = "2.0.0-beta.0", note = "use create instead")]
  // pub async fn new_tls(
  //   id: String,
  //   pact: Box<dyn Pact + Send + Sync>,
  //   addr: std::net::SocketAddr,
  //   tls: &ServerConfig,
  //   config: MockServerConfig
  // ) -> Result<(Arc<Mutex<MockServer>>, impl std::future::Future<Output = ()>), String> {
    // let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    // let matches = Arc::new(Mutex::new(vec![]));
    //
    // #[allow(deprecated)]
    // let mock_server = Arc::new(Mutex::new(MockServer {
    //   id: id.clone(),
    //   address: None,
    //   scheme: MockServerScheme::HTTPS,
    //   pact: pact.boxed(),
    //   matches: matches.clone(),
    //   shutdown_tx: RefCell::new(Some(shutdown_tx)),
    //   event_rx: RefCell::new(None),
    //   config: config.clone(),
    //   metrics: MockServerMetrics::default(),
    //   spec_version: pact_specification(config.pact_specification, pact.specification_version())
    // }));

    // let (future, socket_addr) = crate::legacy::hyper_server::create_and_bind_tls(
    //   pact,
    //   addr,
    //   async {
    //     shutdown_rx.await.ok();
    //   },
    //   matches,
    //   tls.clone(),
    //   mock_server.clone()
    // ).await.map_err(|err| format!("Could not start server: {}", err))?;
    //
    // {
    //   let mut ms = mock_server.lock().unwrap();
    //   ms.deref_mut().address = Some(socket_addr.clone());
    //
    //   debug!("Started mock server on {}:{}", socket_addr.ip(), socket_addr.port());
    // }
    //
    // Ok((mock_server.clone(), future))
  //   todo!()
  // }

  /// Send the shutdown signal to the server
  pub fn shutdown(&self) -> anyhow::Result<()> {
    let shutdown_future = self.shutdown_tx.take();
    match shutdown_future {
      Some(sender) => {
        match sender.send(()) {
          Ok(()) => {
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
  async fn start_event_loop(&mut self, mut event_recv: Receiver<MockServerEvent>) {
    let server_id = self.id.clone();
    let mut metrics = self.metrics.clone();
    let mut matches = self.matches.clone();

    tokio::spawn(async move {
      trace!(%server_id, "Starting mock server event loop");

      let mut total_events = 0;
      let mut metrics = metrics.clone();
      let mut matches = matches.clone();
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
        }
      }

      trace!(%server_id, total_events, "Mock server event loop done");
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
      PactSpecification::Unknown => PactSpecification::V3,
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
      format!("{}://{}:{}", self.scheme, Ipv6Addr::LOCALHOST, self.address.port())
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

fn pact_specification(spec1: PactSpecification, spec2: PactSpecification) -> PactSpecification {
  match spec1 {
    PactSpecification::Unknown => spec2,
    _ => spec1
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::PactSpecification;
  use serde_json::{json, Value};

  use crate::mock_server::MockServerConfig;

  #[test]
  fn test_mock_server_config_from_json() {
    expect!(MockServerConfig::from_json(&Value::Null)).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::Array(vec![]))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::String("s".into()))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&Value::Bool(true))).to(be_equal_to(MockServerConfig::default()));
    expect!(MockServerConfig::from_json(&json!(12334))).to(be_equal_to(MockServerConfig::default()));

    expect!(MockServerConfig::from_json(&json!({
      "corsPreflight": true,
      "pactSpecification": "V4",
      "tlsKey": "key",
      "tlsCertificate": "cert"
    }))).to(be_equal_to(MockServerConfig {
      cors_preflight: true,
      pact_specification: PactSpecification::V4,
      transport_config: hashmap! {
        "tlsKey".to_string() => json!("key"),
        "tlsCertificate".to_string() => json!("cert")
      },
      address: "".to_string(),
      mockserver_id: None
    }));
  }
}
