//! Provides a builder for constructing mock servers

use std::net::{Ipv6Addr, SocketAddr};
use std::panic::RefUnwindSafe;
use anyhow::anyhow;

use pact_models::pact::Pact;
use pact_models::PactSpecification;
use pact_models::v4::pact::V4Pact;
use tokio::runtime::{Handle, TryCurrentError};
use tracing::warn;
use uuid::Uuid;

use crate::MANAGER;
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

  /// Start the mock server, consuming this builder and returning a mock server instance
  pub async fn start(&self) -> anyhow::Result<MockServer> {
    let addr = SocketAddr::new(Ipv6Addr::LOCALHOST.into(), 0);
    let config = self.config.clone();

    MockServer::create(self.pact.clone(), addr, config).await
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
}
