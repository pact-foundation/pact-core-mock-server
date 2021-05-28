use expectest::prelude::*;
use serde_json::json;

use crate::HttpStatus;

#[test]
fn http_status_code_from_json() {
  expect!(HttpStatus::from_json(&json!({}))).to(be_ok().value(HttpStatus::Success));
  expect!(HttpStatus::from_json(&json!({ "status": "success" }))).to(be_ok().value(HttpStatus::Success));
  expect!(HttpStatus::from_json(&json!({ "status": "info" }))).to(be_ok().value(HttpStatus::Information));
  expect!(HttpStatus::from_json(&json!({ "status": "redirect" }))).to(be_ok().value(HttpStatus::Redirect));
  expect!(HttpStatus::from_json(&json!({ "status": "clientError" }))).to(be_ok().value(HttpStatus::ClientError));
  expect!(HttpStatus::from_json(&json!({ "status": "serverError" }))).to(be_ok().value(HttpStatus::ServerError));
  expect!(HttpStatus::from_json(&json!({ "status": "nonError" }))).to(be_ok().value(HttpStatus::NonError));
  expect!(HttpStatus::from_json(&json!({ "status": [200, 201, 204] }))).to(be_ok().value(HttpStatus::StatusCodes(vec![200, 201, 204])));
}
