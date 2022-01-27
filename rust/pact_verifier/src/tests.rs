use std::panic::catch_unwind;
use std::sync::Arc;

use env_logger::*;
use expectest::expect;
use expectest::prelude::*;
use maplit::*;
use serde_json::json;

use pact_consumer::*;
use pact_consumer::prelude::*;
use pact_models::Consumer;
use pact_models::pact::Pact;
use pact_models::PACT_RUST_VERSION;
use pact_models::provider_states::*;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;

use crate::callback_executors::HttpRequestProviderStateExecutor;
use crate::pact_broker::Link;
use crate::PactSource;

use super::{execute_state_change, filter_consumers, filter_interaction, FilterInfo};

#[test]
fn if_no_interaction_filter_is_defined_returns_true() {
  let interaction = RequestResponseInteraction::default();
  expect!(filter_interaction(&interaction, &FilterInfo::None)).to(be_true());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: "bob".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("fred".to_string()))).to(be_false());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_true_if_the_description_does_match() {
  let interaction = RequestResponseInteraction { description: "bob".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("bob".to_string()))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_description() {
  let interaction = RequestResponseInteraction { description: "bobby".to_string(), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description("bob.*".to_string()))).to(be_true());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_false_if_the_state_does_not_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bob".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("fred".to_string()))).to(be_false());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_true_if_the_state_does_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bob".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("bob".to_string()))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_state() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("bob.*".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_is_empty_returns_false_if_the_interaction_state_is_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_is_empty_returns_true_if_the_interaction_state_is_not_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State("".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_must_match_both() {
  let interaction = RequestResponseInteraction { description: "freddy".to_string(), provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_provider_state_does_not_match() {
  let interaction = RequestResponseInteraction { description: "freddy".to_string(), provider_states: vec![ ProviderState::default(&"boddy".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: "frebby".to_string(), provider_states: vec![ ProviderState::default(&"bobby".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_both_do_not_match() {
  let interaction = RequestResponseInteraction { description: "joe".to_string(), provider_states: vec![ ProviderState::default(&"author".to_string()) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(".*ddy".to_string(), "bob.*".to_string()))).to(be_false());
}

#[test]
fn if_no_consumer_filter_is_defined_returns_true() {
  let consumers = vec![];
  let result = Err("".to_string());
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_false_if_the_consumer_name_does_not_match() {
  let consumers = vec!["fred".to_string(), "joe".to_string()];
  let result = Ok((Box::new(RequestResponsePact { consumer: Consumer { name: "bob".to_string() }, .. RequestResponsePact::default() }) as Box<dyn Pact + Send + Sync>, None, PactSource::Unknown));
  expect!(filter_consumers(&consumers, &result)).to(be_false());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_result_is_an_error() {
  let consumers = vec!["fred".to_string(), "joe".to_string()];
  let result = Err("".to_string());
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_consumer_name_does_match() {
  let consumers = vec!["fred".to_string(), "joe".to_string(), "bob".to_string()];
  let result = Ok((Box::new(RequestResponsePact { consumer: Consumer { name: "bob".to_string() }, .. RequestResponsePact::default() }) as Box<dyn Pact + Send + Sync>, None, PactSource::Unknown));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[tokio::test]
async fn test_state_change_with_parameters() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request", "", |mut i| async move {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"params\":{\"A\":\"1\",\"B\":\"2\"},\"action\":\"setup\",\"state\":\"TestState\"}");
      i.response.status(200);
      i
    })
    .await
    .start_mock_server();

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{
        "A".to_string() => json!("1"),
        "B".to_string() => json!("2")
      }
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[tokio::test]
async fn test_state_change_with_parameters_in_query() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request with params in the query string", "", |mut i| async move {
      i.comment("testing state change with parameters in the query");
      i.test_name("test_state_change_with_parameters_in_query");
      i.request.method("POST");
      i.request.path("/");
      i.request.query_param("state", "TestState");
      i.request.query_param("action", "setup");
      i.request.query_param("A", "1");
      i.request.query_param("B", "2");
      i.response.status(200);
      i
    })
    .await
    .start_mock_server();

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{
        "A".to_string() => json!("1"),
        "B".to_string() => json!("2")
      }
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    state_change_body: false,
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();

  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[tokio::test]
async fn test_state_change_returning_json_values() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request which returns a map of values", "", |mut i| async move {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"action\":\"setup\",\"state\":\"TestState\",\"params\":{}}");
      i.response.status(200);
      i.response.header("Content-Type", "application/json");
      i.response.body("{\"a\": \"A\", \"b\": 100}");
      i
    })
    .await
    .start_mock_server();

  let provider_state = ProviderState {
    name: "TestState".to_string(),
    params: hashmap!{}
  };

  let provider_state_executor = Arc::new(HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  });
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, provider_state_executor).await;
  expect!(result.clone()).to(be_ok().value(hashmap! {
    "a".into() => json!("A"),
    "b".into() => json!(100)
  }));
}

#[test]
fn publish_result_does_nothing_if_not_from_broker() {
  try_init().unwrap_or(());

  let server_response = catch_unwind(|| {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_all()
      .build()
      .unwrap();

    runtime.block_on(async {
      let _server = PactBuilder::new("RustPactVerifier", "PactBroker")
        .interaction("publish results", "", |mut i| async move {
          i.request.method("POST");
          i.request.path("/");
          i.response.status(201);
          i
        })
        .await
        .start_mock_server();

      let options = super::PublishOptions {
        provider_version: None,
        build_url: None,
        provider_tags: vec![],
        .. super::PublishOptions::default()
      };
      super::publish_result(&vec![], &PactSource::File("/tmp/test".into()), &options).await;
    })
  });
  expect!(server_response).to(be_err());
}

#[tokio::test]
async fn publish_successful_result_to_broker() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "PactBroker")
    .interaction("publish results", "", |mut i| async move {
      i.request.method("POST");
      i.request.path("/path/to/pact/verification");
      i.request.json_body(json_pattern!({
        "providerApplicationVersion": "1",
        "success": true,
        "testResults": [
          { "interactionId": "1", "success": true }
        ],
        "verifiedBy": json!({
          "implementation": "Pact-Rust",
          "version": PACT_RUST_VERSION
        })
      }));
      i.response.status(201);
      i
    })
    .await
    .start_mock_server();

  let options = super::PublishOptions {
    provider_version: Some("1".into()),
    .. super::PublishOptions::default()
  };

  let links = vec![
    Link {
      name: "pb:publish-verification-results".to_string(),
      href: Some(server.path("/path/to/pact/verification".to_string()).to_string()),
      templated: false,
      title: None
    }
  ];
  
  let source = PactSource::BrokerUrl("Test".to_string(), server.url().to_string(), None, links);
  super::publish_result(&vec![(Some("1".to_string()), Ok(()))], &source, &options).await;
}
