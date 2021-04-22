use expectest::prelude::*;

use super::content_types::JSON;
use super::OptionalBody;

#[test]
fn display_tests() {
  expect!(format!("{}", OptionalBody::Missing)).to(be_equal_to("Missing"));
  expect!(format!("{}", OptionalBody::Empty)).to(be_equal_to("Empty"));
  expect!(format!("{}", OptionalBody::Null)).to(be_equal_to("Null"));
  expect!(format!("{}", OptionalBody::Present("hello".into(), None))).to(be_equal_to("Present(5 bytes)"));
  expect!(format!("{}", OptionalBody::Present("\"hello\"".into(), Some(JSON.clone())))).to(be_equal_to("Present(7 bytes, application/json)"));
}
