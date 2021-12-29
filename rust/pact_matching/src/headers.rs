//! Matching functions for headers

use std::collections::HashMap;
use std::iter::FromIterator;

use maplit::*;

use pact_models::headers::PARAMETERISED_HEADERS;
use pact_models::matchingrules::MatchingRule;
use pact_models::path_exp::DocPath;

use crate::{matchers, MatchingContext, Mismatch};
use crate::matchers::Matches;

fn strip_whitespace<'a, T: FromIterator<&'a str>>(val: &'a str, split_by: &'a str) -> T {
  val.split(split_by).map(|v| v.trim()).collect()
}

fn parse_charset_parameters(parameters: &[&str]) -> HashMap<String, String> {
  parameters.iter().map(|v| v.split('=').map(|p| p.trim()).collect::<Vec<&str>>())
    .fold(HashMap::new(), |mut map, name_value| {
      map.insert(name_value[0].to_string(), name_value[1].to_string());
      map
    })
}

pub(crate) fn match_parameter_header(expected: &str, actual: &str, header: &str, value_type: &str) -> Result<(), Vec<String>> {
  let expected_values: Vec<&str> = strip_whitespace(expected, ";");
  let actual_values: Vec<&str> = strip_whitespace(actual, ";");
  let expected_parameters = expected_values.as_slice().split_first().unwrap();
  let actual_parameters = actual_values.as_slice().split_first().unwrap();
  let header_mismatch = format!("Expected {} '{}' to have value '{}' but was '{}'", value_type, header, expected, actual);

  let mut mismatches = vec![];
  if expected_parameters.0 == actual_parameters.0 {
    let expected_parameter_map = parse_charset_parameters(expected_parameters.1);
    let actual_parameter_map = parse_charset_parameters(actual_parameters.1);
    for (k, v) in expected_parameter_map {
      if actual_parameter_map.contains_key(&k) {
        if v.to_ascii_lowercase() != actual_parameter_map.get(&k).unwrap().to_ascii_lowercase() {
          mismatches.push(header_mismatch.clone());
        }
      } else {
        mismatches.push(header_mismatch.clone());
      }
    }
  } else {
    mismatches.push(header_mismatch);
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches)
  }
}

pub(crate) fn match_header_value(
  key: &str,
  expected: &str,
  actual: &str,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::root().join(key);
  let expected: String = strip_whitespace(expected, ",");
  let actual: String = strip_whitespace(actual, ",");

  let matcher_result = if context.matcher_is_defined(&path) {
    matchers::match_values(&path, &context.select_best_matcher(&path), &expected, &actual)
  } else if PARAMETERISED_HEADERS.contains(&key.to_lowercase().as_str()) {
    match_parameter_header(expected.as_str(), actual.as_str(), key, "header")
  } else {
    Matches::matches_with(&expected, &actual, &MatchingRule::Equality, false)
      .map_err(|err| vec![err.to_string()])
  };
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::HeaderMismatch {
        key: key.to_string(),
        expected: expected.to_string(),
        actual: actual.to_string(),
        mismatch: format!("Mismatch with header '{}': {}", key, message)
      }
    }).collect()
  })
}

fn find_entry<T>(map: &HashMap<String, T>, key: &str) -> Option<(String, T)> where T: Clone {
  match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
    Some(k) => map.get(k).map(|v| (key.to_string(), v.clone()) ),
    None => None
  }
}

fn match_header_maps(
  expected: HashMap<String, Vec<String>>,
  actual: HashMap<String, Vec<String>>,
  context: &dyn MatchingContext
) -> HashMap<String, Vec<Mismatch>> {
  let mut result = hashmap!{};
  for (key, value) in &expected {
    match find_entry(&actual, key) {
      Some((_, actual_value)) => for (index, val) in value.iter().enumerate() {
        result.insert(key.clone(), match_header_value(key, val,
                                                      actual_value.get(index).unwrap_or(&String::default()), context).err().unwrap_or_default());
      },
      None => {
        result.insert(key.clone(), vec![Mismatch::HeaderMismatch { key: key.clone(),
          expected: format!("{:?}", value.join(", ")),
          actual: "".to_string(),
          mismatch: format!("Expected header '{}' but was missing", key) }]);
      }
    }
  }
  result
}

/// Matches the actual headers to the expected ones.
pub fn match_headers(
  expected: Option<HashMap<String, Vec<String>>>,
  actual: Option<HashMap<String, Vec<String>>>,
  context: &(dyn MatchingContext + Send + Sync)
) -> HashMap<String, Vec<Mismatch>> {
  match (actual, expected) {
    (Some(aqm), Some(eqm)) => match_header_maps(eqm, aqm, context),
    (Some(_), None) => hashmap!{},
    (None, Some(eqm)) => eqm.iter().map(|(key, value)| {
      (key.clone(), vec![Mismatch::HeaderMismatch { key: key.clone(),
        expected: format!("{:?}", value.join(", ")),
        actual: "".to_string(),
        mismatch: format!("Expected header '{}' but was missing", key) }])
    }).collect(),
    (None, None) => hashmap!{}
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::*;

  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;

  use crate::{CoreMatchingContext, DiffConfig, Mismatch};
  use crate::headers::{match_header_value, match_headers};

  #[test]
  fn matching_headers_be_true_when_headers_are_equal() {
    let mismatches = match_header_value("HEADER", "HEADER", "HEADER",
                                        &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn matching_headers_be_false_when_headers_are_not_equal() {
    let mismatches = match_header_value("HEADER", "HEADER", "HEADER2",
                                        &CoreMatchingContext::default()).unwrap_err();
    expect!(mismatches.iter()).to_not(be_empty());
    assert_eq!(mismatches[0], Mismatch::HeaderMismatch {
      key: s!("HEADER"),
      expected: s!("HEADER"),
      actual: s!("HEADER2"),
      mismatch: s!(""),
    });
  }

  #[test]
  fn mismatch_message_generated_when_headers_are_not_equal() {
    let mismatches = match_header_value("HEADER", "HEADER_VALUE", "HEADER2",
                                        &CoreMatchingContext::default());

    match mismatches.unwrap_err()[0] {
      Mismatch::HeaderMismatch { ref mismatch, .. } =>
        assert_eq!(mismatch, "Mismatch with header 'HEADER': Expected 'HEADER_VALUE' to be equal to 'HEADER2'"),
      _ => panic!("Unexpected mismatch response")
    }
  }

  #[test]
  fn matching_headers_exclude_whitespaces() {
    let mismatches = match_header_value("HEADER", "HEADER1, HEADER2,   3",
                                        "HEADER1,HEADER2,3", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn matching_headers_includes_whitespaces_within_a_value() {
    let mismatches = match_header_value("HEADER", "HEADER 1, \tHEADER 2,\n3",
                                        "HEADER 1,HEADER 2,3", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn content_type_header_matches_when_headers_are_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/json;charset=UTF-8",
                                        "application/json; charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn content_type_header_does_not_match_when_headers_are_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/pdf;charset=UTF-8",
                                        "application/json;charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_expected_is_empty() {
    let mismatches = match_header_value("CONTENT-TYPE", "",
                                        "application/json;charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_actual_is_empty() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/pdf;charset=UTF-8",
                                        "", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_not_match_when_charsets_are_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/json;charset=UTF-8",
                                        "application/json;charset=UTF-16", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_match_when_charsets_are_different_case() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/json;charset=UTF-8",
                                        "application/json;charset=utf-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn content_type_header_does_not_match_when_charsets_other_parameters_not_equal() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/json;declaration=\"<950118.AEB0@XIson.com>\"",
                                        "application/json;charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn content_type_header_does_match_when_charsets_is_missing_from_expected_header() {
    let mismatches = match_header_value("CONTENT-TYPE", "application/json",
                                        "application/json;charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn mismatched_header_description_reports_content_type_mismatches_correctly() {
    let mismatches = match_header_value("CONTENT-TYPE", "CONTENT-TYPE-VALUE", "HEADER2",
                                        &CoreMatchingContext::default());

    match mismatches.unwrap_err()[0] {
      Mismatch::HeaderMismatch { ref mismatch, .. } =>
        assert_eq!(mismatch, "Mismatch with header 'CONTENT-TYPE': Expected header 'CONTENT-TYPE' to have value 'CONTENT-TYPE-VALUE' but was 'HEADER2'"),
      _ => panic!("Unexpected mismatch response")
    }
  }

  #[test]
  fn accept_header_matches_when_headers_are_equal() {
    let mismatches = match_header_value("ACCEPT", "application/hal+json;charset=utf-8",
                                        "application/hal+json;charset=utf-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn accept_header_does_not_match_when_actual_is_empty() {
    let mismatches = match_header_value("ACCEPT", "application/hal+json",
                                        "", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn accept_header_does_match_when_charset_is_missing_from_expected_header() {
    let mismatches = match_header_value("ACCEPT", "application/hal+json",
                                        "application/hal+json;charset=utf-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn accept_header_does_not_match_when_charsets_are_not_equal() {
    let mismatches = match_header_value("ACCEPT", "application/hal+json;charset=utf-8",
                                        "application/hal+json;charset=utf-16", &CoreMatchingContext::default());
    expect!(mismatches).to(be_err());
  }

  #[test]
  fn accept_header_does_match_when_charsets_are_different_case() {
    let mismatches = match_header_value("ACCEPT", "application/hal+json;charset=utf-8",
                                        "application/hal+json;charset=UTF-8", &CoreMatchingContext::default());
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn mismatched_header_description_reports_accept_header_mismatches_correctly() {
    let mismatches = match_header_value("ACCEPT", "ACCEPT-VALUE", "HEADER2",
                                        &CoreMatchingContext::default());
    match mismatches.unwrap_err()[0] {
      Mismatch::HeaderMismatch { ref mismatch, .. } =>
        assert_eq!(mismatch, "Mismatch with header 'ACCEPT': Expected header 'ACCEPT' to have value 'ACCEPT-VALUE' but was 'HEADER2'"),
      _ => panic!("Unexpected mismatch response")
    }
  }

  #[test]
  fn accept_header_matching_with_multiple_values() {
    let expected = Some(hashmap! { "accept".to_string() => vec!["application/json".to_string(), "application/hal+json".to_string()] });
    let actual = Some(hashmap! { "accept".to_string() => vec!["application/json".to_string(), "application/hal+json".to_string()] });
    let result = match_headers(expected, actual, &CoreMatchingContext::default());
    expect!(result.values().flatten()).to(be_empty());
  }

  #[test]
  fn matching_headers_be_true_when_headers_match_by_matcher() {
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
        "header" => {
          "HEADER" => [ MatchingRule::Regex(s!("\\w+")) ]
        }
      }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    );
    let mismatches = match_header_value("HEADER", "HEADERX", "HEADERY", &context);
    expect!(mismatches).to(be_ok());
  }

  #[test]
  fn matching_headers_be_false_when_headers_do_not_match_by_matcher() {
    let context = CoreMatchingContext::new(
      DiffConfig::AllowUnexpectedKeys,
      &matchingrules! {
          "header" => {
              "HEADER" => [ MatchingRule::Regex(s!("\\d+")) ]
          }
        }.rules_for_category("header").unwrap_or_default(), &hashmap!{}
    );
    let mismatches = match_header_value(&s!("HEADER"), &s!("HEADER"), &s!("HEADER"), &context);
    expect!(mismatches).to(be_err().value(vec![ Mismatch::HeaderMismatch {
      key: s!("HEADER"),
      expected: s!("HEADER"),
      actual: s!("HEADER"),
      mismatch: s!(""),
    } ]));
  }
}
