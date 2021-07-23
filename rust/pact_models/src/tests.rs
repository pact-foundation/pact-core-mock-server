use expectest::prelude::*;
use serde_json::json;

use crate::{generators, HttpStatus, matchingrules, PactSpecification};
use crate::generators::{Generator, Generators, generators_from_json};
use crate::matchingrules::{matchers_from_json, MatchingRule};

#[test]
fn http_status_code_from_json() {
  expect!(HttpStatus::from_json(&json!({}))).to(be_err());
  expect!(HttpStatus::from_json(&json!("success"))).to(be_ok().value(HttpStatus::Success));
  expect!(HttpStatus::from_json(&json!("info"))).to(be_ok().value(HttpStatus::Information));
  expect!(HttpStatus::from_json(&json!("redirect"))).to(be_ok().value(HttpStatus::Redirect));
  expect!(HttpStatus::from_json(&json!("clientError"))).to(be_ok().value(HttpStatus::ClientError));
  expect!(HttpStatus::from_json(&json!("serverError"))).to(be_ok().value(HttpStatus::ServerError));
  expect!(HttpStatus::from_json(&json!("nonError"))).to(be_ok().value(HttpStatus::NonError));
  expect!(HttpStatus::from_json(&json!([200, 201, 204]))).to(be_ok().value(HttpStatus::StatusCodes(vec![200, 201, 204])));
}

#[test]
fn pact_spec_from_string() {
  expect!(PactSpecification::from("")).to(be_equal_to(PactSpecification::Unknown));
  expect!(PactSpecification::from("V1")).to(be_equal_to(PactSpecification::V1));
  expect!(PactSpecification::from("V1.1")).to(be_equal_to(PactSpecification::V1_1));
  expect!(PactSpecification::from("V2")).to(be_equal_to(PactSpecification::V2));
  expect!(PactSpecification::from("V3")).to(be_equal_to(PactSpecification::V3));
  expect!(PactSpecification::from("V4")).to(be_equal_to(PactSpecification::V4));
  expect!(PactSpecification::from("v2")).to(be_equal_to(PactSpecification::V2));
  expect!(PactSpecification::from("xxaasa")).to(be_equal_to(PactSpecification::Unknown));

  expect!(PactSpecification::from("V3".to_string())).to(be_equal_to(PactSpecification::V3));

  let _: PactSpecification = "v1".into();
}

#[test]
fn matchers_from_json_handles_missing_matchers() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_empty_matchers() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {}
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers.rules.iter()).to(be_empty());
}

#[test]
fn matchers_from_json_handles_matcher_with_no_matching_rules() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_correctly() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "matchingRules": {
            "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
            }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex("\\d+".to_string()) ]
        }
    }));
}

#[test]
fn matchers_from_json_loads_matchers_from_deprecated_name() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "deprecatedName": {
              "body": {
                "$.*.path": {
                    "matchers": [{
                        "match": "regex",
                        "regex": "\\d+"
                    }]
                }
              }
          }
      }
     "#).unwrap();
  let matchers = matchers_from_json(&json, &Some("deprecatedName".to_string()));
  let matchers = matchers.unwrap();

  expect!(matchers).to(be_equal_to(matchingrules!{
        "body" => {
            "$.*.path" => [ MatchingRule::Regex(r#"\d+"#.to_string()) ]
        }
    }));
}

#[test]
fn generators_from_json_handles_missing_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {}
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_empty_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {}
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators.categories.iter()).to(be_empty());
}

#[test]
fn generators_from_json_handles_generator_with_no_rules() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.*.path": {}
            }
          }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_ignores_invalid_generators() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
          "path": "/",
          "query": "",
          "headers": {},
          "generators": {
            "body": {
                "$.*.path": {
                  "type": "invalid"
                },
                "$.invalid": {
                  "type": 100
                },
                "$.other": null
            },
            "invalid": {
                "path": "path"
            },
            "more_invalid": 100
          }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(Generators::default()));
}

#[test]
fn generators_from_json_loads_generators_correctly() {
  let json : serde_json::Value = serde_json::from_str(r#"
      {
        "path": "/",
        "query": "",
        "headers": {},
        "generators": {
          "body": {
              "$.*.path": {
                  "type": "RandomInt",
                  "min": 1,
                  "max": 10
              }
          },
          "path": {
            "type": "RandomString"
          }
        }
      }
     "#).unwrap();
  let generators = generators_from_json(&json);
  let generators = generators.unwrap();

  expect!(generators).to(be_equal_to(generators!{
        "BODY" => {
            "$.*.path" => Generator::RandomInt(1, 10)
        },
        "PATH" => { "" => Generator::RandomString(10) }
    }));
}
