//! Collection of utilities for working with JSON

use std::hash::{Hash, Hasher};
use std::str::FromStr;

use serde_json::{self, Value, Map};

/// Trait to convert a JSON structure to a number
pub trait JsonToNum<T> {
  /// Converts the JSON field in the map to a Number
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: T) -> T;
}

impl JsonToNum<i32> for i32 {
  fn json_to_number(map: &serde_json::Map<String, Value>, field: &str, default: i32) -> i32 {
    match map.get(field) {
      Some(val) => match val {
        Value::Number(num) => match num.as_i64() {
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
        Value::Number(num) => match num.as_u64() {
          Some(num) => num as u16,
          None => default
        },
        _ => default
      },
      None => default
    }
  }
}

/// Converts the JSON struct to a String, first checking if it is a JSON String
pub fn json_to_string(value: &Value) -> String {
  match value {
    Value::String(s) => s.clone(),
    _ => value.to_string()
  }
}

/// Converts an optional JSON struct to a usize, returning `None` if it is not a numeric type.
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

/// Hash function for JSON struct
pub fn hash_json<H: Hasher>(v: &Value, state: &mut H) {
  match v {
    Value::Bool(b) => b.hash(state),
    Value::Number(n) => {
      if let Some(num) = n.as_u64() {
        num.hash(state);
      }
      if let Some(num) = n.as_f64() {
        num.to_string().hash(state);
      }
      if let Some(num) = n.as_i64() {
        num.hash(state);
      }
    }
    Value::String(s) => s.hash(state),
    Value::Array(values) => for value in values {
      hash_json(value, state);
    }
    Value::Object(map) => for (k, v) in map {
      k.hash(state);
      hash_json(v, state);
    }
    _ => ()
  }
}

/// Look up a field and return it as a string value
pub fn get_field_as_string(field: &str, map: &Map<String, Value>) -> Option<String> {
  map.get(field).map(|f| json_to_string(f))
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use serde_json::json;

  use super::*;

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
    expect!(json_to_string(&Value::from_str("\"test string\"").unwrap())).to(be_equal_to("test string".to_string()));
    expect!(json_to_string(&Value::from_str("null").unwrap())).to(be_equal_to("null".to_string()));
    expect!(json_to_string(&Value::from_str("100").unwrap())).to(be_equal_to("100".to_string()));
    expect!(json_to_string(&Value::from_str("100.10").unwrap())).to(be_equal_to("100.1".to_string()));
    expect!(json_to_string(&Value::from_str("{}").unwrap())).to(be_equal_to("{}".to_string()));
    expect!(json_to_string(&Value::from_str("[]").unwrap())).to(be_equal_to("[]".to_string()));
    expect!(json_to_string(&Value::from_str("true").unwrap())).to(be_equal_to("true".to_string()));
    expect!(json_to_string(&Value::from_str("false").unwrap())).to(be_equal_to("false".to_string()));
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
