use std::ffi::{CStr, CString};
use std::panic::catch_unwind;

use bytes::Bytes;
use expectest::prelude::*;
use maplit::*;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;

use pact_matching::models::OptionalBody;
use pact_mock_server_ffi::{
  cleanup_mock_server,
  create_mock_server,
  handles::InteractionPart,
  mock_server_mismatches,
  new_interaction,
  new_pact,
  with_header,
  with_multipart_file,
  with_query_parameter
};

#[test]
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
  interaction.with_interaction(&|_, i| {
    expect!(i.request.headers.as_ref()).to(be_some().value(&hashmap!{
      "accept".to_string() => vec!["application/hal+json".to_string(), "application/json".to_string()]
    }));
  });
}

#[test]
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
  interaction.with_interaction(&|_, i| {
    expect!(i.request.query.as_ref()).to(be_some().value(&hashmap!{
      "q".to_string() => vec!["1".to_string(), "2".to_string(), "3".to_string()]
    }));
  });
}

#[test]
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

  interaction.with_interaction(&|_, i| {
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
