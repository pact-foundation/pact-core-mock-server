use serde_json::Value;
use crate::{DiffConfig, Mismatch};
use crate::models::matchingrules::{MatchingRules, RuleLogic, MatchingRule};
use crate::matchers::{Matches, select_best_matcher, match_values};
use itertools::Itertools;
use log::*;
use crate::models::HttpPart;
use hyper::header::{Headers, ContentType};
use std::collections::HashMap;
use formdata::FilePart;
use onig::Regex;
use std::fs;

static ROOT: &str = "$";

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
    Err(format!("Expected binary contents to have content type '{}' but detected contents was '{}'",
      expected, result))
  }
}

pub fn convert_data(data: &Value) -> Vec<u8> {
  match data {
    &Value::String(ref s) => base64::decode(s.as_str()).unwrap_or_else(|_| s.clone().into_bytes()),
    _ => data.to_string().into_bytes()
  }
}

pub fn match_octet_stream(expected: &dyn HttpPart, actual: &dyn HttpPart, _config: DiffConfig, matchers: &MatchingRules) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  let expected = expected.body().value();
  let actual = actual.body().value();
  debug!("matching binary contents ({} bytes)", actual.len());
  let path = vec!["$".to_string()];
  if matchers.matcher_is_defined("body", &path) {
    let matching_rules = select_best_matcher("body", &path, &matchers);
    match matching_rules {
      None => mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.clone()),
        actual: Some(actual.clone()),
        mismatch: format!("No matcher found for category 'body' and path '{}'", path.iter().join("."))}),
      Some(ref rulelist) => {
        let results = rulelist.rules.iter().map(|rule| expected.matches(&actual, rule)).collect::<Vec<Result<(), String>>>();
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

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

pub fn match_mime_multipart(expected: &dyn HttpPart, actual: &dyn HttpPart, _config: DiffConfig, matchers: &MatchingRules) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  debug!("matching MIME multipart contents");

  let actual_headers = get_headers(actual.headers().clone());
  let actual_form_data = formdata::read_formdata(&mut actual.body().value().as_slice(), &actual_headers);

  let expected_headers = get_headers(expected.headers().clone());
  let expected_form_data = formdata::read_formdata(&mut expected.body().value().as_slice(), &expected_headers);

  if expected_form_data.is_err() || actual_form_data.is_err() {
    match expected_form_data {
      Err(e) => {
        mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
          actual: Some(actual.body().value().clone().into()),
          mismatch: format!("Failed to parse the expected body as a MIME multipart body: '{:?}'", e)});
      },
      _ => ()
    }
    match actual_form_data {
      Err(e) => {
        mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(expected.body().value().clone().into()),
          actual: Some(actual.body().value().clone().into()),
          mismatch: format!("Failed to parse the actual body as a MIME multipart body: '{:?}'", e)});
      },
      _ => ()
    }
  } else {
    let actual_form_data = actual_form_data.unwrap();
    let expected_form_data = expected_form_data.unwrap();
    for (key, value) in expected_form_data.fields {
      debug!("Comparing MIME field multipart '{}'", key);
      match actual_form_data.fields.iter().find(|(k, _)| *k == key) {
        Some((_, actual)) => match_field(&key, &value, actual, &mut mismatches, matchers),
        None => {
          debug!("MIME multipart '{}' is missing in the actual body", key);
          mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(key.clone().into_bytes()),
            actual: None,
            mismatch: format!("Expected a MIME part '{}' but was missing", key)});
        }
      }
    }
    for (key, value) in expected_form_data.files {
      debug!("Comparing MIME file multipart '{}'", key);
      match actual_form_data.files.iter().find(|(k, _)| *k == key) {
        Some((_, actual)) => match_file(&key, &value, actual, &mut mismatches, matchers),
        None => {
          debug!("MIME multipart '{}' is missing in the actual body", key);
          mismatches.push(Mismatch::BodyMismatch { path: s!("$"), expected: Some(key.clone().into_bytes()),
            actual: None,
            mismatch: format!("Expected a MIME part '{}' but was missing", key)});
        }
      }
    }
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

fn get_headers(h: Option<HashMap<String, Vec<String>>>) -> Headers {
  let mut headers = Headers::new();
  if let Some(h) = h {
    for (key, values) in h {
      for value in values {
        headers.append_raw(key.clone(), value.as_bytes().to_vec());
      }
    }
  };
  headers
}

fn match_field(key: &String, expected: &String, actual: &String, mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  let path = vec![ROOT.to_string(), key.clone()];
  let matcher_result = if matchers.matcher_is_defined("body", &path) {
    debug!("Calling match_values for path $.{}", key);
    match_values("body", &path, matchers.clone(), expected, actual)
  } else {
    expected.matches(actual, &MatchingRule::Equality).map_err(|err|
      vec![format!("MIME part '{}': {}", key, err)]
    )
  };
  log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.join("."), matcher_result);
  match matcher_result {
    Err(messages) => {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: path.join("."),
          expected: Some(expected.as_bytes().to_vec()),
          actual: Some(actual.as_bytes().to_vec()),
          mismatch: message.clone()
        })
      }
    },
    Ok(_) => ()
  }
}

fn first(vec: Vec<u8>, len: usize) -> Vec<u8> {
  if vec.len() <= len {
    vec
  } else {
    vec.split_at(len).0.to_vec()
  }
}

impl Matches<FilePart> for FilePart {
  fn matches(&self, actual: &FilePart, matcher: &MatchingRule) -> Result<(), String> {
    log::debug!("FilePart: comparing binary data to '{:?}' using {:?}", actual.content_type(), matcher);
    match matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            match fs::read_to_string(actual.path.clone()) {
              Ok(a) => if re.is_match(&a) {
                  Ok(())
                } else {
                  Err(format!("Expected binary file '{}' to match '{}'", actual.path.to_string_lossy(), regex))
                },
              Err(err) => Err(format!("Expected binary file to match '{}' but could not read the file '{}' - {}",
                                      regex, actual.path.to_string_lossy(), err))
            }
          },
          Err(err) => Err(format!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        let expected_contents = fs::read(self.path.clone());
        let actual_contents = fs::read(actual.path.clone());
        match (expected_contents, actual_contents) {
          (Err(err), Ok(_)) => Err(format!("Could not read binary file '{}' - {}", self.path.to_string_lossy(), err)),
          (_, Err(err)) => Err(format!("Could not read binary file '{}' - {}", actual.path.to_string_lossy(), err)),
          (Ok(expected_contents), Ok(actual_contents)) => if expected_contents == actual_contents {
              Ok(())
            } else {
              Err(format!("Expected binary file ({} bytes) starting with {:?} to be equal to ({} bytes) starting with {:?}",
                          actual_contents.len(), first(actual_contents, 20),
                          expected_contents.len(), first(expected_contents, 20)))
            }
        }
      },
      MatchingRule::Include(ref substr) => {
        match fs::read_to_string(actual.path.clone()) {
          Ok(actual_contents) => if actual_contents.contains(substr) {
            Ok(())
          } else {
            Err(format!("Expected binary file ({}) to include '{}'", actual.path.to_string_lossy(), substr))
          },
          Err(err) => Err(format!("Expected binary file to include '{}' but could not read the file '{}' - {}",
                                  substr, actual.path.to_string_lossy(), err))
        }
      },
      MatchingRule::ContentType(content_type) => {
        match fs::read(actual.path.clone()) {
          Ok(actual_contents) => match_content_type(&actual_contents, content_type),
          Err(err) => Err(format!("Expected binary file to have content type '{}' but could not read the file '{}' - {}",
                                  content_type, actual.path.to_string_lossy(), err))
        }
      },
      _ => Err(format!("Unable to match binary file using {:?}", matcher))
    }
  }
}

fn match_file(key: &String, expected: &FilePart, actual: &FilePart, mismatches: &mut Vec<Mismatch>, matchers: &MatchingRules) {
  let path = vec![ROOT.to_string(), key.clone()];
  let matcher_result = if matchers.matcher_is_defined("body", &path) {
    debug!("Calling match_values for path $.{}", key);
    match_values("body", &path, matchers.clone(), expected, actual)
  } else {
    let expected_ct: Option<&ContentType> = expected.headers.get();
    let actual_ct: Option<&ContentType> = actual.headers.get();
    if expected_ct == actual_ct {
      expected.matches(actual, &MatchingRule::Equality).map_err(|err|
        vec![format!("MIME part '{}': {}", key, err)]
      )
    } else {
      let expected_str = if expected_ct.is_some() {
        expected_ct.unwrap().to_string()
      } else {
        "None".to_string()
      };
      let actual_str = if actual_ct.is_some() {
        actual_ct.unwrap().to_string()
      } else {
        "None".to_string()
      };
      mismatches.push(Mismatch::BodyTypeMismatch {
        expected: expected_str.clone(),
        actual: actual_str.clone(),
        mismatch: format!("Expected MIME part '{}' with content type '{}' but was '{}'",
                          key, expected_str, actual_str)
      });
      Ok(())
    }
  };
  log::debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.join("."), matcher_result);
  match matcher_result {
    Err(messages) => {
      for message in messages {
        mismatches.push(Mismatch::BodyMismatch {
          path: path.join("."),
          expected: None,
          actual: None,
          mismatch: message.clone()
        })
      }
    },
    Ok(_) => ()
  }
}

#[cfg(test)]
mod tests {
  use crate::models::{Request, OptionalBody, HttpPart};
  use crate::models::matchingrules::*;
  use crate::binary_utils::match_mime_multipart;
  use expectest::prelude::*;
  use hamcrest2::prelude::*;
  use crate::{DiffConfig, Mismatch};
  use maplit::*;
  use std::str;

  fn mismatch(m: &Mismatch) -> &str {
    match m {
      Mismatch::BodyMismatch { mismatch, .. } => mismatch.as_str(),
      Mismatch::BodyTypeMismatch { mismatch, .. } => mismatch.as_str(),
      _ => ""
    }
  }

  #[test]
  fn match_mime_multipart_error_when_not_multipart() {
    let body = "not a multipart body";
    let request = Request {
      headers: Some(hashmap!{}),
      body: OptionalBody::Present(body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&request, &request, DiffConfig::AllowUnexpectedKeys,
                         &request.matching_rules());

    let mismatches = result.unwrap_err();
    assert_that!(&mismatches, len(2));
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Failed to parse the expected body as a MIME multipart body: \'A MIME multipart error occurred.\'",
      "Failed to parse the actual body as a MIME multipart body: \'A MIME multipart error occurred.\'"
    ]));
  }

  #[test]
  fn match_mime_multipart_equal() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_missing_part() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());
    let mismatches = result.unwrap_err();
    expect(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected a MIME part \'age\' but was missing", "Expected a MIME part \'file\' but was missing"
    ]));
  }

  #[test]
  fn match_mime_multipart_different_values() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nFred\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n2 months\r\n\
      --4567\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'name': Expected 'Baxter' to be equal to 'Fred'",
      "MIME part 'age': Expected '1 month' to be equal to '2 months'",
      "MIME part 'file': Expected binary file (16 bytes) starting with [97, 44, 98, 44, 99, 44, 100, 13, 10, 52, 44, 53, 44, 54, 44, 55] to be equal to (16 bytes) starting with [49, 44, 50, 44, 51, 44, 52, 13, 10, 52, 44, 53, 44, 54, 44, 55]"
    ]));
  }

  #[test]
  fn match_mime_multipart_with_matching_rule() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.name" => [ MatchingRule::Regex(s!("^\\w+$")) ],
          "$.age" => [ MatchingRule::Regex(s!("^\\d+ months?+$")) ]
        }
      },
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nFred\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n2 months\r\n\
      --4567\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_different_content_type() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.csv\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: text/html\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      <html>a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected MIME part 'file' with content type 'text/csv' but was 'text/html'"
    ]));
  }

  #[test]
  fn match_mime_multipart_content_type_matcher() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: image/png\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.htm\"\r\n\r\n\
      <html>1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.file" => [ MatchingRule::ContentType("image/png".into()) ]
        }
      },
      ..Request::default()
    };
    let bytes: [u8; 16] = [
        0x89, 0x50, 0x4e, 0x47,
        0x0d, 0x0a, 0x1a, 0x0a,
        0x00, 0x00, 0x00, 0x0d,
        0x49, 0x48, 0x44, 0x52
     ];

    let mut actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: image/png\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.htm\"\r\n\r\n".as_bytes().to_vec();
    actual_body.extend_from_slice(&bytes);
    actual_body.extend_from_slice("\r\n--4567--\r\n".as_bytes());
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_content_type_matcher_with_mismatch() {
    let expected_body = "--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234\r\n\
      Content-Type: application/jpeg\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"008.htm\"\r\n\r\n\
      1,2,3,4\r\n\
      4,5,6,7\r\n\
      --1234--\r\n";
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body.as_bytes().to_vec(), None),
      matching_rules: matchingrules! {
        "body" => {
          "$.file" => [ MatchingRule::ContentType("application/jpeg".into()) ]
        }
      },
      ..Request::default()
    };
    let actual_body = "--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: application/jpeg\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.htm\"\r\n\r\n\
      a,b,c,d\r\n\
      4,5,6,7\r\n\
      --4567--\r\n";
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.as_bytes().to_vec(), None),
      ..Request::default()
    };

    let result = match_mime_multipart(&expected, &actual, DiffConfig::AllowUnexpectedKeys,
                         &expected.matching_rules());

    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected binary contents to have content type 'application/jpeg' but detected contents was 'text/plain'"
    ]));
  }
}
