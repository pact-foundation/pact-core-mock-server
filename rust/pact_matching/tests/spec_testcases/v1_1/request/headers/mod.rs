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
fn order_of_comma_separated_header_values_different() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/order of comma separated header values different.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Comma separated headers out of order, order can matter http://tools.ietf.org/html/rfc2616",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators, hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "hippos, alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
fn whitespace_after_comma_different() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/whitespace after comma different.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Whitespace between comma separated headers does not matter",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators,hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators, hippos"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
fn header_value_is_different_case() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/header value is different case.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Headers values are case sensitive",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "Alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
fn header_name_is_different_case() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/header name is different case.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Header name is case insensitive",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "ACCEPT": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
fn unexpected_header_found() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/unexpected header found.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Extra headers allowed",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
    println!("FILE: tests/spec_testcases/v1_1/request/headers/matches.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {
            "Content-Type": "hippos",
            "Accept": "alligators"
          }
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
fn empty_headers() {
    println!("FILE: tests/spec_testcases/v1_1/request/headers/empty headers.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Empty headers match",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "query": "",
          "headers": {}
        }
      }
    "#).unwrap();

    let expected = Request::from_json(&pact.get("expected").unwrap(), &PactSpecification::V1_1);
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.body.str_value());
    let actual = Request::from_json(&pact.get("actual").unwrap(), &PactSpecification::V1_1);
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
