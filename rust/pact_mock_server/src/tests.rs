use std::net::SocketAddr;
use expectest::expect;
use expectest::prelude::*;
use maplit::*;
use reqwest::header::ACCEPT;

use pact_matching::Mismatch;
use pact_models::bodies::OptionalBody;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRule;
use pact_models::prelude::v4::{SynchronousHttp, V4Pact};
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};

use crate::matching::{match_request, MatchResult};

use super::*;
use pact_models::v4::interaction::V4Interaction;

#[tokio::test]
async fn match_request_returns_a_match_for_identical_requests() {
    let request = HttpRequest::default();
    let interaction = SynchronousHttp { request: request.clone(), .. SynchronousHttp::default() };
    let interactions = vec![interaction.boxed_v4()];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request.clone(),
      interaction.response.clone())));
}

#[tokio::test]
async fn match_request_returns_a_not_found_for_no_interactions() {
    let request = HttpRequest::default();
    let interactions = vec![];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[tokio::test]
async fn match_request_returns_a_match_for_multiple_identical_requests() {
    let request = HttpRequest::default();
    let interaction = SynchronousHttp { request: request.clone(), .. SynchronousHttp::default() };
    let interaction2 = SynchronousHttp {
      description: "test2".to_string(),
      request: request.clone(),
      ..SynchronousHttp::default()
    };
    let interactions = vec![
      interaction.boxed_v4(),
      interaction2.boxed_v4()
    ];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[tokio::test]
async fn match_request_returns_a_match_for_multiple_requests() {
    let request = HttpRequest { method: "GET".to_string(), .. HttpRequest::default() };
    let request2 = HttpRequest { method: "POST".to_string(), path: "/post".to_string(), .. HttpRequest::default() };
    let interaction = SynchronousHttp { request: request.clone(), .. SynchronousHttp::default() };
    let interaction2 = SynchronousHttp {
      description: "test2".to_string(),
      request: request2.clone(),
      ..SynchronousHttp::default()
    };
    let interactions = vec![
      interaction.boxed_v4(),
      interaction2.boxed_v4()
    ];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[tokio::test]
async fn match_request_returns_a_mismatch_for_incorrect_request() {
    let request = HttpRequest::default();
    let expected_request = HttpRequest { query: Some(hashmap!{ "QueryA".to_string() => vec!["Value A".to_string()] }),
        .. HttpRequest::default() };
    let interaction = SynchronousHttp {
      request: expected_request,
      ..SynchronousHttp::default()
    };
    let interactions = vec![
      interaction.boxed_v4()
    ];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result.match_key()).to(be_equal_to("Request-Mismatch".to_string()));
}

#[tokio::test]
async fn match_request_returns_request_not_found_if_method_or_path_do_not_match() {
    let request = HttpRequest { method: "GET".to_string(), path: "/path".to_string(), .. HttpRequest::default() };
    let expected_request = HttpRequest { method: "POST".to_string(), path: "/otherpath".to_string(),
        .. HttpRequest::default() };
    let interaction = SynchronousHttp {
      request: expected_request,
      ..SynchronousHttp::default()
    };
    let interactions = vec![
      interaction.boxed_v4()
    ];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[tokio::test]
async fn match_request_returns_the_most_appropriate_mismatch_for_multiple_requests() {
    let request = HttpRequest { method: "GET".to_string(), path: "/".to_string(), body: OptionalBody::Present("This is a body".into(), None, None),
      .. HttpRequest::default() };
    let request2 = HttpRequest { method: "GET".to_string(), path: "/".to_string(), query: Some(hashmap!{
        "QueryA".to_string() => vec!["Value A".to_string()]
        }), body: OptionalBody::Present("This is a body".into(), None, None),
      .. HttpRequest::default() };
    let request3 = HttpRequest { method: "GET".to_string(), path: "/".to_string(), query: Some(hashmap!{
        "QueryA".to_string() => vec!["Value A".to_string()]
        }), body: OptionalBody::Missing, .. HttpRequest::default() };
    let interaction = SynchronousHttp { description: "test".to_string(), request: request.clone(), .. SynchronousHttp::default() };
    let interaction2 = SynchronousHttp { description: "test2".to_string(), request: request2.clone(), .. SynchronousHttp::default() };
    let interactions = vec![interaction.boxed_v4(), interaction2.boxed_v4()];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request3, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMismatch(interaction2.request,
        vec![Mismatch::BodyMismatch { path: "/".to_string(), expected: Some("This is a body".into()), actual: None,
        mismatch: "Expected body \'This is a body\' but was missing".to_string() }])));
}

#[tokio::test]
async fn match_request_supports_v2_matchers() {
    let request = HttpRequest { method: "GET".to_string(), path: "/".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }), body: OptionalBody::Present(
            r#"
            {
                "a": 100,
                "b": "one hundred"
            }
            "#.into(), None, None
        ), .. HttpRequest::default() };
    let expected_request = HttpRequest { method: "GET".to_string(), path: "/".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }),
        body: OptionalBody::Present(
            r#"
            {
                "a": 1000,
                "b": "One Thousand"
            }
            "#.into(), None, None
        ), matching_rules: matchingrules!{
          "body" => {
            "$.*" => [ MatchingRule::Type ]
          }
        },
      .. HttpRequest::default()
    };
    let interaction = SynchronousHttp { request: expected_request, .. SynchronousHttp::default() };
    let interactions = vec![interaction.boxed_v4()];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[tokio::test]
async fn match_request_supports_v2_matchers_with_xml() {
    let request = HttpRequest { method: "GET".to_string(), path: "/".to_string(), query: None,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/xml".to_string()] }), body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>world</foo>
            "#.into(), None, None
        ), .. HttpRequest::default() };
    let expected_request = HttpRequest { method: "GET".to_string(), path: "/".to_string(), query: None,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/xml".to_string()] }),
        body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>mars </foo>
            "#.into(), None, None
        ), matching_rules: matchingrules!{
          "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex("[a-z]+".into()) ]
          }
        },
      .. HttpRequest::default()
    };
    let interaction = SynchronousHttp { request: expected_request, .. SynchronousHttp::default() };
    let interactions = vec![interaction.boxed_v4()];
    let pact = V4Pact { interactions, .. V4Pact::default() };
    let result = match_request(&request, &pact).await;
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[test]
fn match_request_with_header_with_multiple_values() {
  let pact = V4Pact {
    interactions: vec![
      SynchronousHttp {
        request: HttpRequest {
          headers: Some(hashmap! {
            "accept".to_string() => vec!["application/hal+json".to_string(), "application/json".to_string()]
          }),
          .. HttpRequest::default()
        },
        .. SynchronousHttp::default()
      }.boxed_v4()
    ],
    .. V4Pact::default()
  };
  let mut manager = ServerManager::new();
  let id = "match_request_with_header_with_multiple_values".to_string();
  let port = manager.start_mock_server(id.clone(), pact.boxed(), 0, MockServerConfig::default()).unwrap();

  let client = reqwest::blocking::Client::new();
  let response = client.get(format!("http://127.0.0.1:{}", port).as_str())
    .header(ACCEPT, "application/hal+json, application/json").send();

  let mismatches = manager.find_mock_server_by_id(&id, &|_, ms| {
    ms.unwrap_left().mismatches()
  });
  manager.shutdown_mock_server_by_port(port);

  expect!(mismatches).to(be_some().value(vec![]));
  expect!(response.unwrap().status()).to(be_equal_to(200));
}

#[tokio::test]
async fn match_request_with_more_specific_request() {
  let request1 = HttpRequest { path: "/animals/available".into(), .. HttpRequest::default() };
  let request2 = HttpRequest { path: "/animals/available".into(), headers: Some(hashmap! {
      "Authorization".to_string() => vec!["Bearer token".to_string()]
    }),
    .. HttpRequest::default() };
  let interaction1 = SynchronousHttp {
    description: "test_more_general_request".into(),
    request: request1.clone(),
    response: HttpResponse { status: 401, .. HttpResponse::default() },
    .. SynchronousHttp::default()
  };
  let interaction2 = SynchronousHttp {
    description: "test_more_specific_request".into(),
    request: request2.clone(),
    response: HttpResponse { status: 200, .. HttpResponse::default() },
    .. SynchronousHttp::default()
  };

  let expected = interaction1.clone();
  let interactions = vec![interaction1.boxed_v4(), interaction2.boxed_v4()];
  let pact = V4Pact { interactions, .. V4Pact::default() };
  let result1 = match_request(&request1.clone(), &pact).await;
  expect!(result1).to(be_equal_to(MatchResult::RequestMatch(expected.request, expected.response)));

  let expected = interaction2.clone();
  let result2 = match_request(&request2.clone(), &pact).await;
  expect!(result2).to(be_equal_to(MatchResult::RequestMatch(expected.request, expected.response)));
}

#[test]
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
  let id = "basic_mock_server_test".to_string();
  let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
  let port = start_mock_server_for_transport(id.clone(), pact.boxed(), addr, "http", MockServerConfig::default()).unwrap();

  let client = reqwest::blocking::Client::new();
  let response = client.get(format!("http://127.0.0.1:{}", port).as_str())
    .header(ACCEPT, "application/json").send();

  let all_matched = mock_server_matched(port);
  let mismatches = mock_server_mismatches(port);
  shutdown_mock_server(port);

  expect!(all_matched).to(be_true());
  expect!(mismatches).to(be_some().value("[]"));
  expect!(response.unwrap().status()).to(be_equal_to(200));
}
