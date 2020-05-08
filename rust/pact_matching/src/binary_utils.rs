use serde_json::Value;
use crate::{DiffConfig, Mismatch};
use crate::models::matchingrules::{MatchingRules, RuleLogic};
use crate::matchers::{Matches, select_best_matcher};
use itertools::Itertools;
use log::*;

pub fn match_content_type<S>(data: &[u8], expected_content_type: S) -> Result<(), String>
  where S: Into<String> {
  let result = tree_magic::from_u8(data);
  let expected = expected_content_type.into();
  let matches = result == expected;
  debug!("Matching binary contents by content type: expected '{}', detected '{}' -> {}",
         expected, result, matches);
  if matches {
    Ok(())
  } else {
    Err(result)
  }
}

pub fn convert_data(data: &Value) -> Vec<u8> {
  match data {
    &Value::String(ref s) => base64::decode(s.as_str()).unwrap_or_else(|_| s.clone().into_bytes()),
    _ => data.to_string().into_bytes()
  }
}

pub fn match_octet_stream(expected: &Vec<u8>, actual: &Vec<u8>, _config: DiffConfig,
                          mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  debug!("matching binary contents ({} bytes)", actual.len());
  let path = vec!["$".to_string()];
  if matchers.matcher_is_defined("body", &path) {
    let matching_rules = select_best_matcher("body", &path, &matchers);
    match matching_rules {
      None => mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        mismatch: format!("No matcher found for category 'body' and path '{}'", path.iter().join("."))}),
      Some(ref rulelist) => {
        let results = rulelist.rules.iter().map(|rule| expected.matches(actual, rule)).collect::<Vec<Result<(), String>>>();
        match rulelist.rule_logic {
          RuleLogic::And => for result in results {
            if let Err(err) = result {
              mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
                actual: Some(actual.clone()),
                mismatch: err })
            }
          },
          RuleLogic::Or => {
            if results.iter().all(|result| result.is_err()) {
              for result in results {
                if let Err(err) = result {
                  mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
                    actual: Some(actual.clone()),
                    mismatch: err })
                }
              }
            }
          }
        }
      }
    }
  } else if expected != actual {
    mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
      actual: Some(actual.clone()),
      mismatch: format!("Expected binary data of {} bytes but received {} bytes", expected.len(), actual.len()) });
  }
}
