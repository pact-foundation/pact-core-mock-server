//! Module for functions dealing with matching query parameters

use std::collections::HashMap;

use maplit::hashmap;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;
use tracing::debug;

use crate::{matchers, Matches, MatchingContext, merge_result, Mismatch};

/// Match the query parameters as Maps
pub(crate) fn match_query_maps(
  expected: HashMap<String, Vec<String>>,
  actual: HashMap<String, Vec<String>>,
  context: &dyn MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  let mut result: HashMap<String, Vec<Mismatch>> = hashmap!{};
  for (key, value) in &expected {
    match actual.get(key) {
      Some(actual_value) => {
        let matches = match_query_values(key, value, actual_value, context);
        let v = result.entry(key.clone()).or_default();
        v.extend(matches.err().unwrap_or_default());
      },
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: format!("{:?}", value),
        actual: "".to_string(),
        mismatch: format!("Expected query parameter '{}' but was missing", key)
      })
    }
  }
  for (key, value) in &actual {
    match expected.get(key) {
      Some(_) => (),
      None => result.entry(key.clone()).or_default().push(Mismatch::QueryMismatch {
        parameter: key.clone(),
        expected: "".to_string(),
        actual: format!("{:?}", value),
        mismatch: format!("Unexpected query parameter '{}' received", key)
      })
    }
  }
  result
}

fn match_query_values(
  key: &str,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::root().join(key);
  if context.matcher_is_defined(&path) {
    debug!("match_query_values: Matcher defined for query parameter '{}", key);
    merge_result(
      matchers::match_values(&path, &context.select_best_matcher(&path), expected, actual)
        .map_err(|err| err.iter().map(|msg| {
          Mismatch::QueryMismatch {
            parameter: key.to_string(),
            expected: format!("{:?}", expected),
            actual: format!("{:?}", actual),
            mismatch: msg.clone()
          }
        }).collect()),
      compare_query_parameter_values(key, expected, actual, context)
    )
  } else {
    if expected.is_empty() && !actual.is_empty() {
      Err(vec![ Mismatch::QueryMismatch {
        parameter: key.to_string(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
        mismatch: format!("Expected an empty parameter list for '{}' but received {:?}", key, actual)
      } ])
    } else {
      let mismatch = if expected.len() != actual.len() {
        Err(vec![ Mismatch::QueryMismatch {
          parameter: key.to_string(),
          expected: format!("{:?}", expected),
          actual: format!("{:?}", actual),
          mismatch: format!(
            "Expected query parameter '{}' with {} value(s) but received {} value(s)",
            key, expected.len(), actual.len())
        } ])
      } else {
        Ok(())
      };
      merge_result(compare_query_parameter_values(key, expected, actual, context), mismatch)
    }
  }
}

fn compare_query_parameter_value(
  key: &str,
  expected: &str,
  actual: &str,
  index: usize,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let index = index.to_string();
  let path = DocPath::root().join(key).join(index.as_str());
  let matcher_result = if context.matcher_is_defined(&path) {
    matchers::match_values(&path, &context.select_best_matcher(&path), expected.to_string(), actual.to_string())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false)
      .map_err(|error| vec![error.to_string()])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::QueryMismatch {
        parameter: key.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        mismatch: message.clone(),
      }
    }).collect()
  })
}

fn compare_query_parameter_values(
  key: &str,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let result: Vec<Mismatch> = expected.iter().enumerate().flat_map(|(index, val)| {
    if index < actual.len() {
      match compare_query_parameter_value(key, val, &actual[index], index, context) {
        Ok(_) => vec![],
        Err(errors) => errors
      }
    } else {
      vec![ Mismatch::QueryMismatch {
        parameter: key.to_string(),
        expected: format!("{:?}", expected),
        actual: format!("{:?}", actual),
        mismatch: format!("Expected query parameter '{}' value '{}' but was missing", key, val)
      } ]
    }
  }).collect();

  if result.is_empty() {
    Ok(())
  } else {
    Err(result)
  }
}
