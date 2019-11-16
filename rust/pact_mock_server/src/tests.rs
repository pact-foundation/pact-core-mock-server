use expectest::prelude::*;
use super::*;
use crate::matching::{MatchResult, match_request};
use pact_matching::models::{Interaction, Request, OptionalBody};
use pact_matching::Mismatch;
use pact_matching::models::matchingrules::*;

#[test]
fn match_request_returns_a_match_for_identical_requests() {
    let request = Request::default();
    let interaction = Interaction { request: request.clone(), .. Interaction::default() };
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
    let interaction = Interaction { request: request.clone(), .. Interaction::default() };
    let interactions = vec![interaction.clone(),
      Interaction { description: s!("test2"), request: request.clone(), .. Interaction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_match_for_multiple_requests() {
    let request = Request { method: s!("GET"), .. Request::default() };
    let request2 = Request { method: s!("POST"), path: s!("/post"), .. Request::default() };
    let interaction = Interaction { request: request.clone(), .. Interaction::default() };
    let interactions = vec![interaction.clone(),
        Interaction { description: s!("test2"), request: request2.clone(), .. Interaction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_returns_a_mismatch_for_incorrect_request() {
    let request = Request::default();
    let expected_request = Request { query: Some(hashmap!{ s!("QueryA") => vec![s!("Value A")] }),
        .. Request::default() };
    let interactions = vec![Interaction { request: expected_request, .. Interaction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result.match_key()).to(be_equal_to(s!("Request-Mismatch")));
}

#[test]
fn match_request_returns_request_not_found_if_method_or_path_do_not_match() {
    let request = Request { method: s!("GET"), path: s!("/path"), .. Request::default() };
    let expected_request = Request { method: s!("POST"), path: s!("/otherpath"),
        .. Request::default() };
    let interactions = vec![Interaction { request: expected_request, .. Interaction::default() }];
    let result = match_request(&request, &interactions);
    expect!(result).to(be_equal_to(MatchResult::RequestNotFound(request)));
}

#[test]
fn match_request_returns_the_most_appropriate_mismatch_for_multiple_requests() {
    let request = Request { method: s!("GET"), path: s!("/"), body: OptionalBody::Present("This is a body".into()),
      .. Request::default() };
    let request2 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Present("This is a body".into()),
      .. Request::default() };
    let request3 = Request { method: s!("GET"), path: s!("/"), query: Some(hashmap!{
        s!("QueryA") => vec![s!("Value A")]
        }), body: OptionalBody::Missing, .. Request::default() };
    let interaction = Interaction { description: s!("test"), request: request.clone(), .. Interaction::default() };
    let interaction2 = Interaction { description: s!("test2"), request: request2.clone(), .. Interaction::default() };
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
            "#.into()
        ), .. Request::default() };
    let expected_request = Request { method: s!("GET"), path: s!("/"),
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/json")] }),
        body: OptionalBody::Present(
            r#"
            {
                "a": 1000,
                "b": "One Thousand"
            }
            "#.into()
        ), matching_rules: matchingrules!{
          "body" => {
            "$.*" => [ MatchingRule::Type ]
          }
        },
      .. Request::default()
    };
    let interaction = Interaction { request: expected_request, .. Interaction::default() };
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}

#[test]
fn match_request_supports_v2_matchers_with_xml() {
    let request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/xml")] }), body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>world</foo>
            "#.into()
        ), .. Request::default() };
    let expected_request = Request { method: s!("GET"), path: s!("/"), query: None,
        headers: Some(hashmap!{ s!("Content-Type") => vec![s!("application/xml")] }),
        body: OptionalBody::Present(
            r#"<?xml version="1.0" encoding="UTF-8"?>
            <foo>hello<bar/>mars </foo>
            "#.into()
        ), matching_rules: matchingrules!{
          "body" => {
            "$.foo['#text']" => [ MatchingRule::Regex(s!("[a-z]+")) ]
          }
        },
      .. Request::default()
    };
    let interaction = Interaction { request: expected_request, .. Interaction::default() };
    let result = match_request(&request, &vec![interaction.clone()]);
    expect!(result).to(be_equal_to(MatchResult::RequestMatch(interaction)));
}
