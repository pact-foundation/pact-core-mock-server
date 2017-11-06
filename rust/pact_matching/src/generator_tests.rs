use super::*;
use expectest::prelude::*;
use models::Response;
use models::generators::*;
use std::str::FromStr;

#[test]
fn returns_original_response_if_there_are_no_generators() {
  let response = Response::default_response();
  expect!(generate_response(response.clone())).to(be_equal_to(response));
}

#[test]
fn applies_status_generator_for_status_to_the_copy_of_the_response() {
  let response = Response { status: 200, generators: generators! {
    "STATUS" => Generator::RandomInt(400, 499)
  }, .. Response::default_response() };
  expect!(generate_response(response).status).to(be_greater_or_equal_to(400));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_response() {
  let response = Response { headers: Some(hashmap!{
      s!("A") => s!("a"),
      s!("B") => s!("b")
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Response::default_response()
  };
  let headers = generate_response(response).headers.unwrap().clone();
  expect!(headers.get("A").unwrap()).to_not(be_equal_to("a"));
}
