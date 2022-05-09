use std::collections::HashMap;
use std::str::from_utf8;

use hex::FromHex;
use itertools::Itertools;
use serde_json::Value;
use tracing::{error, trace, warn};

use crate::PactSpecification;

/// Decodes a query string using a percent-encoding scheme
pub fn decode_query(query: &str) -> Result<String, String> {
  let mut chars = query.chars();
  let mut ch = chars.next();
  let mut buffer = vec![];

  while ch.is_some() {
    let c = ch.unwrap();
    trace!("ch = '{:?}'", ch);
    if c == '%' {
      let c1 = chars.next();
      let c2 = chars.next();
      match (c1, c2) {
        (Some(v1), Some(v2)) => {
          let mut s = String::new();
          s.push(v1);
          s.push(v2);
          let decoded: Result<Vec<u8>, _> = FromHex::from_hex(s.into_bytes());
          match decoded {
            Ok(n) => {
              trace!("decoded = '{:?}'", n);
              buffer.extend_from_slice(&n);
            },
            Err(err) => {
              error!("Failed to decode '%{}{}' to as HEX - {}", v1, v2, err);
              buffer.push('%' as u8);
              buffer.push(v1 as u8);
              buffer.push(v2 as u8);
            }
          }
        },
        (Some(v1), None) => {
          buffer.push('%' as u8);
          buffer.push(v1 as u8);
        },
        _ => buffer.push('%' as u8)
      }
    } else if c == '+' {
      buffer.push(' ' as u8);
    } else {
      buffer.push(c as u8);
    }

    ch = chars.next();
  }

  match from_utf8(&buffer) {
    Ok(s) => Ok(s.to_owned()),
    Err(err) => {
      error!("Failed to decode '{}' to UTF-8 - {}", query, err);
      Err(format!("Failed to decode '{}' to UTF-8 - {}", query, err))
    }
  }
}

/// Encodes the query string using a percent-encoding scheme
pub fn encode_query(query: &str) -> String {
  query.chars().map(|ch| {
    match ch {
      ' ' => "+".to_string(),
      '-' => ch.to_string(),
      'a'..='z' => ch.to_string(),
      'A'..='Z' => ch.to_string(),
      '0'..='9' => ch.to_string(),
      _ => ch.escape_unicode()
        .filter(|u| u.is_digit(16))
        .batching(|it| {
          match it.next() {
            None => None,
            Some(x) => Some((x, it.next().unwrap()))
          }
        })
        .map(|u| format!("%{}{}", u.0, u.1))
        .collect()
    }
  }).collect()
}

/// Parses a query string into an optional map. The query parameter name will be mapped to
/// a list of values. Where the query parameter is repeated, the order of the values will be
/// preserved.
pub fn parse_query_string(query: &str) -> Option<HashMap<String, Vec<String>>> {
  if !query.is_empty() {
    Some(query.split('&').map(|kv| {
      trace!("kv = '{}'", kv);
      if kv.is_empty() {
        vec![]
      } else if kv.contains('=') {
        kv.splitn(2, '=').collect::<Vec<&str>>()
      } else {
        vec![kv]
      }
    }).fold(HashMap::new(), |mut map, name_value| {
      trace!("name_value = '{:?}'", name_value);
      if !name_value.is_empty() {
        let name = decode_query(name_value[0])
          .unwrap_or_else(|_| name_value[0].to_owned());
        let value = if name_value.len() > 1 {
          decode_query(name_value[1]).unwrap_or_else(|_| name_value[1].to_owned())
        } else {
          String::default()
        };
        trace!("decoded: '{}' => '{}'", name, value);
        map.entry(name).or_insert_with(|| vec![]).push(value);
      }
      map
    }))
  } else {
    None
  }
}

/// Converts a query string map into a query string
pub fn build_query_string(query: HashMap<String, Vec<String>>) -> String {
  query.into_iter()
    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
    .flat_map(|kv| {
      kv.1.iter()
        .map(|v| format!("{}={}", kv.0, encode_query(v)))
        .collect_vec()
    })
    .join("&")
}

/// Parses a V2 query string from a JSON struct
pub fn query_from_json(query_json: &Value, spec_version: &PactSpecification) -> Option<HashMap<String, Vec<String>>> {
  match query_json {
    &Value::String(ref s) => parse_query_string(s),
    _ => {
      warn!("Only string versions of request query strings are supported with specification version {}, ignoring.",
        spec_version.to_string());
      None
    }
  }
}

/// Parses a V3 query string from a JSON struct
pub fn v3_query_from_json(query_json: &Value, spec_version: &PactSpecification) -> Option<HashMap<String, Vec<String>>> {
  match query_json {
    &Value::String(ref s) => parse_query_string(s),
    &Value::Object(ref map) => Some(map.iter().map(|(k, v)| {
      (k.clone(), match v {
        &Value::String(ref s) => vec![s.clone()],
        &Value::Array(ref array) => array.iter().map(|item| match item {
          &Value::String(ref s) => s.clone(),
          _ => v.to_string()
        }).collect(),
        _ => {
          warn!("Query paramter value '{}' is not valid, ignoring", v);
          vec![]
        }
      })
    }).collect()),
    _ => {
      warn!("Only string or map versions of request query strings are supported with specification version {}, ignoring.",
                spec_version.to_string());
      None
    }
  }
}

/// Converts a query string structure into a JSON struct
pub fn query_to_json(query: HashMap<String, Vec<String>>, spec_version: &PactSpecification) -> Value {
  match spec_version {
    &PactSpecification::V3 | &PactSpecification::V4 => Value::Object(query.iter().map(|(k, v)| {
      (k.clone(), Value::Array(v.iter().map(|q| Value::String(q.clone())).collect()))}
    ).collect()),
    _ => Value::String(build_query_string(query))
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use expectest::prelude::*;
  use maplit::hashmap;

  use crate::query_strings::parse_query_string;

  #[test]
  fn parse_query_string_test() {
    let query = "a=b&c=d".to_string();
    let expected = hashmap!{
    "a".to_string() => vec!["b".to_string()],
    "c".to_string() => vec!["d".to_string()]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }

  #[test]
  fn parse_query_string_handles_empty_string() {
    let query = "".to_string();
    let expected = None;
    let result = parse_query_string(&query);
    assert_eq!(result, expected);
  }

  #[test]
  fn parse_query_string_handles_missing_values() {
    let query = "a=&c=d".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec!["".to_string()]);
    expected.insert("c".to_string(), vec!["d".to_string()]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
  }

  #[test]
  fn parse_query_string_handles_equals_in_values() {
    let query = "a=b&c=d=e=f".to_string();
    let mut expected = HashMap::new();
    expected.insert("a".to_string(), vec!["b".to_string()]);
    expected.insert("c".to_string(), vec!["d=e=f".to_string()]);
    let result = parse_query_string(&query);
    assert_eq!(result, Some(expected));
  }

  #[test]
  fn parse_query_string_decodes_values() {
    let query = "a=a%20b%20c".to_string();
    let expected = hashmap! {
    "a".to_string() => vec!["a b c".to_string()]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }

  #[test]
  fn parse_query_string_decodes_non_ascii_values() {
    let query = "accountNumber=100&anotherValue=%E6%96%87%E4%BB%B6.txt".to_string();
    let expected = hashmap! {
    "accountNumber".to_string() => vec!["100".to_string()],
    "anotherValue".to_string() => vec!["文件.txt".to_string()]
  };
    let result = parse_query_string(&query);
    expect!(result).to(be_some().value(expected));
  }
}
