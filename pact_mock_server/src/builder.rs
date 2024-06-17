//! Provides a builder for constructing mock servers

use anyhow::anyhow;
use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::v4::pact::V4Pact;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager;
#[cfg(feature = "tls")] use rustls::ServerConfig;

use crate::{configure_core_catalogue, MANAGER};
use crate::mock_server::{MockServer, MockServerConfig};
use crate::server_manager::ServerManager;

/// Builder for constructing mock servers
pub struct MockServerBuilder {
  config: MockServerConfig,
  pact: V4Pact
}

impl MockServerBuilder {
  /// Construct a new builder
  pub fn new() -> Self {
    configure_core_catalogue();
    pact_matching::matchers::configure_core_catalogue();

    MockServerBuilder {
      config: Default::default(),
      pact: V4Pact::default()
    }
  }

  /// Add the Pact that the mock server will respond with
  pub fn with_v4_pact(mut self, pact: V4Pact) -> Self {
    self.pact = pact;
    self.config.pact_specification = PactSpecification::V4;
    self
  }

  /// Add the Pact that the mock server will respond with
  pub fn with_pact(mut self, pact: Box<dyn Pact + Send + Sync>) -> Self {
    self.pact = pact.as_v4_pact().unwrap();
    self.config.pact_specification = pact.specification_version();
    self
  }

  /// The address this mock server mist bind to in the form <host>:<port>. Defaults to the IP6
  /// loopback adapter (ip6-localhost, `[::1]`). Specify 0 for the port to get a random OS assigned
  /// port. This is what you would mostly want with a mock server in a test, otherwise your test
  /// could fail with port conflicts.
  ///
  /// Common options are:
  /// * IP4 loopback adapter: `127.0.0.1:0`
  /// * IP6 loopback adapter: `[::1]:0`
  /// * Bind to all adapters with IP4: `0.0.0.0:0`
  /// * Bind to all adapters with IP6: `[::]:0`
  pub fn bind_to<S: Into<String>>(mut self, address: S) -> Self {
    self.config.address = address.into();
    self
  }

  /// Sets the mock server to bind to the given port on the IP6
  /// loopback adapter (ip6-localhost, `[::1]`). Specify 0 for the port to get a random OS assigned
  /// port. This is what you would mostly want with a mock server in a test, otherwise your test
  /// could fail with port conflicts.
  pub fn bind_to_port(mut self, port: u16) -> Self {
    self.config.address = format!("[::1]:{}", port);
    self
  }

  /// Provide the config used to setup the mock server. Note that this will override any values
  /// that have been set with functions like `bind_to`, etc.
  pub fn with_config(mut self, config: MockServerConfig) -> Self {
    self.config = config;
    self
  }

  /// Provide the TLS config used to setup the TLS connection.
  #[cfg(feature = "tls")]
  pub fn with_tls_config(mut self, tls_config: &ServerConfig) -> Self {
    self.config.tls_config = Some(tls_config.clone());
    self
  }

  /// Sets the unique ID for the mock server. This is an optional method, and a UUID will
  /// be assigned if this value is not specified.
  pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
    self.config.mockserver_id = Some(id.into());
    self
  }

  /// If CORS Pre-Flight requests should be responded to
  pub fn with_cors_preflight(mut self, cors_preflight: bool) -> Self {
    self.config.cors_preflight = cors_preflight;
    self
  }

  /// Set the transport to use. The default transports are 'http' and 'https'. Additional transports
  /// can be provided by plugins.
  #[cfg(feature = "plugins")]
  pub fn with_transport<S: Into<String>>(mut self, transport: S) -> anyhow::Result<Self> {
    let transport = transport.into();
    let key = format!("transport/{}", transport);
    let transport_entry = catalogue_manager::lookup_entry(key.as_str())
      .ok_or_else(|| anyhow!("Transport '{}' is not a known transport", transport))?;
    self.config.transport_entry = Some(transport_entry);
    Ok(self)
  }

  /// Start the mock server, consuming this builder and returning a mock server instance
  pub async fn start(self) -> anyhow::Result<MockServer> {
    MockServer::create(self.pact.clone(), self.config.clone()).await
  }

  /// Start the mock server serving HTTPS, consuming this builder and returning a mock server instance
  #[cfg(feature = "tls")]
  pub async fn start_https(self) -> anyhow::Result<MockServer> {
    MockServer::create_https(self.pact.clone(), self.config.clone()).await
  }

  /// Starts the mockserver, consuming this builder and registers it with the global server manager.
  /// The mock server tasks will be spawned on the server manager's runtime.
  /// Returns the mock server instance.
  pub fn attach_to_global_manager(self) -> anyhow::Result<MockServer> {
    let mut guard = MANAGER.lock().unwrap();
    let manager = guard.get_or_insert_with(|| ServerManager::new());
    manager.spawn_mock_server(self)
  }
}

#[cfg(test)]
mod tests {
  use std::thread;
  use std::time::Duration;
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::prelude::v4::{SynchronousHttp, V4Pact};
  use pact_models::v4::http_parts::HttpRequest;
  use pact_models::v4::interaction::V4Interaction;
  use reqwest::header::ACCEPT;

  use super::MockServerBuilder;

  #[test_log::test]
  fn basic_mock_server_test() {
    if !std::env::var("NO_IP6").is_ok() {
      let pact = V4Pact {
        interactions: vec![
          SynchronousHttp {
            request: HttpRequest {
              headers: Some(hashmap! {
              "accept".to_string() => vec!["application/json".to_string()]
            }),
              ..HttpRequest::default()
            },
            ..SynchronousHttp::default()
          }.boxed_v4()
        ],
        ..V4Pact::default()
      };

      let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

      let mock_server = runtime.block_on(async {
        MockServerBuilder::new()
          .with_v4_pact(pact)
          .start()
          .await
          .unwrap()
      });

      let client = reqwest::blocking::Client::new();
      let response = client.get(format!("http://[::1]:{}", mock_server.port()).as_str())
        .header(ACCEPT, "application/json").send();

      mock_server.shutdown().unwrap();
      let all_matched = mock_server.all_matched();
      let mismatches = mock_server.mismatches();

      expect!(response.unwrap().status()).to(be_equal_to(200));
      expect!(all_matched).to(be_true());
      expect!(mismatches).to(be_equal_to(vec![]));
    }
  }

  #[test_log::test]
  fn basic_mock_server_test_ip4() {
    let pact = V4Pact {
      interactions: vec![
        SynchronousHttp {
          request: HttpRequest {
            headers: Some(hashmap! {
            "accept".to_string() => vec!["application/json".to_string()]
          }),
            .. HttpRequest::default()
          },
          .. SynchronousHttp::default()
        }.boxed_v4()
      ],
      .. V4Pact::default()
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap();

    let mock_server = runtime.block_on(async {
      MockServerBuilder::new()
        .bind_to("127.0.0.1:0")
        .with_v4_pact(pact)
        .start()
        .await
        .unwrap()
    });

    let client = reqwest::blocking::Client::new();
    let response = client.get(format!("http://127.0.0.1:{}", mock_server.port()).as_str())
      .header(ACCEPT, "application/json").send();

    mock_server.shutdown().unwrap();
    let all_matched = mock_server.all_matched();
    let mismatches = mock_server.mismatches();

    expect!(response.unwrap().status()).to(be_equal_to(200));
    expect!(all_matched).to(be_true());
    expect!(mismatches).to(be_equal_to(vec![]));
  }

  #[test_log::test]
  fn basic_mock_server_https_test() {
    let pact = V4Pact {
      interactions: vec![
        SynchronousHttp {
          request: HttpRequest {
            headers: Some(hashmap! {
            "accept".to_string() => vec!["application/json".to_string()]
          }),
            .. HttpRequest::default()
          },
          .. SynchronousHttp::default()
        }.boxed_v4()
      ],
      .. V4Pact::default()
    };

    let runtime = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap();

    let mock_server = runtime.block_on(async {
      MockServerBuilder::new()
        .bind_to("127.0.0.1:0")
        .with_v4_pact(pact)
        .start_https()
        .await
        .unwrap()
    });

    let client = reqwest::blocking::Client::builder()
      .danger_accept_invalid_certs(true)
      .build()
      .unwrap();
    let response = client.get(format!("https://127.0.0.1:{}", mock_server.port()).as_str())
      .header(ACCEPT, "application/json").send();

    // Give the mock server some time
    thread::sleep(Duration::from_millis(100));

    let all_matched = mock_server.all_matched();
    let mismatches = mock_server.mismatches();
    mock_server.shutdown().unwrap();

    expect!(response.unwrap().status()).to(be_equal_to(200));
    expect!(all_matched).to(be_true());
    expect!(mismatches).to(be_equal_to(vec![]));
  }
}
