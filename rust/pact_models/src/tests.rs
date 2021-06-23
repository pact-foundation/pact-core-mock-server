use expectest::prelude::*;
use serde_json::json;

use crate::{HttpStatus, PactSpecification};

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
