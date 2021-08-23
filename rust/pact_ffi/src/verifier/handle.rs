//! Handle interface to creating a verifier

use std::sync::Arc;

use log::debug;

use pact_verifier::{FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, VerificationOptions, verify_provider_async};
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

  /// Execute the verifier
  ///
  /// This will return an integer value based on the status of the verification:
  /// * 0 - verification was successful
  /// * 1 - verification was not successful
  /// * 2 - failed to run the verification
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
      verify_provider_async(
        self.provider.clone(),
        self.sources.clone(),
        filter,
        vec![],
        options,
        &provider_state_executor
      ).await
    })
      .map(|result| if result { 0 } else { 2 })
      .unwrap_or(2)
  }
}
