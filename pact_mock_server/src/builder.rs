//! Provides a builder for constructing mock servers

use std::net::Ipv4Addr;
#[allow(unused_imports)] use anyhow::{anyhow, Context};
use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::v4::pact::V4Pact;
#[cfg(feature = "plugins")] use pact_plugin_driver::catalogue_manager;
#[cfg(feature = "tls")] use rcgen::{CertifiedKey, generate_simple_self_signed};
#[cfg(feature = "tls")] use rustls::crypto::ring::default_provider;
#[cfg(feature = "tls")] use rustls::crypto::CryptoProvider;
#[cfg(feature = "tls")] use rustls::pki_types::PrivateKeyDer;
#[cfg(feature = "tls")] use rustls::ServerConfig;
#[allow(unused_imports)] use tracing::warn;

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

  /// Sets the mock server to bind to the given port on the IP4
  /// loopback adapter (ip4-localhost, `127.0.0.1`). Specify 0 for the port to get a random OS assigned
  /// port. This is what you would mostly want with a mock server in a test, otherwise your test
  /// could fail with port conflicts.
  pub fn bind_to_ip4_port(mut self, port: u16) -> Self {
    self.config.address = format!("{}:{}", Ipv4Addr::LOCALHOST, port);
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

  /// If TLS has been configured for this builder
  pub fn tls_configured(&self) -> bool {
    # [cfg(feature = "tls")]
    { self.config.tls_config.is_some() }

    #[cfg(not(feature = "tls"))]
    { false }
  }

  /// Provide the private key and certificates in PEM format used to setup the TLS connection.
  #[cfg(feature = "tls")]
  pub fn with_tls_certs(mut self, certificates: &str, private_key: &str) -> anyhow::Result<Self> {
    let mut k = private_key.as_bytes();
    let private_key =  rustls_pemfile::pkcs8_private_keys(&mut k)
      .next()
      .ok_or_else(|| anyhow!("No private key found in input"))?
      .context("Failed to read private key from input")?;
    let mut c = certificates.as_bytes();
    let mut certs = vec![];
    for c in rustls_pemfile::certs(&mut c) {
      certs.push(c.context("Failed to read certificate from input")?);
    }

    if CryptoProvider::get_default().is_none() {
      warn!("No TLS cryptographic provided has been configured, defaulting to the standard FIPS provider");
      CryptoProvider::install_default(default_provider())
        .map_err(|_| anyhow!("Failed to install the standard FIPS provider"))?;
    }

    let tls_config = ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(certs, private_key.into())?;
    self.config.tls_config = Some(tls_config);
    Ok(self)
  }

  /// Use a generated self-signed certificate for TLS
  #[cfg(feature = "tls")]
  pub fn with_self_signed_tls(mut self) -> anyhow::Result<Self> {
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(["localhost".to_string()])?;
    let private_key = PrivateKeyDer::try_from(key_pair.serialize_der())
      .map_err(|err| anyhow!(err))?;
    let tls_config = ServerConfig::builder()
      .with_no_client_auth()
      .with_single_cert(vec![ cert.der().clone() ], private_key)?;
    self.config.tls_config = Some(tls_config);
    Ok(self)
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

  /// Returns true if the address is not empty
  pub fn address_assigned(&self) -> bool {
    !self.config.address.is_empty()
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

  /// Starts the mockserver, consuming this builder and registers it with the server manager.
  /// The mock server tasks will be spawned on the server manager's runtime.
  /// Returns the mock server instance.
  pub fn attach_to_manager(self, manager: &mut ServerManager) -> anyhow::Result<MockServer> {
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
  #[cfg(feature = "tls")] use rustls::crypto::ring::default_provider;
  #[cfg(feature = "tls")] use rustls::crypto::CryptoProvider;

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

      let mut mock_server = runtime.block_on(async {
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

    let mut mock_server = runtime.block_on(async {
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
  #[cfg(feature = "tls")]
  fn basic_mock_server_https_test() {
    let _ = CryptoProvider::install_default(default_provider());
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

    let mut mock_server = runtime.block_on(async {
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

  const PRIVATE_KEY: &str = r#"-----BEGIN PRIVATE KEY-----
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
imW0tFuzqWoLfqMFwrD2eXCVTGA5WQ==
-----END PRIVATE KEY-----"#;

  const CERT: &str = r#"-----BEGIN CERTIFICATE-----
MIIFkTCCA3mgAwIBAgIUIb6tnW0f3EmF0CMSiXV/K7Fhn38wDQYJKoZIhvcNAQEL
BQAwWDELMAkGA1UEBhMCQVUxDDAKBgNVBAgMA1ZJQzENMAsGA1UEBwwETWVsYjEL
MAkGA1UECgwCU0IxCzAJBgNVBAsMAlBGMRIwEAYDVQQDDAlsb2NhbGhvc3QwHhcN
MjQwNjE3MjMxMjQyWhcNMzQwNjE1MjMxMjQyWjBYMQswCQYDVQQGEwJBVTEMMAoG
A1UECAwDVklDMQ0wCwYDVQQHDARNZWxiMQswCQYDVQQKDAJTQjELMAkGA1UECwwC
UEYxEjAQBgNVBAMMCWxvY2FsaG9zdDCCAiIwDQYJKoZIhvcNAQEBBQADggIPADCC
AgoCggIBAJNw50IOBAzP0yoJ1/c+LWtllcu0QdhUlWIundxdfpkmEwlT3i8i8//N
XUjQ0myiQeAbKyLl8Br0SE52jITOTcPwR80/qF1LUBytwGnSqtZZBkmwWPDlthxS
2zIPmg/kwXXvTZJCxoswdx96lYyslxn9rnqnRGrFsv4LVG/ljTRyuaMBDa1YvF7l
b/TBTYcTwla5DnD63j8LG2nk6GSo7JIYBy32TF0LseNnMWYjPzQxIn5APYrokX9q
jSb46Z+nSlcJDh6qJ/z8vfDeUtu06BImuMOs0LVXRmEobV+ZtaYBX3NcC/9X3Dfn
dvbj70c7eniFXvGRVb+51YMNr9Z+Zv+g2UVx0WYJVqMhw3SmBINuSx46B+508uMY
DWYvCQhe4VKN28mtejspRV80Nq4MQUqeO/Fbi7Nlxe8gQRrCBZlwk7SKDh76NO3R
wU2Vk+4KNUyKFUgP8PkAQbzxt2PySX5zpSF6dEujwZB4DU78KkgwsNBvzTCvTNfH
Rx/5/1zMDV4/BWB/7hoMWvhb8biNt9hAjaIlNSjAofR8Z+RzQu93jpzllFVNALQm
j7Nt5MrH9zkfkL3G7sGx6n2NHC6f9H3HUJFZng/7wIkOXNBm+f/pkFgtLWT5iNsy
0jcrxo8D0dnaS9kRdNRQ5sNN9gRGSkvca1kVitJBXUpvYI3FxA2BAgMBAAGjUzBR
MB0GA1UdDgQWBBRaclnct+JATeoibBXx2lCnS1obBTAfBgNVHSMEGDAWgBRaclnc
t+JATeoibBXx2lCnS1obBTAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUA
A4ICAQCOHn9y8w7wshdp07p2YiJ1gXKalYCR7NQldyvis/itTp3Xc+TGfriFBfg0
JdEhpI86eb+19sDB98kWG7pZ3DOVV58Twx357pNWOcdl9Qz40qOvclMu4KPBGoon
falI40suBg9p0UObRr4+WP8YmSU210jJ/vUdpJRESQ6ZlTz209atURbtnHyQ64ss
JxVnboaQaHCYtRx6krpw5hlyc7DUk7gL695vkXzXYZ41L6ZxmprqDxnkGYfwxz8E
sdOFIyBL+b0FjEJPZ6zbzdpgfIi//zk2roHl4txt/hXhTWqrtg/3OQaPSOa5zikQ
hRZZCXyC6yT+cb3/4XhsTDnYSEcDSyiQhCGFvMtC//dqX/0A/h5vsSNIktdXmtqX
oOTTFjEvnT4RY1cwE0hYcqZTRBNbvZa8IhvrM76pKJlZoHXTuD2E6J26SNRbFd7U
FqCiIi+UBzTecbn7B+fQVT2zwCTo19HZ7lps4vyq8f5yNh6yO5jaHlr8dbP/aGNT
Q+JdJonVTKPHZk/kcxzYc7sRXokEzEeknjbLsI+8QyWuPB2kjmpaE6bK8NcPiGGf
jp9nJakYPl9nMMdHRHKNXo+jxR49Ww4sikVl0oCGC8I3BzlAy6vdRMBekPayxU+Y
ZSwZXle550Ns2jdFLpdSoFOHWsbPbsILG6ZXTlG9sJIZwujoYQ==
-----END CERTIFICATE-----"#;

  #[test_log::test(tokio::test(flavor = "multi_thread"))]
  #[cfg(feature = "tls")]
  async fn basic_mock_server_https_test_with_provided_cert() -> anyhow::Result<()> {
    let _ = CryptoProvider::install_default(default_provider());
    let pact = V4Pact {
      interactions: vec![ SynchronousHttp::default().boxed_v4() ],
      .. V4Pact::default()
    };

    let mut mock_server = MockServerBuilder::new()
      .bind_to("127.0.0.1:0")
      .with_v4_pact(pact)
      .with_tls_certs(CERT, PRIVATE_KEY)
      .unwrap()
      .start_https()
      .await
      .unwrap();

    let client = reqwest::Client::builder()
      .danger_accept_invalid_certs(true)
      .build()
      .unwrap();
    let response = client.get(format!("https://127.0.0.1:{}", mock_server.port()).as_str())
      .header(ACCEPT, "application/json")
      .send()
      .await;

    let all_matched = mock_server.all_matched();
    let mismatches = mock_server.mismatches();
    mock_server.shutdown().unwrap();

    expect!(response.unwrap().status()).to(be_equal_to(200));
    expect!(all_matched).to(be_true());
    expect!(mismatches).to(be_equal_to(vec![]));

    Ok(())
  }
}
