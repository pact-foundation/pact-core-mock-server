use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use env_logger::*;
use expectest::prelude::*;
use maplit::*;
use reqwest::Client;
use serde_json::Value;

use pact_consumer::prelude::*;
use pact_matching::models::read_pact;
use pact_models::provider_states::ProviderState;
use pact_verifier::{FilterInfo, NullRequestFilterExecutor, ProviderInfo, VerificationOptions, verify_pact};
use pact_verifier::callback_executors::{ProviderStateError, ProviderStateExecutor};

/// Get the path to one of our sample *.json files.
fn fixture_path(path: &str) -> PathBuf {
  env::current_dir()
    .expect("could not find current working directory")
    .join("tests")
    .join(path)
    .to_owned()
}

struct DummyProviderStateExecutor;

#[async_trait]
impl ProviderStateExecutor for DummyProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    _provider_state: &ProviderState,
    _setup: bool,
    _client: Option<&Client>
  ) -> Result<HashMap<String, Value>, ProviderStateError> {
    Ok(hashmap!{})
  }
}

#[tokio::test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
async fn verify_pact_with_match_values_matcher() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("consumer", "matchValuesService")
    .interaction("request requiring matching values", |i| {
      i.test_name("verify_pact_with_match_values_matcher");
      i.request.method("GET");
      i.request.path("/myapp/test");
      i.response.ok().content_type("application/json").body(r#"{
        "field1": "test string",
        "field2": false,
        "field3": {
          "nested1": {
            "0": {
              "value1": "1st test value",
              "value2": 99,
              "value3": 100.0
            },
            "2": {
              "value1": "2nd test value",
              "value2": 98,
              "value3": 102.0
            }
          }
        },
        "field4": 50
      }"#);
    })
    .start_mock_server();

  let provider = ProviderInfo {
    name: "MatchValuesProvider".to_string(),
    host: "127.0.0.1".to_string(),
    port: server.url().port(),
    .. ProviderInfo::default()
  };

  let pact_file = fixture_path("match-values.json");
  let pact = read_pact(pact_file.as_path()).unwrap();
  let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
  let provider_states = Arc::new(DummyProviderStateExecutor{});

  let result = verify_pact(&provider, &FilterInfo::None, pact, &options, &provider_states).await;

  expect!(result.get(0).unwrap().2.clone()).to(be_none());
}
