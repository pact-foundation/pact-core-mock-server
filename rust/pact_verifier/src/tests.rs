use expectest::prelude::*;
use expectest::expect;
use super::{FilterInfo, filter_interaction, filter_consumers, execute_state_change};
use pact_matching::models::*;
use pact_matching::models::provider_states::*;
use pact_matching::s;
use pact_consumer::prelude::*;
use pact_consumer::*;
use env_logger::*;
use crate::PactSource;
use std::panic::catch_unwind;
use crate::pact_broker::Link;
use maplit::*;
use serde_json::json;
use crate::callback_executors::HttpRequestProviderStateExecutor;

#[test]
fn if_no_interaction_filter_is_defined_returns_true() {
  let interaction = RequestResponseInteraction::default();
  expect!(filter_interaction(&interaction, &FilterInfo::None)).to(be_true());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: s!("bob"), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("fred")))).to(be_false());
}

#[test]
fn if_an_interaction_filter_is_defined_returns_true_if_the_description_does_match() {
  let interaction = RequestResponseInteraction { description: s!("bob"), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("bob")))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_description() {
  let interaction = RequestResponseInteraction { description: s!("bobby"), .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::Description(s!("bob.*")))).to(be_true());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_false_if_the_state_does_not_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&s!("bob")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State(s!("fred")))).to(be_false());
}

#[test]
fn if_an_interaction_state_filter_is_defined_returns_true_if_the_state_does_match() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&s!("bob")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State(s!("bob")))).to(be_true());
}

#[test]
fn uses_regexs_to_match_the_state() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State(s!("bob.*")))).to(be_true());
}

#[test]
fn if_the_state_filter_is_empty_returns_false_if_the_interaction_state_is_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State(s!("")))).to(be_false());
}

#[test]
fn if_the_state_filter_is_empty_returns_true_if_the_interaction_state_is_not_defined() {
  let interaction = RequestResponseInteraction { provider_states: vec![], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::State(s!("")))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_must_match_both() {
  let interaction = RequestResponseInteraction { description: s!("freddy"), provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_true());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_provider_state_does_not_match() {
  let interaction = RequestResponseInteraction { description: s!("freddy"), provider_states: vec![ ProviderState::default(&s!("boddy")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_the_description_does_not_match() {
  let interaction = RequestResponseInteraction { description: s!("frebby"), provider_states: vec![ ProviderState::default(&s!("bobby")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
}

#[test]
fn if_the_state_filter_and_interaction_filter_is_defined_is_false_if_both_do_not_match() {
  let interaction = RequestResponseInteraction { description: s!("joe"), provider_states: vec![ ProviderState::default(&s!("author")) ], .. RequestResponseInteraction::default() };
  expect!(filter_interaction(&interaction, &FilterInfo::DescriptionAndState(s!(".*ddy"), s!("bob.*")))).to(be_false());
}

#[test]
fn if_no_consumer_filter_is_defined_returns_true() {
  let consumers = vec![];
  let result = Err(s!(""));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_false_if_the_consumer_name_does_not_match() {
  let consumers = vec![s!("fred"), s!("joe")];
  let result = Ok((Box::new(RequestResponsePact { consumer: Consumer { name: s!("bob") }, .. RequestResponsePact::default() }) as Box<dyn Pact>, None, PactSource::Unknown));
  expect!(filter_consumers(&consumers, &result)).to(be_false());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_result_is_an_error() {
  let consumers = vec![s!("fred"), s!("joe")];
  let result = Err(s!(""));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[test]
fn if_a_consumer_filter_is_defined_returns_true_if_the_consumer_name_does_match() {
  let consumers = vec![s!("fred"), s!("joe"), s!("bob")];
  let result = Ok((Box::new(RequestResponsePact { consumer: Consumer { name: s!("bob") }, .. RequestResponsePact::default() }) as Box<dyn Pact>, None, PactSource::Unknown));
  expect!(filter_consumers(&consumers, &result)).to(be_true());
}

#[tokio::test]
async fn test_state_change_with_parameters() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request", |i| {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"A\":\"1\",\"B\":\"2\",\"action\":\"setup\",\"state\":\"TestState\"}");
      i.response.status(200);
    })
    .start_mock_server();

  let provider_state = ProviderState {
    name: s!("TestState"),
    params: hashmap!{
        s!("A") => json!("1"),
        s!("B") => json!("2")
      }
  };

  let provider_state_executor = HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  };
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, &provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[tokio::test]
async fn test_state_change_with_parameters_in_query() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request with params in the query string", |i| {
      i.request.method("POST");
      i.request.path("/");
      i.request.query_param("state", "TestState");
      i.request.query_param("action", "setup");
      i.request.query_param("A", "1");
      i.request.query_param("B", "2");
      i.response.status(200);
    })
    .start_mock_server();

  let provider_state = ProviderState {
    name: s!("TestState"),
    params: hashmap!{
        s!("A") => json!("1"),
        s!("B") => json!("2")
      }
  };

  let provider_state_executor = HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    state_change_body: false,
    .. HttpRequestProviderStateExecutor::default()
  };
  let client = reqwest::Client::new();

  let result = execute_state_change(&provider_state, true,
                                    None, &client, &provider_state_executor).await;
  expect!(result.clone()).to(be_ok());
}

#[tokio::test]
async fn test_state_change_returning_json_values() {
  try_init().unwrap_or(());

  let server = PactBuilder::new("RustPactVerifier", "SomeRunningProvider")
    .interaction("a state change request which returns a map of values", |i| {
      i.request.method("POST");
      i.request.path("/");
      i.request.header("Content-Type", "application/json");
      i.request.body("{\"action\":\"setup\",\"state\":\"TestState\"}");
      i.response.status(200);
      i.response.header("Content-Type", "application/json");
      i.response.body("{\"a\": \"A\", \"b\": 100}");
    })
    .start_mock_server();

  let provider_state = ProviderState {
    name: s!("TestState"),
    params: hashmap!{}
  };

  let provider_state_executor = HttpRequestProviderStateExecutor {
    state_change_url: Some(server.url().to_string()),
    .. HttpRequestProviderStateExecutor::default()
  };
  let client = reqwest::Client::new();
  let result = execute_state_change(&provider_state, true,
                                    None, &client, &provider_state_executor).await;
  expect!(result.clone()).to(be_ok().value(hashmap! {
    "a".into() => json!("A"),
    "b".into() => json!(100)
  }));
}

#[test]
fn publish_result_does_nothing_if_not_from_broker() {
  try_init().unwrap_or(());

  let server_response = catch_unwind(|| {
    let mut runtime = tokio::runtime::Builder::new()
      .basic_scheduler()
      .enable_all()
      .build()
      .unwrap();

    runtime.block_on(async {
      let _server = PactBuilder::new("RustPactVerifier", "PactBroker")
        .interaction("publish results", |i| {
          i.request.method("POST");
          i.request.path("/");
          i.response.status(201);
        })
        .start_mock_server();

      let options = super::VerificationOptions {
        publish: true,
        provider_version: None,
        build_url: None,
        request_filter: None::<Box<super::NullRequestFilterExecutor>>,
        provider_tags: vec![],
        disable_ssl_verification: false
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
    .interaction("publish results", |i| {
      i.request.method("POST");
      i.request.path("/path/to/pact/verification");
      i.request.json_body(json_pattern!({
        "providerApplicationVersion": "1",
        "success": true,
        "verifiedBy": json!({
          "implementation": "Pact-Rust",
          "version": PACT_RUST_VERSION
        })
      }));
      i.response.status(201);
    })
    .start_mock_server();

  let options = super::VerificationOptions {
    publish: true,
    provider_version: Some("1".into()),
    build_url: None,
    request_filter: None::<Box<super::NullRequestFilterExecutor>>,
    provider_tags: vec![],
    disable_ssl_verification: false
  };
  let links = vec![
    Link {
      name: "pb:publish-verification-results".to_string(),
      href: Some(server.path("/path/to/pact/verification".to_string()).to_string()),
      templated: false
    }
  ];
  let source = PactSource::BrokerUrl("Test".to_string(), server.url().to_string(), None, links);
  super::publish_result(&vec![], &source, &options).await;
}
