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
  sources: Vec<PactSource>,
  state_change: Arc<HttpRequestProviderStateExecutor>,
  options: VerificationOptions<NullRequestFilterExecutor>,
  consumers: Vec<String>
}

impl VerifierHandle {
  /// Create a new verifier and return the handle to it
  pub fn new() -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new(),
      state_change: Arc::new(HttpRequestProviderStateExecutor::default()),
      options: VerificationOptions::default(),
      consumers: vec![]
    }
  }

  /// Retrieve the provider info from the handle
  pub fn provider_info(&self) -> ProviderInfo {
    self.provider.clone()
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
  #[allow(clippy::too_many_arguments)]
  pub fn add_pact_broker_source(
    &mut self,
    url: &str,
    provider_name: &str,
    enable_pending: bool,
    include_wip_pacts_since: Option<String>,
    provider_tags: Vec<String>,
    provider_branch: Option<String>,
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
        provider_branch,
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
        provider_branch,
        selectors,
        auth: None,
        links: vec![]
      });
    }
  }

  /// Update the provider state
  pub fn update_provider_state(
    &mut self,
    state_change_url: Option<String>,
    state_change_teardown: bool,
    state_change_body: bool
  ) {
    self.state_change = Arc::new(HttpRequestProviderStateExecutor {
      state_change_url,
      state_change_teardown,
      state_change_body
    })
  }

  /// Update the verification options
  pub fn update_verification_options(
    &mut self,
    publish: bool,
    provider_version: &str,
    build_url: Option<String>,
    provider_tags: Vec<String>,
    disable_ssl_verification: bool,
    request_timeout: u64
  ) {
    self.options = VerificationOptions {
      publish,
      provider_version: Some(provider_version.to_string()),
      build_url,
      request_filter: None::<Arc<NullRequestFilterExecutor>>,
      provider_tags,
      disable_ssl_verification,
      request_timeout,
      .. VerificationOptions::default()
    }
  }

  /// Update the consumer filter
  pub fn update_consumers(
    &mut self,
    consumers: Vec<String>
  ) {
    self.consumers = consumers
  }

  /// Execute the verifier
  pub fn execute(&self) -> i32 {
    let filter = FilterInfo::None;

    for s in &self.sources {
      debug!("Pact source to verify = {}", s);
    };

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
      if verify_provider_async(
        self.provider.clone(),
        self.sources.clone(),
        filter,
        self.consumers.clone(),
        self.options.clone(),
        &self.state_change.clone()
      ).await {
        0
      } else {
        1
      }
    })
  }
}

impl Default for VerifierHandle {
   fn default() -> Self {
       Self::new()
   }
}
