//! Provides a builder for constructing mock servers

use std::net::{Ipv6Addr, SocketAddr};
use std::panic::RefUnwindSafe;

use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::v4::pact::V4Pact;

use crate::mock_server::{MockServer, MockServerConfig};

/// Builder for constructing mock servers
pub struct MockServerBuilder {
  config: MockServerConfig,
  pact: V4Pact
}

impl MockServerBuilder {
  /// Construct a new builder
  pub fn new() -> Self {
    MockServerBuilder {
      config: Default::default(),
      pact: V4Pact::default()
    }
  }

  /// Add the Pact that the mock server will respond with
  pub fn with_v4_pact(&mut self, pact: V4Pact) -> &mut Self {
    self.pact = pact;
    self.config.pact_specification = PactSpecification::V4;
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
  pub fn bind_to(&mut self, address: &str) -> &mut Self {
    self.config.address = address.to_string();
    self
  }

  /// Start the mock server, consuming this builder and returning a mock server instance
  pub async fn start(&self) -> anyhow::Result<MockServer> {
    MockServer::create(self.pact.clone(), self.config.clone()).await
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::prelude::v4::{SynchronousHttp, V4Pact};
  use pact_models::v4::http_parts::HttpRequest;
  use pact_models::v4::interaction::V4Interaction;
  use reqwest::header::ACCEPT;

  use super::MockServerBuilder;

  #[test_log::test]
  fn basic_mock_server_test() {
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
}
