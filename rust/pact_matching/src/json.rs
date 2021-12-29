//! The `json` module provides functions to compare and display the differences between JSON bodies

use std::str::FromStr;

use ansi_term::Colour::*;
use anyhow::anyhow;
use difference::*;
use log::*;
use onig::Regex;
use semver::Version;
use serde_json::{json, Value};

use pact_models::http_parts::HttpPart;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;
use pact_models::time_utils::validate_datetime;

use crate::{DiffConfig, MatchingContext, merge_result};
use crate::binary_utils::{convert_data, match_content_type};
use crate::matchers::*;
use crate::matchingrules::{compare_lists_with_matchingrule, compare_maps_with_matchingrule};

use super::Mismatch;

fn type_of(json: &Value) -> String {
  match json {
    Value::Object(_) => "Map",
    Value::Array(_) => "List",
    Value::Null => "Null",
    Value::Bool(_) => "Boolean",
    Value::Number(_) => "Number",
    Value::String(_) => "String"
  }.to_string()
}

impl Matches<Value> for Value {
  fn matches_with(&self, actual: Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    self.matches_with(&actual, matcher, cascaded)
  }
}

impl Matches<&Value> for &Value {
  fn matches_with(&self, actual: &Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    (*self).matches_with(actual, matcher, cascaded)
  }
}

impl Matches<&Value> for Value {
  fn matches_with(&self, actual: &Value, matcher: &MatchingRule, cascaded: bool) -> anyhow::Result<()> {
    let result = match matcher {
      MatchingRule::Regex(regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            let actual_str = match actual {
              Value::String(ref s) => s.clone(),
              _ => actual.to_string()
            };
            if re.is_match(&actual_str) {
              Ok(())
            } else {
              Err(anyhow!("Expected '{}' to match '{}'", json_to_string(actual), regex))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Include(substr) => {
        let actual_str = match actual {
          Value::String(ref s) => s.clone(),
          _ => actual.to_string()
        };
        if actual_str.contains(substr) {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to include '{}'", json_to_string(actual), substr))
        }
      },
      MatchingRule::Type => {
        match (self, actual) {
          (&Value::Array(_), &Value::Array(_)) => Ok(()),
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected '{}' to be the same type as '{}'", json_to_string(self), json_to_string(actual))),
        }
      },
      MatchingRule::MinType(min) => {
        match (self, actual) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() < *min {
            Err(anyhow!("Expected '{}' to have at least {} item(s)", json_to_string(actual), min))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected '{}' to be the same type as '{}'", json_to_string(self), json_to_string(actual))),
        }
      },
      MatchingRule::MaxType(max) => {
        match (self, actual) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() > *max {
            Err(anyhow!("Expected '{}' to have at most {} item(s)", json_to_string(actual), max))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected '{}' to be the same type as '{}'", json_to_string(self), json_to_string(actual))),
        }
      },
      MatchingRule::MinMaxType(min, max) => {
        match (self, actual) {
          (&Value::Array(_), &Value::Array(ref actual_array)) => if !cascaded && actual_array.len() < *min {
            Err(anyhow!("Expected '{}' to have at least {} item(s)", json_to_string(actual), min))
          } else if !cascaded && actual_array.len() > *max {
            Err(anyhow!("Expected '{}' to have at most {} item(s)", json_to_string(actual), max))
          } else {
            Ok(())
          },
          (&Value::Bool(_), &Value::Bool(_)) => Ok(()),
          (&Value::Number(_), &Value::Number(_)) => Ok(()),
          (&Value::Null, &Value::Null) => Ok(()),
          (&Value::Object(_), &Value::Object(_)) => Ok(()),
          (&Value::String(_), &Value::String(_)) => Ok(()),
          (_, _) => Err(anyhow!("Expected '{}' to be the same type as '{}'", json_to_string(self), json_to_string(actual))),
        }
      },
      MatchingRule::Equality | MatchingRule::Values => {
        if self == actual {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to be equal to '{}'", json_to_string(self), json_to_string(actual)))
        }
      },
      MatchingRule::Null => match actual {
        Value::Null => Ok(()),
        _ => Err(anyhow!("Expected '{}' to be a null value", json_to_string(actual)))
      },
      MatchingRule::Integer => if actual.is_i64() || actual.is_u64() {
        Ok(())
      } else {
        Err(anyhow!("Expected '{}' to be an integer value", json_to_string(actual)))
      },
      MatchingRule::Decimal => if actual.is_f64() {
        Ok(())
      } else {
        Err(anyhow!("Expected '{}' to be a decimal value", json_to_string(actual)))
      },
      MatchingRule::Number => if actual.is_number() {
        Ok(())
      } else {
        Err(anyhow!("Expected '{}' to be a number", json_to_string(actual)))
      },
      MatchingRule::Date(ref s) => {
        validate_datetime(&json_to_string(actual), s)
          .map_err(|err| anyhow!("Expected '{}' to match a date format of '{}': {}", actual, s, err))
      },
      MatchingRule::Time(ref s) => {
        validate_datetime(&json_to_string(actual), s)
          .map_err(|err| anyhow!("Expected '{}' to match a time format of '{}': {}", actual, s, err))
      },
      MatchingRule::Timestamp(ref s) => {
        validate_datetime(&json_to_string(actual), s)
          .map_err(|err| anyhow!("Expected '{}' to match a timestamp format of '{}': {}", actual, s, err))
      },
      MatchingRule::ContentType(ref expected_content_type) => {
        match_content_type(&convert_data(actual), expected_content_type)
          .map_err(|err| anyhow!("Expected data to have a content type of '{}' but was {}", expected_content_type, err))
      }
      MatchingRule::Boolean => match actual {
        Value::Bool(_) => Ok(()),
        Value::String(val) => if val == "true" || val == "false" {
          Ok(())
        } else {
          Err(anyhow!("Expected '{}' to match a boolean", json_to_string(actual)))
        }
        _ => Err(anyhow!("Expected '{}' to match a boolean", json_to_string(actual)))
      }
      MatchingRule::NotEmpty => match actual {
        Value::Null => Err(anyhow!("Expected non-empty but got a NULL")),
        Value::String(s) => if s.is_empty() {
          Err(anyhow!("Expected an non-empty string"))
        } else {
          Ok(())
        }
        Value::Array(a) => if a.is_empty() {
          Err(anyhow!("Expected an non-empty array"))
        } else {
          Ok(())
        }
        Value::Object(o) => if o.is_empty() {
          Err(anyhow!("Expected an non-empty object"))
        } else {
          Ok(())
        }
        _ => Ok(())
      }
      MatchingRule::Semver => match actual {
        Value::String(s) => match Version::parse(s) {
          Ok(_) => Ok(()),
          Err(err) => Err(anyhow!("'{}' is not a valid semantic version - {}", s, err))
        }
        _ => Err(anyhow!("Expected something that matches a semantic version, but got '{}'", actual))
      }
      _ => Ok(())
    };
    debug!("JSON -> JSON: Comparing '{}' to '{}' using {:?} -> {:?}", self, actual, matcher, result);
    result
  }
}

/// Matches the expected JSON to the actual, and populates the mismatches vector with any differences
pub fn match_json(expected: &dyn HttpPart, actual: &dyn HttpPart, context: &dyn MatchingContext) -> Result<(), Vec<super::Mismatch>> {
  let expected_json = serde_json::from_slice(&*expected.body().value().unwrap_or_default());
  let actual_json = serde_json::from_slice(&*actual.body().value().unwrap_or_default());

  if expected_json.is_err() || actual_json.is_err() {
    let mut mismatches = vec![];
    if let Err(e) = expected_json {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".to_string(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the expected body: '{}'", e),
      });
    }
    if let Err(e) = actual_json {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".to_string(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the actual body: '{}'", e),
      });
    }
    Err(mismatches.clone())
  } else {
    compare_json(&DocPath::root(), &expected_json.unwrap(), &actual_json.unwrap(), context)
  }
}

fn walk_json(json: &Value, path: &mut dyn Iterator<Item=&str>) -> Option<Value> {
  match path.next() {
    Some(p) => match json {
      Value::Object(_) => json.get(p).cloned(),
      Value::Array(ref array) => match usize::from_str(p) {
        Ok(index) => array.get(index).cloned(),
        Err(_) => None
      },
      _ => None
    },
    None => None
  }
}

/// Returns a diff of the expected versus the actual JSON bodies, focusing on a particular path
pub fn display_diff(expected: &str, actual: &str, path: &str, indent: &str) -> String {
  let expected_body = if expected.is_empty() {
    Value::String("".into())
  } else {
    Value::from_str(expected).unwrap_or_default()
  };
  let actual_body = if actual.is_empty() {
    Value::String("".into())
  } else {
    Value::from_str(actual).unwrap_or_default()
  };
  let mut path = path.split('.').skip(1);
  let next = path.next();
  let expected_fragment = if next.is_none() {
    serde_json::to_string_pretty(&expected_body).unwrap_or_default()
  } else {
    match walk_json(&expected_body, &mut path.clone()) {
      Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
      None => s!("")
    }
  };
  let actual_fragment = if next.is_none() {
    serde_json::to_string_pretty(&actual_body).unwrap_or_default()
  } else {
    match walk_json(&actual_body, &mut path.clone()) {
      Some(json) => format!("{:?}", serde_json::to_string_pretty(&json)),
      None => s!("")
    }
  };
  let changeset = Changeset::new(&expected_fragment, &actual_fragment, "\n");
  let mut output = String::new();
  for change in changeset.diffs {
      match change {
          Difference::Same(ref x) => output.push_str(&format!("{}{}\n", indent, x)),
          Difference::Add(ref x) => output.push_str(&Green.paint(format!("{}+{}\n", indent, x)).to_string()),
          Difference::Rem(ref x) => output.push_str(&Red.paint(format!("{}-{}\n", indent, x)).to_string())
      }
  }
  output
}

/// Compares the actual JSON to the expected one
pub fn compare_json(path: &DocPath, expected: &Value, actual: &Value, context: &dyn MatchingContext) -> Result<(), Vec<Mismatch>> {
  debug!("compare: Comparing path {}", path.join("."));
  match (expected, actual) {
    (&Value::Object(ref emap), &Value::Object(ref amap)) => compare_maps(path, emap, amap, context),
    (&Value::Object(_), _) => {
      Err(vec![ Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(json_to_string(expected).into()),
        actual: Some(json_to_string(actual).into()),
        mismatch: format!("Type mismatch: Expected {} {} but received {} {}",
                          type_of(expected), expected, type_of(actual), actual),
      } ])
    }
    (&Value::Array(ref elist), &Value::Array(ref alist)) => compare_lists(path, elist, alist, context),
    (&Value::Array(_), _) => {
      Err(vec![ Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(json_to_string(expected).into()),
        actual: Some(json_to_string(actual).into()),
        mismatch: format!("Type mismatch: Expected {} {} but received {} {}",
                          type_of(expected), json_to_string(expected), type_of(actual), json_to_string(actual)),
      } ])
    }
    (_, _) => compare_values(path, expected, actual, context)
  }
}

fn compare_maps(
  path: &DocPath,
  expected: &serde_json::Map<String, Value>,
  actual: &serde_json::Map<String, Value>,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let spath = path.to_string();
  debug!("compare_maps: Comparing maps at {}: {:?} -> {:?}", spath, expected, actual);
  if expected.is_empty() && context.config() == DiffConfig::NoUnexpectedKeys && !actual.is_empty() {
    debug!("compare_maps: Expected map is empty, but actual is not");
    Err(vec![ Mismatch::BodyMismatch {
      path: spath,
      expected: Some(json_to_string(&json!(expected)).into()),
      actual: Some(json_to_string(&json!(actual)).into()),
      mismatch: format!("Expected an empty Map but received {}", json_to_string(&json!(actual))),
    } ])
  } else {
    let mut result = Ok(());
    let expected = expected.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let actual = actual.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    if context.matcher_is_defined(path) {
      debug!("compare_maps: Matcher is defined for path {}", path);
      for matcher in context.select_best_matcher(path).rules {
        result = merge_result(result,compare_maps_with_matchingrule(&matcher, path, &expected, &actual, context, &mut |p, expected, actual| {
          compare_json(p, expected, actual, context)
        }));
      }
    } else {
      let expected_keys = expected.keys().cloned().collect();
      let actual_keys = actual.keys().cloned().collect();
      result = merge_result(result, context.match_keys(path, &expected_keys, &actual_keys));
      for (key, value) in expected.iter() {
        let p = path.join(key);
        if actual.contains_key(key) {
          result = merge_result(result, compare_json(&p, value, &actual[key], context));
        }
      }
    };
    result
  }
}

fn compare_lists(
  path: &DocPath,
  expected: &[Value],
  actual: &[Value],
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let spath = path.to_string();
  if context.matcher_is_defined(path) {
    debug!("compare_lists: matcher defined for path '{}'", path);
    let mut result = Ok(());
    for matcher in context.select_best_matcher(path).rules {
      let values_result = compare_lists_with_matchingrule(&matcher, path, expected, actual, context, &|p, expected, actual, context| {
        compare_json(p, expected, actual, context)
      });
      result = merge_result(result, values_result);
    }
    result
  } else if expected.is_empty() && !actual.is_empty() {
    Err(vec![ Mismatch::BodyMismatch {
      path: spath,
      expected: Some(json_to_string(&json!(expected)).into()),
      actual: Some(json_to_string(&json!(actual)).into()),
      mismatch: format!("Expected an empty List but received {}", json_to_string(&json!(actual))),
    } ])
  } else {
    let result = compare_list_content(path, expected, actual, context);
    if expected.len() != actual.len() {
      merge_result(result, Err(vec![ Mismatch::BodyMismatch {
        path: spath,
        expected: Some(json_to_string(&json!(expected)).into()),
        actual: Some(json_to_string(&json!(actual)).into()),
        mismatch: format!("Expected a List with {} elements but received {} elements",
                          expected.len(), actual.len()),
      } ]))
    } else {
      result
    }
  }
}

fn compare_list_content(
  path: &DocPath,
  expected: &[Value],
  actual: &[Value],
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let mut result = Ok(());
  for (index, value) in expected.iter().enumerate() {
    let ps = index.to_string();
    debug!("Comparing list item {} with value '{:?}' to '{:?}'", index, actual.get(index), value);
    let p = path.join(ps);
    if index < actual.len() {
      result = merge_result(result, compare_json(&p, value, &actual[index], context));
    } else if !context.matcher_is_defined(&p) {
      result = merge_result(result,Err(vec![ Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(json_to_string(&json!(expected)).into()),
        actual: Some(json_to_string(&json!(actual)).into()),
        mismatch: format!("Expected {} but was missing", json_to_string(value)) } ]))
    }
  }
  result
}

fn compare_values(
  path: &DocPath,
  expected: &Value,
  actual: &Value,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let matcher_result = if context.matcher_is_defined(path) {
    debug!("compare_values: Calling match_values for path {}", path);
    match_values(path, &context.select_best_matcher(&path), expected, actual)
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err| vec![err.to_string()])
  };
  log::debug!("compare_values: Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.to_string(), matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(format!("{}", expected).into()),
        actual: Some(format!("{}", actual).into()),
        mismatch: message.clone()
      }
    }).collect()
  })
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use expectest::expect;
  use expectest::prelude::*;
  use maplit::hashmap;

  use pact_models::{matchingrules, matchingrules_list};
  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};
  use pact_models::request::Request;

  use crate::{CoreMatchingContext, DiffConfig};
  use crate::Mismatch;
  use crate::Mismatch::BodyMismatch;

  use super::*;

  macro_rules! request {
    ($e:expr) => (Request { body: OptionalBody::Present($e.into(), None, None), .. Request::default() })
  }

  #[test]
  fn match_json_handles_invalid_expected_json() {
    let expected = request!(r#"{"json": "is bad"#);
    let actual = request!("{}");
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(vec![Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("") }]));
  }

  #[test]
  fn match_json_handles_invalid_actual_json() {
    let expected = request!("{}");
    let actual = request!(r#"{json: "is bad"}"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err().value(
      vec![
        Mismatch::BodyMismatch {
          path: s!("$"),
          expected: expected.body.value(),
          actual: actual.body.value(),
          mismatch: s!("Type mismatch: Expected List [{}] but received Map {}")
        }
      ]
    ));
  }

  fn mismatch_message(mismatch: &Result<(), Vec<Mismatch>>) -> String {
    match mismatch {
      Err(mismatches) => match &mismatches.first() {
        Some(Mismatch::BodyMismatch { mismatch, .. }) => mismatch.clone(),
        _ => "".into()
      },
      _ => "".into()
    }
  }

  #[test]
  fn match_json_handles_expecting_a_map_but_getting_a_list() {
    let expected = request!(r#"{}"#);
    let actual = request!(r#"[]"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Type mismatch: Expected Map {} but received List []")));
    expect!(result).to(be_err().value(vec![Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("")
    }]));
  }

  #[test]
  fn match_json_handles_expecting_a_list_but_getting_a_map() {
    let expected = request!(r#"[{}]"#);
    let actual = request!(r#"{}"#);
    let result = match_json(&expected.clone(), &actual.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Type mismatch: Expected List [{}] but received Map {}")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: expected.body.value(),
      actual: actual.body.value(),
      mismatch: s!("")
    }]));
  }

  #[test]
  fn match_json_handles_comparing_strings() {
    let val1 = request!(r#""string value""#);
    let val2 = request!(r#""other value""#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 'string value' to be equal to 'other value'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_integers() {
    let val1 = request!(r#"100"#);
    let val2 = request!(r#"200"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected '100' to be equal to '200'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_floats() {
    let val1 = request!(r#"100.01"#);
    let val2 = request!(r#"100.02"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected '100.01' to be equal to '100.02'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_booleans() {
    let val1 = request!(r#"true"#);
    let val2 = request!(r#"false"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected 'true' to be equal to 'false'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: val1.body.value(),
      actual: val2.body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_nulls() {
    let val1 = request!(r#"null"#);
    let val2 = request!(r#"33"#);
    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected '' to be equal to '33'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch {
      path: s!("$"),
      expected: val1.clone().body.value(),
      actual: val2.clone().body.value(),
      mismatch: s!("")
    } ]));
  }

  #[test]
  fn match_json_handles_comparing_lists() {
    let val1 = request!(r#"[]"#);
    let val2 = request!(r#"[11,22,33]"#);
    let val3 = request!(r#"[11,44,33]"#);
    let val4 = request!(r#"[11,44,33, 66]"#);

    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val3.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected an empty List but received [11,22,33]".to_string()));
    expect!(result).to(be_err());

    let result = match_json(&val2.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected '22' to be equal to '44'".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$[1]".to_string(),
        expected: Some("22".into()), actual: Some("44".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected a List with 3 elements but received 4 elements".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[11,44,33]".into()),
        actual: Some("[11,44,33,66]".into()), mismatch: "".to_string() } ]));

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$[1]".to_string(),
        expected: Some("22".into()),
        actual: Some("44".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$[1] -> Expected '22' to be equal to '44'".to_string()));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[11,22,33]".into()),
        actual: Some("[11,44,33,66]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Expected a List with 3 elements but received 4 elements".to_string()));

    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules! {
        "body" => {
            "$" => [ MatchingRule::Type ]
        }
    }.rules_for_category("body").unwrap(), &hashmap!{});
    let result = match_json(&val2.clone(), &val4.clone(), &context);
    expect!(result).to(be_ok());
    let result = match_json(&val4, &val2, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn match_json_handles_comparing_maps() {
    let val1 = request!(r#"{}"#);
    let val2 = request!(r#"{"a": 1, "b": 2}"#);
    let val3 = request!(r#"{"a": 1, "b": 3}"#);
    let val4 = request!(r#"{"a": 1, "b": 2, "c": 3}"#);

    let result = match_json(&val1.clone(), &val1.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val4.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val1.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected an empty Map but received {\"a\":1,\"b\":2}")));

    let result = match_json(&val2.clone(), &val3.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected '2' to be equal to '3'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.b"),
        expected: Some("2".into()), actual: Some("3".into()), mismatch: s!("") } ]));

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_ok());

    let result = match_json(&val2.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to("Expected a Map with keys a, b but received one with keys a, b, c".to_string()));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\"]".into()),
        actual: Some("[\"a\",\"b\",\"c\"]".into()), mismatch: "Expected a Map with keys a, b but received one with keys a, b, c".to_string()
    } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(mismatch_message(&result)).to(be_equal_to(s!("Expected '3' to be equal to '2'")));
    expect!(result).to(be_err().value(vec![ Mismatch::BodyMismatch { path: s!("$.b"),
        expected: Some("3".into()),
        actual: Some("2".into()), mismatch: s!("") } ]));

    let result = match_json(&val3.clone(), &val4.clone(), &CoreMatchingContext::with_config(DiffConfig::NoUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(2));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\"]".into()),
        actual: Some("[\"a\",\"b\",\"c\"]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Expected a Map with keys a, b but received one with keys a, b, c".to_string()));
    let mismatch = mismatches[1].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$.b".to_string(),
        expected: Some("3".into()),
        actual: Some("2".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$.b -> Expected '3' to be equal to '2'".to_string()));

    let result = match_json(&val4.clone(), &val2.clone(), &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter()).to(have_count(1));
    let mismatch = mismatches[0].clone();
    expect!(&mismatch).to(be_equal_to(&Mismatch::BodyMismatch { path: "$".to_string(),
        expected: Some("[\"a\",\"b\",\"c\"]".into()),
        actual: Some("[\"a\",\"b\"]".into()), mismatch: "".to_string()}));
    expect!(mismatch.description()).to(be_equal_to("$ -> Actual map is missing the following keys: c".to_string()));

    let result = match_json(&val3, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$.*" => [ MatchingRule::Type ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_ok());
  }

    #[test]
    fn equality_matcher_test() {
        let matcher = MatchingRule::Equality;
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_err());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_err());
    }

    #[test]
    fn regex_matcher_test() {
        let matcher = MatchingRule::Regex("^\\d+$".into());
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("10a".into()), &matcher, false)).to(be_err());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_ok());
    }

  #[test]
  fn includes_matcher_test() {
    let matcher = MatchingRule::Include("10".into());
    expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(Value::String("1a0".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_ok());
  }

    #[test]
    fn type_matcher_test() {
        let matcher = MatchingRule::Type;
        expect!(Value::String("100".into()).matches_with(Value::String("100".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("101".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(Value::String("10a".into()), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(json!(100), &matcher, false)).to(be_err());
    }

    #[test]
    fn min_type_matcher_test() {
        let matcher = MatchingRule::MinType(2);
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, false)).to(be_ok());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_err());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, true)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

    #[test]
    fn max_type_matcher_test() {
        let matcher = MatchingRule::MaxType(1);
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, false)).to(be_err());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]), &matcher, true)).to(be_ok());
        expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_ok());
        expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

    #[test]
    fn min_max_type_matcher_test() {
      let matcher = MatchingRule::MinMaxType(2, 3);
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100)]),
        &matcher, false)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100)]), &matcher, false)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100), json!(100)]), &matcher, false)).to(be_err());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100), json!(100),
        json!(100), json!(100)]), &matcher, true)).to(be_ok());
      expect!(Value::Array(vec![]).matches_with(&Value::Array(vec![json!(100)]), &matcher, false)).to(be_err());
      expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_ok());
    }

  #[test]
  fn integer_matcher_test() {
    let matcher = MatchingRule::Integer;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(&json!(100.02), &matcher, false)).to(be_err());
  }

  #[test]
  fn decimal_matcher_test() {
    let matcher = MatchingRule::Decimal;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.01), &matcher, false)).to(be_ok());
  }

  #[test]
  fn number_matcher_test() {
    let matcher = MatchingRule::Number;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_ok());
    expect!(Value::String("100".into()).matches_with(&json!(100.01), &matcher, false)).to(be_ok());
  }

  #[test]
  fn boolean_matcher_test() {
    let matcher = MatchingRule::Boolean;
    expect!(Value::Bool(true).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::Bool(true).matches_with(&Value::Bool(false), &matcher, false)).to(be_ok());
    expect!(Value::Bool(true).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::Bool(true).matches_with(&Value::String("true".into()), &matcher, false)).to(be_ok());
    expect!(Value::Bool(true).matches_with(&Value::String("false".into()), &matcher, false)).to(be_ok());
  }

  #[test]
  fn null_matcher_test() {
    let matcher = MatchingRule::Null;
    expect!(Value::String("100".into()).matches_with(&Value::String("100".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::String("101".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::String("10a".into()), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!(100.2), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&json!("null"), &matcher, false)).to(be_err());
    expect!(Value::String("100".into()).matches_with(&Value::Null, &matcher, false)).to(be_ok());
  }

  #[test_log::test]
  fn compare_maps_handles_wildcard_matchers() {
    let val1 = request!(r#"
    {
      "articles": [
        {
          "variants": {
            "001": {
              "bundles": {
                "001-A": {
                  "description": "someDescription",
                  "referencedArticles": [
                    {
                        "bundleId": "someId"
                    }
                  ]
                }
              }
            }
          }
        }
      ]
    }"#);
    let val2 = request!(r#"{
      "articles": [
        {
          "variants": {
            "002": {
              "bundles": {
                "002-A": {
                  "description": "someDescription",
                  "referencedArticles": [
                    {
                        "bundleId": "someId"
                    }
                  ]
                }
              }
            }
          }
        }
      ]
    }"#);

    let matching_rules = matchingrules! {
      "body" => {
        "$.articles[*].variants" => [ MatchingRule::Values ],
        "$.articles[*].variants.*.bundles" => [ MatchingRule::Values ],
        "$.articles[*].variants.*.bundles.*.referencedArticles[*]" => [ MatchingRule::Type ]
      }
    };
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                       &matching_rules.rules_for_category("body").unwrap(),
                                       &hashmap!{});
    let result = match_json(&val1, &val2, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_lists_with_array_contains_matcher() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 3, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$" => [ MatchingRule::ArrayContains(vec![]) ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_lists_without_array_contains_matcher_fails() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 3, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys));
    expect!(result).to(be_err());
  }

  #[test]
  fn compare_lists_with_array_contains_matcher_fails() {
    let val1 = request!(r#"
    [1, 2, 3]
    "#);
    let val2 = request!(r#"
    [10, 22, 6, 1, 5, 2]
    "#);

    let result = match_json(&val1, &val2, &CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules!{
      "body" => {
        "$" => [ MatchingRule::ArrayContains(vec![]) ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{}));
    expect!(result).to(be_err().value(vec![
      BodyMismatch {
        path: "$".to_string(),
        expected: Some("3".into()),
        actual: Some("[\"10\",\"22\",\"6\",\"1\",\"5\",\"2\"]".into()),
        mismatch: "Variant at index 2 (3) was not found in the actual list".to_string()
      }
    ]));
  }

  #[test]
  fn compare_lists_with_array_contains_matcher_with_more_complex_object() {
    let expected = request!(r#"
    {
      "class": [ "order" ],
      "properties": {
          "orderNumber": 42,
          "itemCount": 3,
          "status": "pending"
      },
      "entities": [
        {
          "class": [ "info", "customer" ],
          "properties": {
            "customerId": "pj123",
            "name": "Peter Joseph"
          }
        }
      ],
      "actions": [
        {
          "name": "add-item",
          "title": "Add Item",
          "method": "POST",
          "href": "http://api.x.io/orders/42/items"
        }
      ],
      "links": [
        { "rel": [ "next" ], "href": "http://api.x.io/orders/43" }
      ]
    }
    "#);
    let actual = request!(r#"
    {
      "class": [ "order" ],
      "properties": {
          "orderNumber": 12,
          "itemCount": 6,
          "status": "pending"
      },
      "entities": [
        {
          "class": [ "items", "collection" ],
          "rel": [ "http://x.io/rels/order-items" ],
          "href": "http://api.x.io/orders/12/items"
        },
        {
          "class": [ "info", "customer" ],
          "rel": [ "http://x.io/rels/customer" ],
          "properties": {
            "customerId": "rh565421",
            "name": "Ron Haich"
          },
          "links": [
            { "rel": [ "self" ], "href": "http://api.x.io/customers/rh565421" }
          ]
        }
      ],
      "actions": [
        {
          "name": "add-item",
          "title": "Add Item",
          "method": "POST",
          "href": "http://api.x.io/orders/12/items",
          "type": "application/x-www-form-urlencoded",
          "fields": [
            { "name": "orderNumber", "type": "hidden", "value": "12" },
            { "name": "productCode", "type": "text" },
            { "name": "quantity", "type": "number" }
          ]
        },
        {
          "name": "delete-order",
          "title": "Delete Order",
          "method": "DELETE",
          "href": "http://api.x.io/orders/12"
        },
        {
          "name": "update-order",
          "title": "Update Order",
          "method": "POST",
          "href": "http://api.x.io/orders/12"
        }
      ],
      "links": [
        { "rel": [ "self" ], "href": "http://api.x.io/orders/12" },
        { "rel": [ "previous" ], "href": "http://api.x.io/orders/11" },
        { "rel": [ "next" ], "href": "http://api.x.io/orders/13" }
      ]
    }
    "#);

    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys, &matchingrules! {
      "body" => {
        "$.entities" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.properties.customerId" => [ MatchingRule::Type ], "$.properties.name" => [ MatchingRule::Type ]
          }, HashMap::default())])
        ],
        "$.properties.orderNumber" => [ MatchingRule::Integer ],
        "$.properties.itemCount" => [ MatchingRule::Integer ],
        "$.actions" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.href" => [ MatchingRule::Regex(".*/orders/\\d+/items".to_string()) ]
          }, HashMap::default())])
        ],
        "$.links" => [
          MatchingRule::ArrayContains(vec![(0, matchingrules_list! {
            "body";
            "$.href" => [ MatchingRule::Regex(".*/orders/\\d+".to_string()) ]
          }, HashMap::default())])
        ]
      }
    }.rules_for_category("body").unwrap(), &hashmap!{});
    let result = match_json(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn compare_maps_handles_empty_expected_maps() {
    let expected_json = json!({});
    let expected = expected_json.as_object().unwrap();
    let actual_json = json!({"foo": "bar"});
    let actual = actual_json.as_object().unwrap();
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
                                       &MatchingRuleCategory::empty("body"), &hashmap!{});
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    expect!(result).to(be_ok());

    let context = CoreMatchingContext::new(DiffConfig::NoUnexpectedKeys,
                                       &MatchingRuleCategory::empty("body"), &hashmap!{});
    let result = compare_maps(&DocPath::root(), expected, actual, &context);
    expect!(result).to(be_err());
  }
}
