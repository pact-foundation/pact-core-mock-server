// use std::collections::HashMap;
// use std::env;
// use std::path::PathBuf;
// use std::sync::Arc;
//
// use async_trait::async_trait;
// use env_logger::*;
// use expectest::prelude::*;
// use maplit::*;
// use reqwest::Client;
// use serde_json::Value;
//
// use pact_consumer::*;
// use pact_consumer::prelude::*;
// use pact_models::pact::read_pact;
// use pact_models::provider_states::ProviderState;
// use pact_verifier::{FilterInfo, NullRequestFilterExecutor, ProviderInfo, VerificationOptions, verify_pact_internal};
// use pact_verifier::callback_executors::ProviderStateExecutor;
//
// /// Get the path to one of our sample *.json files.
// fn fixture_path(path: &str) -> PathBuf {
//   env::current_dir()
//     .expect("could not find current working directory")
//     .join("tests")
//     .join(path)
//     .to_owned()
// }
//
// struct DummyProviderStateExecutor;
//
// #[async_trait]
// impl ProviderStateExecutor for DummyProviderStateExecutor {
//   async fn call(
//     self: Arc<Self>,
//     _interaction_id: Option<String>,
//     _provider_state: &ProviderState,
//     _setup: bool,
//     _client: Option<&Client>
//   ) -> anyhow::Result<HashMap<String, Value>> {
//     Ok(hashmap!{})
//   }
//
//   fn teardown(self: &Self) -> bool {
//         return false
//     }
// }
//
// #[tokio::test]
// async fn verify_pact_with_match_values_matcher() {
//   try_init().unwrap_or(());
//
//   let server = PactBuilder::new("consumer", "matchValuesService")
//     .interaction("request requiring matching values", "", |mut i| async move {
//       i.test_name("verify_pact_with_match_values_matcher");
//       i.request.method("GET");
//       i.request.path("/myapp/test");
//       i.response.ok().content_type("application/json").body(r#"{
//         "field1": "test string",
//         "field2": false,
//         "field3": {
//           "nested1": {
//             "0": {
//               "value1": "1st test value",
//               "value2": 99,
//               "value3": 100.0
//             },
//             "2": {
//               "value1": "2nd test value",
//               "value2": 98,
//               "value3": 102.0
//             }
//           }
//         },
//         "field4": 50
//       }"#);
//       i
//     })
//     .await
//     .start_mock_server(None);
//
//   let provider = ProviderInfo {
//     name: "MatchValuesProvider".to_string(),
//     host: "127.0.0.1".to_string(),
//     port: server.url().port(),
//     .. ProviderInfo::default()
//   };
//
//   let pact_file = fixture_path("match-values.json");
//   let pact = read_pact(pact_file.as_path()).unwrap();
//   let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
//   let provider_states = Arc::new(DummyProviderStateExecutor{});
//
//   let result = verify_pact_internal(
//     &provider,
//     &FilterInfo::None,
//     pact,
//     &options,
//     &provider_states,
//     false
//   ).await;
//
//   expect!(result.unwrap().results.get(0).unwrap().result.as_ref()).to(be_ok());
// }
//
// #[tokio::test]
// async fn verify_pact_with_attributes_with_special_values() {
//   try_init().unwrap_or(());
//
//   let server = PactBuilder::new_v4("book_consumer", "book_provider")
//     .interaction("create book request", "", |mut i| async move {
//       i.test_name("verify_pact_with_attributes_with_special_values");
//       i.request.method("POST");
//       i.request.path("/books");
//       i.request.content_type("application/json");
//
//       i.response.ok().content_type("application/json").json_body(json_pattern!({
//         "@context": "/api/contexts/Book",
//         "@id": "/api/books/0114b2a8-3347-49d8-ad99-0e792c5a30e6",
//         "@type": "Book",
//         "title": "Voluptas et tempora repellat corporis excepturi.",
//         "description": "Quaerat odit quia nisi accusantium natus voluptatem. Explicabo corporis eligendi ut ut sapiente ut qui quidem. Optio amet velit aut delectus. Sed alias asperiores perspiciatis deserunt omnis. Mollitia unde id in.",
//         "author": "Melisa Kassulke",
//         "%publicationDate%": "1999-02-13T00:00:00+07:00",
//         "reviews": []
//       }));
//       i
//     })
//     .await
//     .start_mock_server(None);
//
//   let provider = ProviderInfo {
//     name: "BookProvider".to_string(),
//     host: "127.0.0.1".to_string(),
//     port: server.url().port(),
//     .. ProviderInfo::default()
//   };
//
//   let pact_file = fixture_path("pact_with_special_chars.json");
//   let pact = read_pact(pact_file.as_path()).unwrap();
//   let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
//   let provider_states = Arc::new(DummyProviderStateExecutor{});
//
//   let result = verify_pact_internal(
//     &provider,
//     &FilterInfo::None,
//     pact,
//     &options,
//     &provider_states,
//     false
//   ).await;
//
//   expect!(result.unwrap().results.get(0).unwrap().result.as_ref()).to(be_ok());
// }
//
// #[tokio::test]
// async fn verifying_a_pact_with_pending_interactions() {
//   try_init().unwrap_or(());
//   let provider = ProviderInfo {
//     name: "PendingProvider".to_string(),
//     host: "127.0.0.1".to_string(),
//     .. ProviderInfo::default()
//   };
//
//   let pact_file = fixture_path("v4-pending-pact.json");
//   let pact = read_pact(pact_file.as_path()).unwrap();
//   let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
//   let provider_states = Arc::new(DummyProviderStateExecutor{});
//
//   let result = verify_pact_internal(
//     &provider,
//     &FilterInfo::None,
//     pact,
//     &options,
//     &provider_states,
//     false
//   ).await;
//
//   expect!(result.as_ref().unwrap().results.get(0).unwrap().result.as_ref()).to(be_err());
//   expect!(result.as_ref().unwrap().results.get(0).unwrap().pending).to(be_true());
// }
//
// #[tokio::test]
// async fn verifying_a_pact_with_min_type_matcher_and_child_arrays() {
//   try_init().unwrap_or(());
//
//   let server = PactBuilder::new_v4("consumer", "Issue396Service")
//     .interaction("get data request", "", |mut i| async move {
//       i.test_name("verifying_a_pact_with_min_type_matcher_and_child_arrays");
//       i.request.method("GET");
//       i.request.path("/data");
//       i.response.ok().content_type("application/json").json_body(json_pattern!({
//           "parent": [
//             {
//               "child": [
//                 "a"
//               ]
//             },
//             {
//               "child": [
//                 "a"
//               ]
//             }
//           ]
//         }));
//       i
//     })
//     .await
//     .start_mock_server(None);
//
//   let provider = ProviderInfo {
//     name: "Issue396Service".to_string(),
//     host: "127.0.0.1".to_string(),
//     port: server.url().port(),
//     .. ProviderInfo::default()
//   };
//
//   let pact_file = fixture_path("issue396.json");
//   let pact = read_pact(pact_file.as_path()).unwrap();
//   let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
//   let provider_states = Arc::new(DummyProviderStateExecutor{});
//
//   let result = verify_pact_internal(
//     &provider,
//     &FilterInfo::None,
//     pact,
//     &options,
//     &provider_states,
//     false
//   ).await;
//
//   expect!(result.unwrap().results.get(0).unwrap().result.as_ref()).to(be_ok());
// }
