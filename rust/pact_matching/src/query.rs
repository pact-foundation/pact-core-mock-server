//! Module for functions dealing with matching query parameters

use std::collections::HashMap;

use itertools::Itertools;
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
      compare_query_parameter_values(&path, expected, actual, context)
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
      merge_result(compare_query_parameter_values(&path, expected, actual, context), mismatch)
    }
  }
}

fn compare_query_parameter_value(
  path: &DocPath,
  expected: &str,
  actual: &str,
  index: usize,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let index = index.to_string();
  let index_path = path.join(index.as_str());
  let matcher_result = if context.matcher_is_defined(&index_path) {
    matchers::match_values(&index_path, &context.select_best_matcher(&index_path),
      expected.to_string(), actual.to_string())
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false)
      .map_err(|error| vec![error.to_string()])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::QueryMismatch {
        parameter: path.first_field().unwrap_or_default().to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        mismatch: message.clone()
      }
    }).collect()
  })
}

fn compare_query_parameter_values(
  path: &DocPath,
  expected: &[String],
  actual: &[String],
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let empty = String::new();
  let result: Vec<Mismatch> = expected.iter()
    .pad_using(actual.len(), |_| &empty)
    .enumerate()
    .flat_map(|(index, val)| {
      if index < actual.len() {
        match compare_query_parameter_value(path, val, &actual[index], index, context) {
          Ok(_) => vec![],
          Err(errors) => errors
        }
      } else if context.matcher_is_defined(path) {
        vec![]
      } else {
        let key = path.first_field().unwrap_or_default().to_string();
        vec![ Mismatch::QueryMismatch {
          parameter: key.clone(),
          expected: format!("{:?}", expected),
          actual: format!("{:?}", actual),
          mismatch: format!("Expected query parameter '{}' value '{}' but was missing", key, val)
        } ]
      }
    })
    .collect();

  if result.is_empty() {
    Ok(())
  } else {
    Err(result)
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::matchingrules;

  use crate::{CoreMatchingContext, DiffConfig, MatchingRule};

  #[test]
  fn compare_values_with_type_matcher() {
    let expected = ["1".to_string(), "2".to_string(), "3".to_string(), "4".to_string()];
    let actual = ["1".to_string(), "3".to_string()];
    let rules = matchingrules! {
      "query" => { "id" => [ MatchingRule::MinType(2) ] }
    };
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &rules.rules_for_category("query").unwrap_or_default(),
      &hashmap!{}
    );

    expect!(super::match_query_values("id", &expected, &actual, &context))
      .to(be_ok());
  }
}
