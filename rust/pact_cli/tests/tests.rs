use expectest::prelude::*;
use serde_json::Value;

use pact_cli::verification::verify_json;
use pact_models::PactSpecification;
use pact_models::verify_json::ResultLevel;

#[test]
fn valid_basic_pact() {
  let pact_file = include_str!("pact.json");
  let json: Value = serde_json::from_str(pact_file).unwrap();

  let results = verify_json(&json, &PactSpecification::V1, pact_file, true);

  expect!(results.iter()).to(be_empty());
}

#[test]
fn valid_pact_metadata() {
  let pact_file = include_str!("test_pact.json");
  let json: Value = serde_json::from_str(pact_file).unwrap();

  let results = verify_json(&json, &PactSpecification::V1, pact_file, true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}

#[test]
fn valid_v1_pact_metadata() {
  let pact_file = include_str!("v1-pact.json");
  let json: Value = serde_json::from_str(pact_file).unwrap();

  let results = verify_json(&json, &PactSpecification::V1, pact_file, true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}

#[test]
fn valid_v2_pact() {
  let pact_file = include_str!("v2-pact.json");
  let json: Value = serde_json::from_str(pact_file).unwrap();

  let results = verify_json(&json, &PactSpecification::Unknown, pact_file, true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}
