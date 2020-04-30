use pact_mock_server_ffi::*;
use std::ffi::{CString, CStr};
use expectest::prelude::*;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::panic::catch_unwind;

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
