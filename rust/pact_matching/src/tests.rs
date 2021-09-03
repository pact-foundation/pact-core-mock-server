use std::collections::HashMap;

use expectest::prelude::*;
use pact_plugin_driver::catalogue_manager::register_core_entries;

use pact_models::{matchingrules, matchingrules_list};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::TEXT;
use pact_models::HttpStatus;
use pact_models::request::Request;

use super::*;

#[test]
fn match_method_returns_nothing_if_the_method_matches() {
  expect!(match_method(&"GET".to_string(), &"GET".to_string())).to(be_ok());
}

#[test]
fn match_method_returns_a_mismatch_if_the_method_does_not_match() {
  expect!(match_method(&"GET".to_string(), &"POST".to_string())).to(
    be_err().value(Mismatch::MethodMismatch { expected: "GET".into(), actual: "POST".into() }));
}

#[test]
fn match_method_returns_nothing_if_the_method_matches_with_different_case() {
  expect!(match_method(&"POST".to_string(), &"post".to_string())).to(be_ok());
}

#[test]
fn match_status_returns_nothing_if_the_status_matches() {
  expect!(match_status(200, 200, &MatchingContext::default())).to(be_ok());
}

#[test]
fn match_status_returns_a_mismatch_if_the_status_does_not_match() {
  expect!(match_status(200, 300, &MatchingContext::default())).to(
    be_err().value(vec![Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }])
  );
}

#[test]
fn match_status_using_matchers() {
  let rules = matchingrules_list! {
    "status"; "" => [ MatchingRule::StatusCode(HttpStatus::Success) ]
  };
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules, &hashmap!{}
  );
  expect!(match_status(200, 204, &context)).to(be_ok());
  let result = match_status(200, 500, &context);
  expect!(result.clone()).to(be_err().value(vec![Mismatch::StatusMismatch {
    expected: 200,
    actual: 500,
    mismatch: "".into()
  }]));
  expect!(result.unwrap_err().first().unwrap().description()).to(
    be_equal_to("Expected status code 500 to be a Successful response (200–299)"));
}

#[test]
fn match_query_returns_nothing_if_there_are_no_query_strings() {
  let expected = None;
  let actual = None;
  let result = match_query(expected, actual, &MatchingContext::default());
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_applies_matching_rules_when_param_has_an_underscore() {
  let expected = hashmap! { s!("user_id") => vec![s!("1")] };
  let actual = hashmap! { s!("user_id") => vec![s!("2")] };
  let rules = matchingrules! {
    "query" => { "user_id" => [ MatchingRule::Regex(s!("^[0-9]+$")) ] }
  };
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &rules.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let result = match_query(Some(expected), Some(actual), &context);
  expect!(result.values().flatten()).to(be_empty());
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_no_expected_query_string() {
  let expected = None;
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("a"),
    expected: s!(""),
    actual: s!("[\"b\"]"),
    mismatch: s!("Unexpected query parameter 'a' received"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_no_actual_query_string() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let expected = Some(query_map);
  let actual = None;
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("a"),
    expected: s!("[\"b\"]"),
    actual: s!(""),
    mismatch: s!("Expected query parameter 'a' but was missing"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_actual_query_parameter_that_is_not_expected() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![s!("d")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("c"),
    expected: s!(""),
    actual: s!("[\"d\"]"),
    mismatch: s!("Unexpected query parameter 'c' received"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_expected_query_parameter_that_is_not_received() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![s!("d")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("c"),
    expected: s!("[\"d\"]"),
    actual: s!(""),
    mismatch: s!("Expected query parameter 'c' but was missing"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_there_is_an_empty_expected_query_parameter_and_a_non_empty_actual() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![s!("d")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("c"),
    expected: s!("[]"),
    actual: s!("[\"d\"]"),
    mismatch: s!("Expected an empty parameter list for 'c' but received [\"d\"]"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_the_query_values_have_different_lengths() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![s!("d"), s!("e")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  query_map.insert(s!("c"), vec![s!("d")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  assert_eq!(mismatches.len(), 2);
  assert_eq!(mismatches[0], Mismatch::QueryMismatch {
    parameter: s!("c"),
    expected: s!("[\"d\", \"e\"]"),
    actual: s!("[\"d\"]"),
    mismatch: s!("Expected query parameter 'c' with 2 value(s) but received 1 value(s)"),
  });
  assert_eq!(mismatches[1], Mismatch::QueryMismatch {
    parameter: s!("c"),
    expected: s!("[\"d\", \"e\"]"),
    actual: s!("[\"d\"]"),
    mismatch: s!("Expected query parameter 'c' value 'e' but was missing"),
  });
}

#[test]
fn match_query_returns_a_mismatch_if_the_values_are_not_the_same() {
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("c")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &MatchingContext::default());
  let mismatches: Vec<Mismatch> = result.values().flatten().cloned().collect();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches.first().unwrap(), &Mismatch::QueryMismatch {
    parameter: s!("a"),
    expected: s!("b"),
    actual: s!("c"),
    mismatch: s!("Expected 'b' but received 'c' for query parameter 'a'")
  });
}

#[tokio::test]
async fn body_does_not_match_if_different_content_types() {
  let expected = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/json")] }),
    body: OptionalBody::Present(Bytes::new(), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("text/plain")] }),
    body: OptionalBody::Missing,
    ..Request::default()
  };
  let result = match_body(&expected, &actual, &MatchingContext::default(),
                          &MatchingContext::default()).await;
  let mismatches = result.mismatches();
  expect!(mismatches.iter()).to_not(be_empty());
  expect!(mismatches[0].clone()).to(be_equal_to(Mismatch::BodyTypeMismatch {
    expected: s!("application/json"),
    actual: s!("text/plain"),
    mismatch: s!(""),
    expected_body: None,
    actual_body: None
  }));
}

#[tokio::test]
async fn body_matching_uses_any_matcher_for_content_type_header() {
  let expected = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/json")] }),
    body: OptionalBody::Present(Bytes::from("100"), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/hal+json")] }),
    body: OptionalBody::Present(Bytes::from("100"), None, None),
    ..Request::default()
  };
  let header_context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "header" => { "Content-Type" => [ MatchingRule::Regex("application/.*json".into()) ] }
    }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
  );
  let result = match_body(&expected, &actual, &MatchingContext::default(), &header_context).await;
  let mismatches = result.mismatches();
  expect!(mismatches.iter()).to(be_empty());
}

#[tokio::test]
async fn body_matches_if_expected_is_missing() {
  let expected = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/json")] }),
    body: OptionalBody::Missing,
    ..Request::default()
  };
  let actual = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/json")] }),
    body: OptionalBody::Present("{}".into(), None, None),
    ..Request::default()
  };
  let result = match_body(&expected, &actual, &MatchingContext::default(), &MatchingContext::default()).await;
  expect!(result.mismatches().iter()).to(be_empty());
}

#[tokio::test]
async fn body_matches_with_extended_mime_types() {
  let expected = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/thrift+json")] }),
    body: OptionalBody::Present(r#"{"test":true}"#.into(), None, None),
    ..Request::default()
  };
  let actual = Request {
    method: s!("GET"),
    path: s!("/"),
    query: None,
    headers: Some(hashmap! { s!("Content-Type") => vec![s!("application/thrift+json")] }),
    body: OptionalBody::Present(r#"{"test": true}"#.into(), None, None),
    ..Request::default()
  };
  register_core_entries(CONTENT_MATCHER_CATALOGUE_ENTRIES.as_ref());
  register_core_entries(MATCHER_CATALOGUE_ENTRIES.as_ref());
  let result = match_body(&expected, &actual, &MatchingContext::default(), &MatchingContext::default()).await;
  expect!(result.mismatches().iter()).to(be_empty());
}

#[test]
fn partial_equal_for_method_mismatch() {
  let mismatch = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") };
  let mismatch2 = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") };
  let mismatch3 = Mismatch::MethodMismatch { expected: s!("get"), actual: s!("put") };
  let mismatch4 = Mismatch::MethodMismatch { expected: s!("post"), actual: s!("post") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: "".into(), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: "".into(), path: s!("/") }));
}

#[test]
fn partial_equal_for_path_mismatch() {
  let mismatch = Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() };
  let mismatch2 = Mismatch::PathMismatch { expected: s!("get"), actual: s!("post"), mismatch: "".into() };
  let mismatch3 = Mismatch::PathMismatch { expected: s!("get"), actual: s!("put"), mismatch: "".into() };
  let mismatch4 = Mismatch::PathMismatch { expected: s!("post"), actual: s!("post"), mismatch: "".into() };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: "".into(), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: "".into(), path: s!("/") }));
}

#[test]
fn partial_equal_for_status_mismatch() {
  let mismatch = Mismatch::StatusMismatch { expected: 100, actual: 200, mismatch: "".into() };
  let mismatch2 = Mismatch::StatusMismatch { expected: 100, actual: 200, mismatch: "".into() };
  let mismatch3 = Mismatch::StatusMismatch { expected: 100, actual: 300, mismatch: "".into() };
  let mismatch4 = Mismatch::StatusMismatch { expected: 200, actual: 100, mismatch: "".into() };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!("200"), actual: s!("300"), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_body_type_mismatch() {
  let mismatch = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch2 = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch3 = Mismatch::BodyTypeMismatch { expected: s!("get"), actual: s!("put"), mismatch: s!(""), expected_body: None, actual_body: None };
  let mismatch4 = Mismatch::BodyTypeMismatch { expected: s!("post"), actual: s!("post"), mismatch: s!(""), expected_body: None, actual_body: None };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_query_mismatch() {
  let mismatch = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch2 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch3 = Mismatch::QueryMismatch { parameter: s!("key2"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch4 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v100"), actual: s!("v2"), mismatch: s!("") };
  let mismatch5 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v200"), mismatch: s!("") };
  let mismatch6 = Mismatch::QueryMismatch { parameter: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_header_mismatch() {
  let mismatch = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch2 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch3 = Mismatch::HeaderMismatch { key: s!("key2"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("") };
  let mismatch4 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v100"), actual: s!("v2"), mismatch: s!("") };
  let mismatch5 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v200"), mismatch: s!("") };
  let mismatch6 = Mismatch::HeaderMismatch { key: s!("key"), expected: s!("v1"), actual: s!("v2"), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyMismatch { expected: Some("get".into()), actual: Some("post".into()), mismatch: s!(""), path: s!("/") }));
}

#[test]
fn partial_equal_for_body_mismatch() {
  let mismatch = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch2 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch3 = Mismatch::BodyMismatch { path: s!("key2"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("") };
  let mismatch4 = Mismatch::BodyMismatch { path: s!("key"), expected: None, actual: Some("v2".into()), mismatch: s!("") };
  let mismatch5 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: None, mismatch: s!("") };
  let mismatch6 = Mismatch::BodyMismatch { path: s!("key"), expected: Some("v1".into()), actual: Some("v2".into()), mismatch: s!("did not match") };
  expect!(&mismatch).to(be_equal_to(&mismatch));
  expect!(&mismatch).to(be_equal_to(&mismatch2));
  expect!(&mismatch).to(be_equal_to(&mismatch6));
  expect!(&mismatch).to_not(be_equal_to(&mismatch3));
  expect!(&mismatch).to_not(be_equal_to(&mismatch4));
  expect!(&mismatch).to_not(be_equal_to(&mismatch5));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::MethodMismatch { expected: s!("get"), actual: s!("post") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::StatusMismatch { expected: 200, actual: 300, mismatch: "".into() }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::PathMismatch { expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::HeaderMismatch { key: s!(""), expected: s!(""), actual: s!(""), mismatch: s!("") }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::BodyTypeMismatch { expected: s!(""), actual: s!(""), mismatch: s!(""), expected_body: None, actual_body: None }));
  expect!(&mismatch).to_not(be_equal_to(&Mismatch::QueryMismatch { parameter: s!(""), expected: s!("get"), actual: s!("post"), mismatch: s!("") }));
}

#[test]
fn match_path_returns_nothing_if_the_path_matches() {
  let context = MatchingContext::default();
  let result = match_path(&"/path/one".to_string(), &"/path/one".to_string(), &context);
  expect!(result).to(be_ok());
}

#[test]
fn match_path_returns_a_mismatch_if_the_path_does_not_match() {
  let context = MatchingContext::default();
  let result = match_path(&"/path/one".to_string(), &"/path/two".to_string(), &context);
  expect!(result).to(be_err().value(vec![ Mismatch::PathMismatch {
    expected: s!("/path/one"),
    actual: s!("/path/two"),
    mismatch: s!(""),
  } ]));
}

#[test]
fn match_path_returns_nothing_if_the_path_matches_with_a_matcher() {
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "path" => { "" => [ MatchingRule::Regex(s!("/path/\\d+")) ] }
    }.rules_for_category("path").unwrap_or_default(), &hashmap!{}
  );
  let result = match_path(&"/path/1234".to_string(), &"/path/5678".to_string(), &context);
  expect!(result).to(be_ok());
}

#[test]
fn match_path_returns_a_mismatch_if_the_path_does_not_match_with_a_matcher() {
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
        "path" => { "" => [ MatchingRule::Regex(s!("/path/\\d+")) ] }
    }.rules_for_category("path").unwrap_or_default(), &hashmap!{}
  );
  let result = match_path(&"/path/1234".to_string(), &"/path/abc".to_string(), &context);
  expect!(result).to(be_err().value(vec![ Mismatch::PathMismatch {
    expected: s!("/path/1234"),
    actual: s!("/path/abc"),
    mismatch: s!(""),
  }]));
}

#[test]
fn match_query_returns_no_mismatch_if_the_values_are_not_the_same_but_match_by_a_matcher() {
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "query" => {
        "a" => [ MatchingRule::Regex(s!("\\w+")) ]
      }
    }.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("c")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &context);
  expect!(result.get("a".into()).unwrap().iter()).to(be_empty());
}

#[test]
fn match_query_returns_a_mismatch_if_the_values_do_not_match_by_a_matcher() {
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "query" => {
        "a" => [ MatchingRule::Regex(s!("\\d+")) ]
      }
    }.rules_for_category("query").unwrap_or_default(), &hashmap!{}
  );
  let mut query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let expected = Some(query_map);
  query_map = HashMap::new();
  query_map.insert(s!("a"), vec![s!("b")]);
  let actual = Some(query_map);
  let result = match_query(expected, actual, &context);
  expect!(result.iter()).to_not(be_empty());
  assert_eq!(result.get("a".into()).unwrap()[0], Mismatch::QueryMismatch {
    parameter: s!("a"),
    expected: s!("b"),
    actual: s!("b"),
    mismatch: s!(""),
  });
}

macro_rules! request {
  ($e:expr) => (Request { body: OptionalBody::Present($e.into(), None, None), .. Request::default() })
}

#[tokio::test]
async fn matching_text_body_be_true_when_bodies_are_equal() {
  let expected = request!("body value");
  let actual = request!("body value");
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual,
    &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys)).await;
  expect!(mismatches.mismatches().iter()).to(be_empty());
}

#[tokio::test]
async fn matching_text_body_be_false_when_bodies_are_not_equal() {
  let expected = request!("expected body value");
  let actual = request!("actual body value");
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual,
    &MatchingContext::with_config(DiffConfig::AllowUnexpectedKeys)).await.mismatches();
  expect!(mismatches.iter()).to_not(be_empty());
  assert_eq!(mismatches[0], Mismatch::BodyMismatch {
    path: s!("$"),
    expected: expected.body.value(),
    actual: actual.body.value(),
    mismatch: s!(""),
  });
}

#[tokio::test]
async fn matching_text_body_must_use_defined_matcher() {
  let expected = request!("expected body value");
  let actual = request!("actualbodyvalue");

  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex(s!("\\w+")) ]
      }
    }.rules_for_category("body").unwrap_or_default(), &hashmap!{}
  );
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual, &context).await;
  expect!(mismatches.mismatches().iter()).to(be_empty());

  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Regex(s!("\\d+")) ]
      }
    }.rules_for_category("body").unwrap_or_default(), &hashmap!{}
  );
  let mismatches = compare_bodies(&TEXT.clone(), &expected, &actual, &context).await;
  expect!(mismatches.mismatches().iter()).to_not(be_empty());
}

#[test]
fn values_matcher_defined() {
  let context = MatchingContext::new(
    DiffConfig::AllowUnexpectedKeys,
    &matchingrules! {
      "body" => {
        "$" => [ MatchingRule::Values ],
        "$.x" => [ MatchingRule::Type ],
        "$.y" => [ MatchingRule::Values ],
        "$.z" => [ MatchingRule::Type, MatchingRule::Values ],
        "$.x[*].y" => [ MatchingRule::Values ],
        "$.y[*].y" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{});

  expect!(context.values_matcher_defined(&["$"])).to(be_true());
  expect!(context.values_matcher_defined(&["$", "x"])).to(be_false());
  expect!(context.values_matcher_defined(&["$", "y"])).to(be_true());
  expect!(context.values_matcher_defined(&["$", "z"])).to(be_true());
  expect!(context.values_matcher_defined(&["$", "x", "0", "y"])).to(be_true());
  expect!(context.values_matcher_defined(&["$", "x", "0", "z"])).to(be_false());
  expect!(context.values_matcher_defined(&["$", "y", "0", "y"])).to(be_false());
}
