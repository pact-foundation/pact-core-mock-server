#[allow(unused_imports)]
use test_env_log::test;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use serde_json;
#[allow(unused_imports)]
use expectest::prelude::*;
#[allow(unused_imports)]
use pact_matching::models::{Interaction, http_interaction_from_json};
#[allow(unused_imports)]
use pact_matching::{match_interaction_request, match_interaction_response};

#[test]
fn order_of_comma_separated_header_values_different() {
    println!("FILE: tests/spec_testcases/v2/request/headers/order of comma separated header values different.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/order of comma separated header values different.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/order of comma separated header values different.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn whitespace_after_comma_different() {
    println!("FILE: tests/spec_testcases/v2/request/headers/whitespace after comma different.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/whitespace after comma different.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/whitespace after comma different.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_value_is_different_case() {
    println!("FILE: tests/spec_testcases/v2/request/headers/header value is different case.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/header value is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/header value is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn header_name_is_different_case() {
    println!("FILE: tests/spec_testcases/v2/request/headers/header name is different case.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/header name is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/header name is different case.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches_with_regex() {
    println!("FILE: tests/spec_testcases/v2/request/headers/matches with regex.json");
    let pact : serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Headers match with regexp",
        "expected" : {
          "method": "POST",
          "path": "/path",
          "headers": {
            "Accept": "alligators",
            "Content-Type": "hippos"
          },
          "matchingRules": {
            "$.headers.Accept": {"match": "regex", "regex": "\\w+"}
          }
        },
        "actual": {
          "method": "POST",
          "path": "/path",
          "headers": {
            "Content-Type": "hippos",
            "Accept": "crocodiles"
          }
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/matches with regex.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/matches with regex.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_header_found() {
    println!("FILE: tests/spec_testcases/v2/request/headers/unexpected header found.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/unexpected header found.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/unexpected header found.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v2/request/headers/matches.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/matches.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn empty_headers() {
    println!("FILE: tests/spec_testcases/v2/request/headers/empty headers.json");
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

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v2/request/headers/empty headers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("EXPECTED: {:?}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v2/request/headers/empty headers.json", &interaction_json, &PactSpecification::V2).unwrap();
    println!("ACTUAL: {:?}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V2).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
