#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::Request;
#[allow(unused_imports)]
use pact_matching::match_request_result;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use serde_json;

#[test]
fn different_param_order() {
    println!("FILE: tests/spec_testcases/v1/request/query/different param order.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Query strings are matched using basic string equality, these are not equal. (Not supported)",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "hippo=John&alligator=Mary",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_request_result(expected, actual).mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn different_param_values() {
    println!("FILE: tests/spec_testcases/v1/request/query/different param values.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Queries are not the same - hippo is Fred instead of John",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=Fred",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_request_result(expected, actual).mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn trailing_amperand() {
    println!("FILE: tests/spec_testcases/v1/request/query/trailing amperand.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Query strings are matched using basic string equality, these are not equal. (not supported)",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John&",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_request_result(expected, actual).mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v1/request/query/matches.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Queries are the same",
        "expected" : {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        },
        "actual": {
          "method": "GET",
          "path": "/path",
          "query": "alligator=Mary&hippo=John",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1);
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.body.str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_request_result(expected, actual).mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
