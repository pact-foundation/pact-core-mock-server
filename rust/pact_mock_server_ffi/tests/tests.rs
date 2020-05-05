use std::ffi::{CString, CStr};
use expectest::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::panic::catch_unwind;
use pact_mock_server_ffi::{
  create_mock_server,
  mock_server_mismatches,
  cleanup_mock_server,
  new_pact,
  new_interaction,
  with_header,
  handles::InteractionPart,
  with_query_parameter
};
use maplit::*;

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
