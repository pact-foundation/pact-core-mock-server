use expectest::prelude::*;
use serde_json:: json;

use pact_models::PactSpecification;

use super::{verify_json, ResultLevel};

#[test]
fn empty_json() {
  let json = json!({});
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());
}

#[test]
fn empty_json_strict() {
  let json = json!({});
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(2));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Missing consumer".to_string(), "Missing provider".to_string()]));
}

#[test]
fn invalid_json() {
  let json = json!(["this is a pact file!"]);
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Must be an Object, got Array".to_string()]));
}

#[test]
fn with_extra_properties() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "someOther": "property"
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string(), "Unexpected attribute 'someOther'".to_string()]));
}

#[test]
fn with_extra_properties_strict() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "someOther": "property"
  });
  let results = verify_json(&json, &PactSpecification::V1, "", true);

  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to_not(be_empty());
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(have_count(1));

  let errors: Vec<String> = results.iter()
    .filter(|result| result.level == ResultLevel::ERROR)
    .map(|result| result.message.clone())
    .collect();
  expect!(errors).to(be_equal_to(
    vec!["Unexpected attribute 'someOther'".to_string()]));
}

#[test]
fn with_metadata() {
  let json = json!({
    "consumer": {
      "name": "test"
    },
    "provider": {
      "name": "test"
    },
    "interactions": [],
    "metadata": {
      "a": "b",
      "c": ["d"],
      "e": "f"
    }
  });
  let results = verify_json(&json, &PactSpecification::V1, "", false);

  expect!(results.iter()).to(have_count(1));
  expect!(results.iter().filter(|result| result.level == ResultLevel::ERROR)).to(be_empty());

  let messages: Vec<String> = results.iter()
    .map(|result| result.message.clone())
    .collect();
  expect!(messages).to(be_equal_to(
    vec!["Interactions is empty".to_string()]));
}
