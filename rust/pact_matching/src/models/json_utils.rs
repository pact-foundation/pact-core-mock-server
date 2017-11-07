use serde_json::{self, Value, Map};
use std::str::FromStr;

pub trait JsonToNum<T> {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: T) -> T;
}

impl JsonToNum<i32> for i32 {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: i32) -> i32 {
    match map.get(field) {
      Some(val) => match val {
        &Value::Number(ref num) => match num.as_i64() {
          Some(num) => num as i32,
          None => default
        },
        _ => default
      },
      None => default
    }
  }
}

impl JsonToNum<u16> for u16 {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: u16) -> u16 {
    match map.get(field) {
      Some(val) => match val {
        &Value::Number(ref num) => match num.as_u64() {
          Some(num) => num as u16,
          None => default
        },
        _ => default
      },
      None => default
    }
  }
}

pub fn json_to_string(value: &Value) -> String {
  match value {
    &Value::String(ref s) => s.clone(),
    _ => value.to_string()
  }
}

pub fn json_to_num(value: Option<Value>) -> Option<usize> {
  if let Some(value) = value {
    match value {
      Value::Number(n) => if n.is_i64() && n.as_i64().unwrap() > 0 { Some(n.as_i64().unwrap() as usize) }
        else if n.is_f64() { Some(n.as_f64().unwrap() as usize) }
        else if n.is_u64() { Some(n.as_u64().unwrap() as usize) }
        else { None },
      Value::String(ref s) => usize::from_str(&s.clone()).ok(),
      _ => None
    }
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use expectest::prelude::*;

  #[test]
  fn json_to_int_test() {
    expect!(<i32>::json_to_number(&serde_json::Map::new(), "any", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "any", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": "5" }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<i32>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(5));
    expect!(<i32>::json_to_number(&json!({ "min": -5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(-5));
    expect!(<i32>::json_to_number(&json!({ "min": 5.0 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));

    expect!(<u16>::json_to_number(&serde_json::Map::new(), "any", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "any", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": "5" }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(5));
    expect!(<u16>::json_to_number(&json!({ "min": -5 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
    expect!(<u16>::json_to_number(&json!({ "min": 5.0 }).as_object().unwrap(), "min", 1)).to(be_equal_to(1));
  }

  #[test]
  fn json_to_string_test() {
    expect!(json_to_string(&Value::from_str("\"test string\"").unwrap())).to(be_equal_to(s!("test string")));
    expect!(json_to_string(&Value::from_str("null").unwrap())).to(be_equal_to(s!("null")));
    expect!(json_to_string(&Value::from_str("100").unwrap())).to(be_equal_to(s!("100")));
    expect!(json_to_string(&Value::from_str("100.10").unwrap())).to(be_equal_to(s!("100.1")));
    expect!(json_to_string(&Value::from_str("{}").unwrap())).to(be_equal_to(s!("{}")));
    expect!(json_to_string(&Value::from_str("[]").unwrap())).to(be_equal_to(s!("[]")));
    expect!(json_to_string(&Value::from_str("true").unwrap())).to(be_equal_to(s!("true")));
    expect!(json_to_string(&Value::from_str("false").unwrap())).to(be_equal_to(s!("false")));
  }

  #[test]
  fn json_to_num_test() {
    expect!(json_to_num(Value::from_str("\"test string\"").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("null").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("{}").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("[]").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("true").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("false").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("100").ok())).to(be_some().value(100));
    expect!(json_to_num(Value::from_str("-100").ok())).to(be_none());
    expect!(json_to_num(Value::from_str("100.10").ok())).to(be_some().value(100));
  }
}
