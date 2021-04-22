#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_models::PactSpecification;
#[allow(unused_imports)]
use serde_json;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use pact_matching::models::{Interaction, http_interaction_from_json};
#[allow(unused_imports)]
use pact_matching::{match_interaction_request, match_interaction_response};

#[test]
fn different_method() {
    println!("FILE: tests/spec_testcases/v4/request/method/different method.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Methods is incorrect",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {}
        },
        "actual": {
          "method": "GET",
          "path": "/",
          "query": {},
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/method/different method.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/method/different method.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V4).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn method_is_different_case() {
    println!("FILE: tests/spec_testcases/v4/request/method/method is different case.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Methods case does not matter",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {}
        },
        "actual": {
          "method": "post",
          "path": "/",
          "query": {},
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/method/method is different case.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/method/method is different case.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V4).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v4/request/method/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Methods match",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {}
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": {},
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v4/request/method/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v4/request/method/matches.json", &interaction_json, &PactSpecification::V4).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V4).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
