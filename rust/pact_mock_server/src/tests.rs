use expectest::prelude::*;
use expectest::expect;
use maplit::*;
use super::*;
use crate::matching::{MatchResult, match_request};
use pact_matching::models::{Interaction, Request, OptionalBody, Response, RequestResponseInteraction};
use pact_matching::Mismatch;
use pact_matching::models::matchingrules::*;
use pact_matching::matchingrules;
use reqwest::header::ACCEPT;

#[test]
fn match_request_returns_a_match_for_identical_requests() {
    let request = Request::default();
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![interaction.clone()];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_not_found_for_no_interactions() {
    let request = Request::default();
    let interactions = vec![];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_a_match_for_multiple_identical_requests() {
    let request = Request::default();
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![interaction.clone(),
      RequestResponseInteraction { description: s!("test2"), request: request.clone(), .. RequestResponseInteraction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_match_for_multiple_requests() {
    let request = Request { method: s!("GET"), .. Request::default() };
    let request2 = Request { method: s!("POST"), path: s!("/post"), .. Request::default() };
    let interaction = RequestResponseInteraction { request: request.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![interaction.clone(),
      RequestResponseInteraction { description: s!("test2"), request: request2.clone(), .. RequestResponseInteraction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_mismatch_for_incorrect_request() {
    let request = Request::default();
    let expected_request = Request { query: Some(hashmap!{ s!("QueryA") => vec![s!("Value A")] }),
        .. Request::default() };
    let interactions = vec![RequestResponseInteraction { request: expected_request, .. RequestResponseInteraction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result.match_key()).to(be_equal_to(s!("Request-Mismatch")));
}

#[test]
fn match_request_returns_request_not_found_if_method_or_path_do_not_match() {
    let request = Request { method: s!("GET"), path: s!("/path"), .. Request::default() };
    let expected_request = Request { method: s!("POST"), path: s!("/otherpath"),
        .. Request::default() };
    let interactions = vec![RequestResponseInteraction { request: expected_request, .. RequestResponseInteraction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_the_most_appropriate_mismatch_for_multiple_requests() {
    let request = Request { method: s!("GET"), path: s!("/"), body: OptionalBody::Present("This is a body".into(), None),
      .. Request::default() };
    let request2 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Present("This is a body".into(), None),
      .. Request::default() };
    let request3 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Missing, .. Request::default() };
    let interaction = RequestResponseInteraction { description: s!("test"), request: request.clone(), .. RequestResponseInteraction::default() };
    let interaction2 = RequestResponseInteraction { description: s!("test2"), request: request2.clone(), .. RequestResponseInteraction::default() };
    let interactions = vec![interaction.clone(), interaction2.clone()];
    let result = match_request(&request3, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMismatch(interaction2,
        vec![Mismatch::BodyMismatch { path: s!("/"), expected: Some("This is a body".into()), actual: None,
        mismatch: s!("Expected body \'This is a body\' but was missing") }])));
}

#[test]
fn match_request_supports_v2_matchers() {
    let request = Request { method: s!("GET"), path: s!("/"),
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] }), body: OptionalBody::Present(
            r#"
            {
                "a": 100,
                "b": "one hundred"
            }
            "#.into(), None
        ), .. Request::default() };
    let expected_request = Request { method: s!("GET"), path: s!("/"),
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] }),
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
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_supports_v2_matchers_with_xml() {
    let request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/xml")] }), body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>world</foo>
            "#.into(), None
        ), .. Request::default() };
    let expected_request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/xml")] }),
        body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>mars </foo>
            "#.into(), None
        ), matching_rules: matchingrules!{
          "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex(s!("[a-z]+")) ]
          }
        },
      .. Request::default()
    };
    let interaction = RequestResponseInteraction { request: expected_request, .. RequestResponseInteraction::default() };
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
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
  let port = manager.start_mock_server(id.clone(), pact, 0).unwrap();

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
    description: s!("test_more_general_request"),
    request: request1.clone(),
    response: Response { status: 401, .. Response::default() },
    .. RequestResponseInteraction::default()
  };
  let interaction2 = RequestResponseInteraction {
    description: s!("test_more_specific_request"),
    request: request2.clone(),
    response: Response { status: 200, .. Response::default() },
    .. RequestResponseInteraction::default()
  };

  let result1 = match_request(&request1.clone(), &vec![interaction1.clone(), interaction2.clone()]);
  expect!(result1).to(be_equal_to(MatchResult::RequestMatch(interaction1.clone())));

  let result2 = match_request(&request2.clone(), &vec![interaction1.clone(), interaction2.clone()]);
  expect!(result2).to(be_equal_to(MatchResult::RequestMatch(interaction2.clone())));
}
