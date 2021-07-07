use expectest::expect;
use expectest::prelude::*;
use maplit::*;
use reqwest::header::ACCEPT;

use pact_matching::Mismatch;
use pact_matching::models::{Interaction, RequestResponseInteraction, RequestResponsePact};
use pact_models::bodies::OptionalBody;
use pact_models::matchingrules;
use pact_models::matchingrules::MatchingRule;
use pact_models::request::Request;
use pact_models::response::Response;

use crate::matching::{match_request, MatchResult};

use super::*;

#[test]
fn match_request_returns_a_match_for_identical_requests() {
    let request = Request::default();
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![&interaction as &dyn Interaction];
    let result = match_request(&request, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request.clone(),
      interaction.response.clone())));
}

#[test]
fn match_request_returns_a_not_found_for_no_interactions() {
    let request = Request::default();
    let interactions = vec![];
    let result = match_request(&request, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_a_match_for_multiple_identical_requests() {
    let request = Request::default();
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interaction2 = RequestResponseInteraction {
      description: "test2".to_string(),
      request: request.clone(),
      ..RequestResponseInteraction::default()
    };
    let interactions = vec![
      &interaction as &dyn Interaction,
      &interaction2 as &dyn Interaction
    ];
    let result = match_request(&request, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[test]
fn match_request_returns_a_match_for_multiple_requests() {
    let request = Request { method: "GET".to_string(), .. Request::default() };
    let request2 = Request { method: "POST".to_string(), path: "/post".to_string(), .. Request::default() };
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interaction2 = RequestResponseInteraction {
      description: "test2".to_string(),
      request: request2.clone(),
      ..RequestResponseInteraction::default()
    };
    let interactions = vec![
      &interaction  as &dyn Interaction,
      &interaction2 as &dyn Interaction
    ];
    let result = match_request(&request, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[test]
fn match_request_returns_a_mismatch_for_incorrect_request() {
    let request = Request::default();
    let expected_request = Request { query: Some(hashmap!{ "QueryA".to_string() => vec!["Value A".to_string()] }),
        .. Request::default() };
    let interaction = RequestResponseInteraction {
      request: expected_request,
      ..RequestResponseInteraction::default()
    };
    let interactions = vec![
      &interaction as &dyn Interaction
    ];
    let result = match_request(&request, interactions);
    expect!(result.match_key()).to(be_equal_to("Request-Mismatch".to_string()));
}

#[test]
fn match_request_returns_request_not_found_if_method_or_path_do_not_match() {
    let request = Request { method: "GET".to_string(), path: "/path".to_string(), .. Request::default() };
    let expected_request = Request { method: "POST".to_string(), path: "/otherpath".to_string(),
        .. Request::default() };
    let interaction = RequestResponseInteraction {
      request: expected_request,
      ..RequestResponseInteraction::default()
    };
    let interactions = vec![
      &interaction as &dyn Interaction
    ];
    let result = match_request(&request, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_the_most_appropriate_mismatch_for_multiple_requests() {
    let request = Request { method: "GET".to_string(), path: "/".to_string(), body: OptionalBody::Present("This is a body".into(), None),
      .. Request::default() };
    let request2 = Request { method: "GET".to_string(), path: "/".to_string(), query: Some(hashmap!{
        "QueryA".to_string() => vec!["Value A".to_string()]
        }), body: OptionalBody::Present("This is a body".into(), None),
      .. Request::default() };
    let request3 = Request { method: "GET".to_string(), path: "/".to_string(), query: Some(hashmap!{
        "QueryA".to_string() => vec!["Value A".to_string()]
        }), body: OptionalBody::Missing, .. Request::default() };
    let interaction = RequestResponseInteraction { description: "test".to_string(), request: request.clone(), .. RequestResponseInteraction::default() };
    let interaction2 = RequestResponseInteraction { description: "test2".to_string(), request: request2.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![&interaction as &dyn Interaction, &interaction2 as &dyn Interaction];
    let result = match_request(&request3, interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMismatch(interaction2.request,
        vec![Mismatch::BodyMismatch { path: "/".to_string(), expected: Some("This is a body".into()), actual: None,
        mismatch: "Expected body \'This is a body\' but was missing".to_string() }])));
}

#[test]
fn match_request_supports_v2_matchers() {
    let request = Request { method: "GET".to_string(), path: "/".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }), body: OptionalBody::Present(
            r#"
            {
                "a": 100,
                "b": "one hundred"
            }
            "#.into(), None
        ), .. Request::default() };
    let expected_request = Request { method: "GET".to_string(), path: "/".to_string(),
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] }),
        body: OptionalBody::Present(
            r#"
            {
                "a": 1000,
                "b": "One Thousand"
            }
            "#.into(), None
        ), matching_rules: matchingrules!{
          "body" => {
            "$.*" => [ MatchingRule::Type ]
          }
        },
      .. Request::default()
    };
    let interaction = RequestResponseInteraction { request: expected_request, .. RequestResponseInteraction::default() };
    let result = match_request(&request, vec![&interaction as &dyn Interaction]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[test]
fn match_request_supports_v2_matchers_with_xml() {
    let request = Request { method: "GET".to_string(), path: "/".to_string(), query: None,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/xml".to_string()] }), body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>world</foo>
            "#.into(), None
        ), .. Request::default() };
    let expected_request = Request { method: "GET".to_string(), path: "/".to_string(), query: None,
        headers: Some(hashmap!{ "Content-Type".to_string() => vec!["application/xml".to_string()] }),
        body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>mars </foo>
            "#.into(), None
        ), matching_rules: matchingrules!{
          "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex("[a-z]+".into()) ]
          }
        },
      .. Request::default()
    };
    let interaction = RequestResponseInteraction { request: expected_request, .. RequestResponseInteraction::default() };
    let result = match_request(&request, vec![&interaction as &dyn Interaction]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction.request, interaction.response)));
}

#[test]
fn match_request_with_header_with_multiple_values() {
  let pact = RequestResponsePact {
    interactions: vec![
      RequestResponseInteraction {
        request: Request {
          headers: Some(hashmap! {
            "accept".to_string() => vec!["application/hal+json".to_string(), "application/json".to_string()]
          }),
          .. Request::default()
        },
        .. RequestResponseInteraction::default()
      }
    ],
    .. RequestResponsePact::default()
  };
  let mut manager = ServerManager::new();
  let id = "match_request_with_header_with_multiple_values".to_string();
  let port = manager.start_mock_server(id.clone(), pact.boxed(), 0, MockServerConfig::default()).unwrap();

  let client = reqwest::blocking::Client::new();
  let response = client.get(format!("http://127.0.0.1:{}", port).as_str())
    .header(ACCEPT, "application/hal+json, application/json").send();

  let mismatches = manager.find_mock_server_by_id(&id, &|ms| ms.mismatches());
  manager.shutdown_mock_server_by_port(port);

  expect!(mismatches).to(be_some().value(vec![]));
  expect!(response.unwrap().status()).to(be_equal_to(200));
}

#[test]
fn match_request_with_more_specific_request() {
  let request1 = Request { path: "/animals/available".into(), .. Request::default() };
  let request2 = Request { path: "/animals/available".into(), headers: Some(hashmap! {
      "Authorization".to_string() => vec!["Bearer token".to_string()]
    }),
    .. Request::default() };
  let interaction1 = RequestResponseInteraction {
    description: "test_more_general_request".into(),
    request: request1.clone(),
    response: Response { status: 401, .. Response::default() },
    .. RequestResponseInteraction::default()
  };
  let interaction2 = RequestResponseInteraction {
    description: "test_more_specific_request".into(),
    request: request2.clone(),
    response: Response { status: 200, .. Response::default() },
    .. RequestResponseInteraction::default()
  };

  let expected = interaction1.clone();
  let result1 = match_request(&request1.clone(), vec![
    &interaction1 as &dyn Interaction, &interaction2 as &dyn Interaction]);
  expect!(result1).to(be_equal_to(MatchResult::RequestMatch(expected.request, expected.response)));

  let expected = interaction2.clone();
  let result2 = match_request(&request2.clone(), vec![
    &interaction1 as &dyn Interaction, &interaction2 as &dyn Interaction]);
  expect!(result2).to(be_equal_to(MatchResult::RequestMatch(expected.request, expected.response)));
}
