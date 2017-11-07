use super::*;
use expectest::prelude::*;
use models::{Request, Response};
use models::generators::*;
use std::str::FromStr;

#[test]
fn returns_original_response_if_there_are_no_generators() {
  let response = Response::default_response();
  expect!(generate_response(&response)).to(be_equal_to(response));
}

#[test]
fn applies_status_generator_for_status_to_the_copy_of_the_response() {
  let response = Response { status: 200, generators: generators! {
    "STATUS" => Generator::RandomInt(400, 499)
  }, .. Response::default_response() };
  expect!(generate_response(&response).status).to(be_greater_or_equal_to(400));
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
  let headers = generate_response(&response).headers.unwrap().clone();
  expect!(headers.get("A").unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn returns_original_request_if_there_are_no_generators() {
  let request = Request::default_request();
  expect!(generate_request(&request)).to(be_equal_to(request));
}

#[test]
fn applies_path_generator_for_the_path_to_the_copy_of_the_request() {
  let request = Request { path: s!("/path"), generators: generators! {
    "PATH" => Generator::RandomInt(1, 10)
  }, .. Request::default_request() };
  expect!(generate_request(&request).path).to_not(be_equal_to("/path"));
}

#[test]
fn applies_header_generator_for_headers_to_the_copy_of_the_request() {
  let request = Request { headers: Some(hashmap!{
      s!("A") => s!("a"),
      s!("B") => s!("b")
    }), generators: generators! {
      "HEADER" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default_request()
  };
  let headers = generate_request(&request).headers.unwrap().clone();
  expect!(headers.get("A").unwrap()).to_not(be_equal_to("a"));
}

#[test]
fn applies_query_generator_for_query_parameters_to_the_copy_of_the_request() {
  let request = Request { query: Some(hashmap!{
      s!("A") => vec![ s!("a") ],
      s!("B") => vec![ s!("b") ]
    }), generators: generators! {
      "QUERY" => {
        "A" => Generator::Uuid
      }
    }, .. Request::default_request()
  };
  let query = generate_request(&request).query.unwrap().clone();
  let query_val = &query.get("A").unwrap()[0];
  expect!(query_val).to_not(be_equal_to("a"));
}
