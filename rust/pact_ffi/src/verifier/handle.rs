//! Handle interface to creating a verifier

use std::sync::Arc;

use log::debug;

use pact_models::prelude::HttpAuth;
use pact_verifier::{FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, VerificationOptions, verify_provider_async, ConsumerVersionSelector};
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;

#[derive(Debug, Clone)]
/// Wraps a Pact verifier
pub struct VerifierHandle {
  provider: ProviderInfo,
  sources: Vec<PactSource>
}

impl VerifierHandle {
  /// Create a new verifier and return the handle to it
  pub fn new() -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new()
    }
  }

  /// Update the provider info
  pub fn update_provider_info(
    &mut self,
    name: String,
    scheme: String,
    host: String,
    port: u16,
    path: String
  ) {
    self.provider = ProviderInfo {
      name,
      protocol: scheme,
      host,
      port: if port == 0 { None } else { Some(port) },
      path
    }
  }

  /// Add a file source to be verified
  pub fn add_file_source(&mut self, file: &str) {
    self.sources.push(PactSource::File(file.to_string()));
  }

  /// Add a directory source to be verified. This will verify all pact files in the directory.
  pub fn add_directory_source(&mut self, dir: &str) {
    self.sources.push(PactSource::Dir(dir.to_string()));
  }

  /// Add a URL source to be verified. This will fetch the pact file from the URL. If a username
  /// and password is given, then basic authentication will be used when fetching the pact file.
  /// If a token is provided, then bearer token authentication will be used.
  pub fn add_url_source(&mut self, url: &str, auth: &HttpAuth) {
    if !auth.is_none() {
      self.sources.push(PactSource::URL(url.to_string(), Some(auth.clone())));
    } else {
      self.sources.push(PactSource::URL(url.to_string(), None));
    }
  }

  /// Add a Pact broker source to be verified. This will fetch all the pact files from the broker
  /// that match the provider name. If a username
  /// and password is given, the basic authentication will be used when fetching the pact file.
  /// If a token is provided, then bearer token authentication will be used.
  pub fn add_pact_broker_source(
    &mut self,
    url: &str,
    provider_name: &str,
    enable_pending: bool,
    include_wip_pacts_since: Option<String>,
    provider_tags: Vec<String>,
    selectors: Vec<ConsumerVersionSelector>,
    auth: &HttpAuth
  ) {
    if !auth.is_none() {
      self.sources.push(PactSource::BrokerWithDynamicConfiguration {
        provider_name: provider_name.to_string(),
        broker_url: url.to_string(),
        enable_pending,
        include_wip_pacts_since,
        provider_tags,
        selectors,
        auth: Some(auth.clone()),
        links: vec![]
      });
    } else {
      self.sources.push(PactSource::BrokerWithDynamicConfiguration {
        provider_name: provider_name.to_string(),
        broker_url: url.to_string(),
        enable_pending,
        include_wip_pacts_since,
        provider_tags,
        selectors,
        auth: None,
        links: vec![]
      });
    }
  }

  /// Execute the verifier
  pub fn execute(&self) -> i32 {
    let filter = FilterInfo::None;
    let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
      state_change_url: None,
      state_change_teardown: false,
      state_change_body: false
    });

    let options = VerificationOptions {
      request_filter: None::<Arc<NullRequestFilterExecutor>>,
      .. VerificationOptions::default()
    };

    for s in &self.sources {
      debug!("Pact source to verify = {}", s);
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
      if verify_provider_async(
        self.provider.clone(),
        self.sources.clone(),
        filter,
        vec![],
        options,
        &provider_state_executor
      ).await {
        0
      } else {
        1
      }
    })
  }
}
