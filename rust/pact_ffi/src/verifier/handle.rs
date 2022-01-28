//! Handle interface to creating a verifier

use std::sync::Arc;

use log::debug;
use pact_models::prelude::HttpAuth;

use pact_verifier::{
  ConsumerVersionSelector,
  FilterInfo,
  NullRequestFilterExecutor,
  PactSource,
  ProviderInfo,
  PublishOptions,
  VerificationOptions,
  verify_provider_async
};
use pact_verifier::callback_executors::HttpRequestProviderStateExecutor;
use pact_verifier::metrics::VerificationMetrics;
use pact_verifier::verification_result::VerificationExecutionResult;

#[derive(Debug, Clone)]
/// Wraps a Pact verifier
pub struct VerifierHandle {
  provider: ProviderInfo,
  sources: Vec<PactSource>,
  filter: FilterInfo,
  state_change: Arc<HttpRequestProviderStateExecutor>,
  verification_options: VerificationOptions<NullRequestFilterExecutor>,
  publish_options: Option<PublishOptions>,
  consumers: Vec<String>,
  /// Calling application name and version
  calling_app: Option<(String, String)>,
  /// Output captured from the verifier
  verifier_output: VerificationExecutionResult
}

impl VerifierHandle {
  /// Create a new verifier and return the handle to it (deprecated, use new_for_application)
  #[deprecated(since = "0.1.4", note = "Use new_for_application instead")]
  pub fn new() -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new(),
      filter: FilterInfo::None,
      state_change: Arc::new(HttpRequestProviderStateExecutor::default()),
      verification_options: VerificationOptions::default(),
      publish_options: None,
      consumers: vec![],
      calling_app: None,
      verifier_output: VerificationExecutionResult::new()
    }
  }

  /// Create a new verifier and return the handle to it
  pub fn new_for_application(calling_app_name: &str, calling_app_version: &str) -> VerifierHandle {
    VerifierHandle {
      provider: ProviderInfo::default(),
      sources: Vec::new(),
      filter: FilterInfo::None,
      state_change: Arc::new(HttpRequestProviderStateExecutor::default()),
      verification_options: VerificationOptions::default(),
      publish_options: None,
      consumers: vec![],
      calling_app: Some((calling_app_name.to_string(), calling_app_version.to_string())),
      verifier_output: VerificationExecutionResult::new()
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

  /// Update the filter info
  pub fn update_filter_info(
    &mut self,
    filter_description: String,
    filter_state: String,
    filter_no_state: bool
  ) {
    self.filter = if !filter_description.is_empty() && (!filter_state.is_empty() || filter_no_state) {
        if !filter_state.is_empty() {
            FilterInfo::DescriptionAndState(filter_description, filter_state)
        } else {
            FilterInfo::DescriptionAndState(filter_description, String::new())
        }
    } else if !filter_description.is_empty() {
        FilterInfo::Description(filter_description)
    } else if !filter_state.is_empty() {
        FilterInfo::State(filter_state)
    } else if filter_no_state {
        FilterInfo::State(String::new())
    } else {
        FilterInfo::None
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
    enable_pending: bool,
    include_wip_pacts_since: Option<String>,
    provider_tags: Vec<String>,
    provider_branch: Option<String>,
    selectors: Vec<ConsumerVersionSelector>,
    auth: &HttpAuth
  ) {
    if !auth.is_none() {
      self.sources.push(PactSource::BrokerWithDynamicConfiguration {
        provider_name: self.provider.name.clone(),
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
        provider_name: self.provider.name.clone(),
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

  /// Update options used when running a verification
  /// 
  /// # Args
  /// 
  /// - disable_ssl_verification - Disable SSL verification on all HTTPS requests
  /// - request_timeout - Timeout for all requests to the provider
  pub fn update_verification_options(
    &mut self,
    disable_ssl_verification: bool,
    request_timeout: u64
  ) {
    self.verification_options = VerificationOptions {
      request_filter: None::<Arc<NullRequestFilterExecutor>>,
      disable_ssl_verification,
      request_timeout
    }
  }

  /// Update the details used when publishing results
  /// 
  /// # Args
  /// 
  /// - `provider_version` - Version of the provider
  pub fn update_publish_options(
    &mut self,
    provider_version: &str,
    build_url: Option<String>,
    provider_tags: Vec<String>,
    provider_branch: Option<String>
  ) {
    self.publish_options = Some(PublishOptions {
      provider_version: Some(provider_version.to_string()),
      build_url,
      provider_tags,
      provider_branch,
    })
  }

  /// Update the consumer filter
  pub fn update_consumers(
    &mut self,
    consumers: Vec<String>
  ) {
    self.consumers = consumers
  }

  /// Execute the verifier
  ///
  /// This will return an integer value based on the status of the verification:
  /// * 0 - verification was successful
  /// * 1 - verification was not successful
  /// * 2 - failed to run the verification
  ///
  /// Anu captured output from the verification will be stored against this handle
  pub fn execute(&mut self) -> i32 {
    for s in &self.sources {
      debug!("Pact source to verify = {s}");
    };

    let (calling_app_name, calling_app_version) = self.calling_app.clone().unwrap_or_else(|| {
      ("pact_ffi".to_string(), env!("CARGO_PKG_VERSION").to_string())
    });
    let runtime = tokio::runtime::Runtime::new().unwrap();
    match runtime.block_on(async {
      verify_provider_async(
        self.provider.clone(),
        self.sources.clone(),
        self.filter.clone(),
        self.consumers.clone(),
        &self.verification_options,
        self.publish_options.as_ref(),
        &self.state_change.clone(),
        Some(VerificationMetrics {
          test_framework: "pact_ffi".to_string(),
          app_name: calling_app_name.clone(),
          app_version: calling_app_version.clone()
        })
      ).await
    }) {
      Ok(result) => {
        self.verifier_output = result.clone();
        if result.result { 0 } else { 1 }
      }
      Err(_) => 2
    }
  }
}

impl Default for VerifierHandle {
   fn default() -> Self {
       Self::new()
   }
}
