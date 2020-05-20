#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::Response;
#[allow(unused_imports)]
use pact_matching::match_response;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use serde_json;

#[test]
fn different_status() {
    println!("FILE: tests/spec_testcases/v1_1/response/status/different status.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
          "match": false,
          "comment": "Status is incorrect",
          "expected": {
              "status": 202
          },
          "actual": {
              "status": 400
          }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v1_1/response/status/matches.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
          "match": true,
          "comment": "Status matches",
          "expected": {
              "status": 202
          },
          "actual": {
              "status": 202
          }
      }
    "#).unwrap();

    let expected = Response::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Response::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_response(expected, actual);
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
