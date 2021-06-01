use pact_mock_server_ffi::message_with_metadata;
use pact_mock_server_ffi::write_pact_file;
use pact_mock_server_ffi::write_message_pact_file;
use libc::c_char;
use pact_mock_server_ffi::message_reify;
use pact_mock_server_ffi::message_with_contents;
use pact_mock_server_ffi::message_given;
use pact_mock_server_ffi::new_message;
use pact_mock_server_ffi::message_expects_to_receive;
use pact_mock_server_ffi::new_message_pact;
use std::ffi::{CStr, CString};
use std::panic::catch_unwind;

use bytes::Bytes;
use expectest::prelude::*;
use maplit::*;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use pact_mock_server_ffi::{
  cleanup_mock_server,
  create_mock_server,
  handles::InteractionPart,
  mock_server_mismatches,
  new_interaction,
  new_pact,
  upon_receiving,
  with_request,
  with_body,
  with_header,
  with_multipart_file,
  with_query_parameter,
  response_status,
  create_mock_server_for_pact
};
use pact_models::bodies::OptionalBody;

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
fn post_to_mock_server_with_misatches() {
  let pact_json = include_str!("post-pact.json");
  let pact_json_c = CString::new(pact_json).expect("Could not construct C string from json");
  let address = CString::new("127.0.0.1:0").unwrap();
  let port = create_mock_server(pact_json_c.as_ptr(), address.as_ptr(), false);
  expect!(port).to(be_greater_than(0));

  let _result = catch_unwind(|| {
    let client = Client::default();
    client.post(format!("http://127.0.0.1:{}/path", port).as_str())
      .header(CONTENT_TYPE, "application/json")
      .body(r#"{"foo":"no-very-bar"}"#)
      .send()
  });

  let mismatches = unsafe {
    CStr::from_ptr(mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[{\"method\":\"POST\",\"mismatches\":[{\"actual\":\"\\\"no-very-bar\\\"\",\"expected\":\"\\\"bar\\\"\",\"mismatch\":\"Expected \'bar\' to be equal to \'no-very-bar\'\",\"path\":\"$.foo\",\"type\":\"BodyMismatch\"}],\"path\":\"/path\",\"type\":\"request-mismatch\"}]"));
}

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
fn create_header_with_multiple_values() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_header_with_multiple_values").unwrap();
  let interaction = new_interaction(pact_handle, description.as_ptr());
  let name = CString::new("accept").unwrap();
  let value_1 = CString::new("application/hal+json").unwrap();
  let value_2 = CString::new("application/json").unwrap();
  with_header(interaction.clone(), InteractionPart::Request, name.as_ptr(), 1, value_2.as_ptr());
  with_header(interaction.clone(), InteractionPart::Request, name.as_ptr(), 0, value_1.as_ptr());
  interaction.with_interaction(&|_, _, i| {
    expect!(i.request.headers.as_ref()).to(be_some().value(&hashmap!{
      "accept".to_string() => vec!["application/hal+json".to_string(), "application/json".to_string()]
    }));
  });
}

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
fn create_query_parameter_with_multiple_values() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_query_parameter_with_multiple_values").unwrap();
  let interaction = new_interaction(pact_handle, description.as_ptr());
  let name = CString::new("q").unwrap();
  let value_1 = CString::new("1").unwrap();
  let value_2 = CString::new("2").unwrap();
  let value_3 = CString::new("3").unwrap();
  with_query_parameter(interaction.clone(), name.as_ptr(), 2, value_3.as_ptr());
  with_query_parameter(interaction.clone(), name.as_ptr(), 0, value_1.as_ptr());
  with_query_parameter(interaction.clone(), name.as_ptr(), 1, value_2.as_ptr());
  interaction.with_interaction(&|_, _, i| {
    expect!(i.request.query.as_ref()).to(be_some().value(&hashmap!{
      "q".to_string() => vec!["1".to_string(), "2".to_string(), "3".to_string()]
    }));
  });
}

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine
fn create_multipart_file() {
  let consumer_name = CString::new("consumer").unwrap();
  let provider_name = CString::new("provider").unwrap();
  let pact_handle = new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("create_multipart_file").unwrap();
  let interaction = new_interaction(pact_handle, description.as_ptr());
  let content_type = CString::new("application/json").unwrap();
  let file = CString::new("tests/multipart-test-file.json").unwrap();
  let part_name = CString::new("file").unwrap();

  with_multipart_file(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), file.as_ptr(), part_name.as_ptr());

  interaction.with_interaction(&|_, _, i| {
    let boundary = match &i.request.headers {
      Some(hashmap) => {
        hashmap.get("Content-Type")
          .map(|vec| vec[0].as_str())
          // Sorry for awful mime parsing..
          .map(|content_type: &str| content_type.split("boundary=").collect::<Vec<_>>())
          .map(|split| split[1])
          .unwrap_or("")
      },
      None => ""
    };

    expect!(i.request.headers.as_ref()).to(be_some().value(&hashmap!{
      "Content-Type".to_string() => vec![format!("multipart/form-data; boundary={}", boundary)],
    }));

    let actual_req_body_str = match &i.request.body {
      OptionalBody::Present(body, _) => body.clone(),
      _ => Bytes::new(),
    };

    let expected_req_body = Bytes::from(format!(
      "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"multipart-test-file.json\"\r\nContent-Type: application/json\r\n\r\ntrue\r\n--{boundary}--\r\n",
      boundary = boundary
    ));

    expect!(actual_req_body_str).to(be_equal_to(expected_req_body));
  });
}

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
fn http_consumer_feature_test() {
  let consumer_name = CString::new("http-consumer").unwrap();
  let provider_name = CString::new("http-provider").unwrap();
  let pact_handle = new_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let description = CString::new("request_with_matchers").unwrap();
  let interaction = new_interaction(pact_handle.clone(), description.as_ptr());
  let special_header = CString::new("My-Special-Content-Type").unwrap();
  let content_type = CString::new("Content-Type").unwrap();
  let authorization = CString::new("Authorization").unwrap();
  let path_matcher = CString::new("{\"value\":\"/request/1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"\\/request\\/[0-9]+\"}").unwrap();
  let value_header_with_matcher = CString::new("{\"value\":\"application/json\",\"pact:matcher:type\":\"dummy\"}").unwrap();
  let auth_header_with_matcher = CString::new("{\"value\":\"Bearer 1234\",\"pact:matcher:type\":\"regex\", \"regex\":\"Bearer [0-9]+\"}").unwrap();
  let query_param_matcher = CString::new("{\"value\":\"bar\",\"pact:matcher:type\":\"regex\", \"regex\":\"(bar|baz|bat)\"}").unwrap();
  let request_body_with_matchers = CString::new("{\"id\": {\"value\":1,\"pact:matcher:type\":\"type\"}}").unwrap();
  let response_body_with_matchers = CString::new("{\"created\": {\"value\":\"maybe\",\"pact:matcher:type\":\"regex\", \"regex\":\"(yes|no|maybe)\"}}").unwrap();
  let address = CString::new("127.0.0.1:0").unwrap();
  let file_path = CString::new("/tmp/pact").unwrap();
  let description = CString::new("a request to test the FFI interface").unwrap();
  let method = CString::new("POST").unwrap();
  let query =  CString::new("foo").unwrap();
  let header = CString::new("application/json").unwrap();

  upon_receiving(interaction.clone(), description.as_ptr());
  with_request(interaction.clone(), method  .as_ptr(), path_matcher.as_ptr());
  with_header(interaction.clone(), InteractionPart::Request, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
  with_header(interaction.clone(), InteractionPart::Request, authorization.as_ptr(), 0, auth_header_with_matcher.as_ptr());
  with_query_parameter(interaction.clone(), query.as_ptr(), 0, query_param_matcher.as_ptr());
  with_body(interaction.clone(), InteractionPart::Request, header.as_ptr(), request_body_with_matchers.as_ptr());
  // will respond with...
  with_header(interaction.clone(), InteractionPart::Response, content_type.as_ptr(), 0, value_header_with_matcher.as_ptr());
  with_header(interaction.clone(), InteractionPart::Response, special_header.as_ptr(), 0, value_header_with_matcher.as_ptr());
  with_body(interaction.clone(), InteractionPart::Response, header.as_ptr(), response_body_with_matchers.as_ptr());
  response_status(interaction.clone(), 200);
  let port = create_mock_server_for_pact(pact_handle.clone(), address.as_ptr(), false);

  expect!(port).to(be_greater_than(0));

  // Mock server has started, we can't now modify the pact
  expect!(upon_receiving(interaction.clone(), description.as_ptr())).to(be_false());

  let _ = catch_unwind(|| {
    let client = Client::default();
    let result = client.post(format!("http://127.0.0.1:{}/request/9999?foo=baz", port).as_str())
      .header("Content-Type", "application/json")
      .header("Authorization", "Bearer 9999")
      .body(r#"{"id": 7}"#)
      .send();

    match result {
      Ok(res) => {
        expect!(res.status()).to(be_eq(200));
        expect!(res.headers().get("My-Special-Content-Type").unwrap()).to(be_eq("application/json"));
        let json: serde_json::Value = res.json().unwrap_or_default();
        expect!(json.get("created").unwrap().as_str().unwrap()).to(be_eq("maybe"));
      },
      Err(_) => {
        panic!("expected 200 response but request failed");
      }
    };
  });

  let mismatches = unsafe {
    CStr::from_ptr(mock_server_mismatches(port)).to_string_lossy().into_owned()
  };

  write_pact_file(port, file_path.as_ptr(), true);
  cleanup_mock_server(port);

  expect!(mismatches).to(be_equal_to("[]"));
}

#[test]
#[cfg(not(target_env = "musl"))] // fails on alpine with SIGSEGV
fn message_consumer_feature_test() {
  let consumer_name = CString::new("message-consumer").unwrap();
  let provider_name = CString::new("message-provider").unwrap();
  let description = CString::new("message_request_with_matchers").unwrap();
  let content_type = CString::new("application/json").unwrap();
  let metadata_key = CString::new("message-queue-name").unwrap();
  let metadata_val = CString::new("message-queue-val").unwrap();
  let request_body_with_matchers = CString::new("{\"id\": {\"value\":1,\"pact:matcher:type\":\"type\"}}").unwrap();
  let file_path = CString::new("/tmp/pact").unwrap();
  let given = CString::new("a functioning FFI interface").unwrap();
  let receive_description = CString::new("a request to test the FFI interface").unwrap();

  let message_pact_handle = new_message_pact(consumer_name.as_ptr(), provider_name.as_ptr());
  let message_handle = new_message(message_pact_handle.clone(), description.as_ptr());
  message_given(message_handle.clone(), given.as_ptr());
  message_expects_to_receive(message_handle.clone(), receive_description.as_ptr());
  message_with_contents(message_handle.clone(), content_type.as_ptr(), request_body_with_matchers.as_ptr(), request_body_with_matchers.as_bytes().len());
  message_with_metadata(message_handle.clone(), metadata_key.as_ptr(), metadata_val.as_ptr());
  let res: *const c_char = message_reify(message_handle.clone());
  let reified: &CStr = unsafe { CStr::from_ptr(res) };
  expect!(reified.to_str().to_owned()).to(be_ok().value("{\"contents\":{\"id\":1},\"description\":\"a request to test the FFI interface\",\"matchingRules\":{\"body\":{\"$.id\":{\"combine\":\"AND\",\"matchers\":[{\"match\":\"type\"}]}}},\"metadata\":{\"contentType\":\"application/json\",\"message-queue-name\":\"message-queue-val\"},\"providerStates\":[{\"name\":\"a functioning FFI interface\"}]}".to_string()));
  let res = write_message_pact_file(message_pact_handle.clone(), file_path.as_ptr(), true);
  expect!(res).to(be_eq(0));
}
