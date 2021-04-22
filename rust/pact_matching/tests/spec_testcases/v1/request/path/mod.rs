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
fn empty_path_found_when_forward_slash_expected() {
    println!("FILE: tests/spec_testcases/v1/request/path/empty path found when forward slash expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Empty path found when forward slash expected",
        "expected" : {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/empty path found when forward slash expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/empty path found when forward slash expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn unexpected_trailing_slash_in_path() {
    println!("FILE: tests/spec_testcases/v1/request/path/unexpected trailing slash in path.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Path has unexpected trailing slash, trailing slashes can matter",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something/",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/unexpected trailing slash in path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/unexpected trailing slash in path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn forward_slash_found_when_empty_path_expected() {
    println!("FILE: tests/spec_testcases/v1/request/path/forward slash found when empty path expected.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Foward slash found when empty path expected",
        "expected" : {
          "method": "POST",
          "path": "",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/forward slash found when empty path expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/forward slash found when empty path expected.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn incorrect_path() {
    println!("FILE: tests/spec_testcases/v1/request/path/incorrect path.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Paths do not match",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something/else",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/incorrect path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/incorrect path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn missing_trailing_slash_in_path() {
    println!("FILE: tests/spec_testcases/v1/request/path/missing trailing slash in path.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": false,
        "comment": "Path is missing trailing slash, trailing slashes can matter",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something/",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/missing trailing slash in path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/missing trailing slash in path.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}

#[test]
fn matches() {
    println!("FILE: tests/spec_testcases/v1/request/path/matches.json");
    #[allow(unused_mut)]
    let mut pact: serde_json::Value = serde_json::from_str(r#"
      {
        "match": true,
        "comment": "Paths match",
        "expected" : {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        },
        "actual": {
          "method": "POST",
          "path": "/path/to/something",
          "query": "",
          "headers": {}
      
        }
      }
    "#).unwrap();

    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("expected").unwrap()});
    let expected = http_interaction_from_json("tests/spec_testcases/v1/request/path/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("EXPECTED: {}", expected);
    println!("BODY: {}", expected.contents().str_value());
    let interaction_json = serde_json::json!({"type": "Synchronous/HTTP", "request": pact.get("actual").unwrap()});
    let actual = http_interaction_from_json("tests/spec_testcases/v1/request/path/matches.json", &interaction_json, &PactSpecification::V1).unwrap();
    println!("ACTUAL: {}", actual);
    println!("BODY: {}", actual.contents().str_value());
    let pact_match = pact.get("match").unwrap();
    let result = match_interaction_request(expected, actual, &PactSpecification::V1).unwrap().mismatches();
    println!("RESULT: {:?}", result);
    if pact_match.as_bool().unwrap() {
       expect!(result.iter()).to(be_empty());
    } else {
       expect!(result.iter()).to_not(be_empty());
    }
}
