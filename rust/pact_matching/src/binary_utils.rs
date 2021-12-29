use std::collections::HashMap;
use std::convert::TryInto;
use std::str::from_utf8;

use anyhow::anyhow;
use bytes::{Buf, Bytes};
use http::header::{HeaderMap, HeaderName};
use log::*;
use onig::Regex;
use pact_models::content_types::{ContentType, detect_content_type_from_bytes};
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::{MatchingRule, RuleLogic};
use pact_models::path_exp::DocPath;
use serde_json::Value;

use crate::{MatchingContext, Mismatch};
use crate::matchers::{match_values, Matches};

pub fn match_content_type<S>(data: &[u8], expected_content_type: S) -> anyhow::Result<()>
  where S: Into<String> {
  let result = tree_magic_mini::from_u8(data);
  let expected = expected_content_type.into();
  let matches = result == expected;
  debug!("Matching binary contents by content type: expected '{}', detected '{}' -> {}",
         expected, result, matches);
  if matches {
    Ok(())
  } else if result == "text/plain" {
    detect_content_type_from_bytes(data)
      .and_then(|ct| if ct ==  ContentType::from(&expected) { Some(()) } else { None })
      .ok_or_else(|| anyhow!("Expected binary contents to have content type '{}' but detected contents was '{}'",
        expected, result))
  } else {
    Err(anyhow!("Expected binary contents to have content type '{}' but detected contents was '{}'",
      expected, result))
  }
}

pub fn convert_data(data: &Value) -> Vec<u8> {
  match data {
    Value::String(s) => base64::decode(s.as_str()).unwrap_or_else(|_| s.clone().into_bytes()),
    _ => data.to_string().into_bytes()
  }
}

pub fn match_octet_stream(
  expected: &dyn HttpPart,
  actual: &dyn HttpPart,
  context: &dyn MatchingContext
) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  let expected = expected.body().value().unwrap_or_default();
  let actual = actual.body().value().unwrap_or_default();
  debug!("matching binary contents ({} bytes)", actual.len());
  let path = DocPath::root();
  if context.matcher_is_defined(&path) {
    let matchers = context.select_best_matcher(&path);
    if matchers.is_empty() {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: Some(expected),
        actual: Some(actual),
        mismatch: format!("No matcher found for category 'body' and path '{}'", path),
      })
    } else {
      let results = matchers.rules.iter().map(|rule|
        expected.matches_with(&actual, rule, matchers.cascaded)).collect::<Vec<anyhow::Result<()>>>();
      match matchers.rule_logic {
        RuleLogic::And => for result in results {
          if let Err(err) = result {
            mismatches.push(Mismatch::BodyMismatch {
              path: "$".into(),
              expected: Some(expected.clone()),
              actual: Some(actual.clone()),
              mismatch: err.to_string(),
            })
          }
        },
        RuleLogic::Or => {
          if results.iter().all(|result| result.is_err()) {
            for result in results {
              if let Err(err) = result {
                mismatches.push(Mismatch::BodyMismatch {
                  path: "$".into(),
                  expected: Some(expected.clone()),
                  actual: Some(actual.clone()),
                  mismatch: err.to_string(),
                })
              }
            }
          }
        }
      }
    }
  } else if expected != actual {
    mismatches.push(Mismatch::BodyMismatch {
      path: "$".into(),
      expected: Some(expected.clone()),
      actual: Some(actual.clone()),
      mismatch: format!("Expected binary data of {} bytes but received {} bytes",
                        expected.len(), actual.len()),
    });
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

enum MimePart {
  Field(MimeField),
  File(MimeFile)
}

impl MimePart {
  fn name(&self) -> &String {
    match self {
      Self::Field(field) => &field.name,
      Self::File(file) => &file.name,
    }
  }
}

struct MimeField {
  name: String,
  data: String
}

#[derive(Debug)]
struct MimeFile {
  name: String,
  content_type: Option<mime::Mime>,
  filename: String,
  data: Bytes
}

pub fn match_mime_multipart(
  expected: &dyn HttpPart,
  actual: &dyn HttpPart,
  context: &dyn MatchingContext
) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  debug!("matching MIME multipart contents");

  let actual_parts = parse_multipart(actual.body().value().unwrap_or_default().reader(), actual.headers());
  let expected_parts = parse_multipart(expected.body().value().unwrap_or_default().reader(), expected.headers());

  if expected_parts.is_err() || actual_parts.is_err() {
    if let Err(e) = expected_parts {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the expected body as a MIME multipart body: '{}'", e)
      });
    }
    if let Err(e) = actual_parts {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: expected.body().value(),
        actual: actual.body().value(),
        mismatch: format!("Failed to parse the actual body as a MIME multipart body: '{}'", e)
      });
    }
  } else {
    let actual_parts = actual_parts.unwrap();
    let expected_parts = expected_parts.unwrap();

    for expected_part in expected_parts {
      let name = expected_part.name();

      debug!("Comparing MIME field multipart '{}'", expected_part.name());
      match actual_parts.iter().find(|part| part.name() == expected_part.name()) {
        Some(actual_part) => for error in match_mime_part(&expected_part, actual_part, context).err().unwrap_or_default() {
          mismatches.push(error);
        },
        None => {
          debug!("MIME multipart '{}' is missing in the actual body", name);
          mismatches.push(Mismatch::BodyMismatch {
            path: "$".into(),
            expected: Some(Bytes::from(name.clone())),
            actual: None,
            mismatch: format!("Expected a MIME part '{}' but was missing", name)});
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

fn match_mime_part(
  expected: &MimePart,
  actual: &MimePart,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let key = expected.name();

  match (expected, actual) {
    (MimePart::Field(expected_field), MimePart::Field(actual_field)) => {
      match_field(key, &expected_field.data, &actual_field.data, context)
    },
    (MimePart::File(expected_file), MimePart::File(actual_file)) => {
      match_file(key, expected_file, actual_file, context)
    }
    (MimePart::Field(_), MimePart::File(_)) => {
      Err(vec![
        Mismatch::BodyMismatch { path: "$".into(),
          expected: Some(Bytes::from(key.clone())),
          actual: None,
          mismatch: format!("Expected a MIME field '{}' but was file", key)}
      ])
    },
    (MimePart::File(_), MimePart::Field(_)) => {
      Err(vec![
        Mismatch::BodyMismatch { path: "$".into(),
          expected: Some(Bytes::from(key.clone())),
          actual: None,
          mismatch: format!("Expected a MIME file '{}' but was field", key)}
      ])
    }
  }
}

fn match_field(
  key: &str,
  expected: &str,
  actual: &str,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::root().join(key);
  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path $.{}", key);
    match_values(&path, &context.select_best_matcher(&path), expected, actual)
  } else {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err|
      vec![format!("MIME part '{}': {}", key, err)]
    )
  };
  debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path, matcher_result);
  matcher_result.map_err(|messages| {
    messages.iter().map(|message| {
      Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(Bytes::from(expected.as_bytes().to_vec())),
        actual: Some(Bytes::from(actual.as_bytes().to_vec())),
        mismatch: message.clone()
      }
    }).collect()
  })
}

fn first(bytes: &[u8], len: usize) -> &[u8] {
  if bytes.len() <= len {
    bytes
  } else {
    bytes.split_at(len).0
  }
}

impl Matches<&MimeFile> for &MimeFile {
  fn matches_with(&self, actual: &MimeFile, matcher: &MatchingRule, _cascaded: bool) -> anyhow::Result<()> {
    debug!("FilePart: comparing binary data to '{:?}' using {:?}", actual.content_type, matcher);
    match matcher {
      MatchingRule::Regex(ref regex) => {
        match Regex::new(regex) {
          Ok(re) => {
            match from_utf8(&*actual.data) {
              Ok(a) => if re.is_match(a) {
                  Ok(())
                } else {
                  Err(anyhow!("Expected binary file '{}' to match '{}'", actual.filename, regex))
                },
              Err(err) => Err(anyhow!("Expected binary file to match '{}' but could convert the file to a string '{}' - {}",
                                      regex, actual.filename, err))
            }
          },
          Err(err) => Err(anyhow!("'{}' is not a valid regular expression - {}", regex, err))
        }
      },
      MatchingRule::Equality => {
        if self.data == actual.data {
          Ok(())
        } else {
          Err(anyhow!("Expected binary file ({} bytes) starting with {:?} to be equal to ({} bytes) starting with {:?}",
          actual.data.len(), first(&actual.data, 20),
          self.data.len(), first(&self.data, 20)))
        }
      },
      MatchingRule::Include(ref substr) => {
        match from_utf8(&*actual.data) {
          Ok(actual_contents) => if actual_contents.contains(substr) {
            Ok(())
          } else {
            Err(anyhow!("Expected binary file ({}) to include '{}'", actual.filename, substr))
          },
          Err(err) => Err(anyhow!("Expected binary file to include '{}' but could not convert the file to a string '{}' - {}",
                                  substr, actual.filename, err))
        }
      },
      MatchingRule::ContentType(content_type) => match_content_type(&actual.data, content_type),
      _ => Err(anyhow!("Unable to match binary file using {:?}", matcher))
    }
  }
}

fn match_file(
  key: &str,
  expected: &MimeFile,
  actual: &MimeFile,
  context: &dyn MatchingContext
) -> Result<(), Vec<Mismatch>> {
  let path = DocPath::root().join(key);
  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path $.{}", key);
    match_values( &path, &context.select_best_matcher(&path), expected, actual).map_err(|errors| {
      errors.iter().map(|err| Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: None,
        actual: None,
        mismatch: format!("MIME part '{}': {}", key, err)
      }).collect()
    })
  } else if expected.content_type == actual.content_type {
    expected.matches_with(actual, &MatchingRule::Equality, false).map_err(|err|
      vec![Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: None,
        actual: None,
        mismatch: format!("MIME part '{}': {}", key, err)
      }]
    )
  } else {
    let expected_str = expected.content_type.as_ref().map(|mime| mime.to_string()).unwrap_or_else(|| "None".to_string());
    let actual_str = actual.content_type.as_ref().map(|mime| mime.to_string()).unwrap_or_else(|| "None".to_string());
    Err(vec![Mismatch::BodyTypeMismatch {
      expected: expected_str.clone(),
      actual: actual_str.clone(),
      mismatch: format!("Expected MIME part '{}' with content type '{}' but was '{}'",
                        key, expected_str, actual_str),
      expected_body: None,
      actual_body: None
    }])
  };
  debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.to_string(), matcher_result);
  matcher_result
}

fn parse_multipart<R: std::io::Read>(body: R, headers: &Option<HashMap<String, Vec<String>>>) -> Result<Vec<MimePart>, String> {
  let boundary = get_multipart_boundary(headers)?;
  let mut mp = multipart::server::Multipart::with_body(body, boundary);

  let mut parts = vec![];

  use std::io::Read;

  loop {
    match mp.read_entry() {
      Ok(Some(mut entry)) => {
        let name = entry.headers.name.to_string();
        let content_type = entry.headers.content_type;

        let mut data = vec![];
        entry.data.read_to_end(&mut data).map_err(|e| format!("Failed to read multipart data: {}", e))?;

        if let Some(filename) = entry.headers.filename {
          parts.push(MimePart::File(MimeFile {
            name,
            content_type,
            filename,
            data: Bytes::from(data),
          }));
        } else {
          parts.push(MimePart::Field(MimeField {
            name,
            data: String::from_utf8(data).map_err(|e| format!("Decode error: {}", e))?
          }))
        }
      },
      Ok(None) => return Ok(parts),
      Err(e) => return Err(format!("Failed to read multipart entry: {}", e)),
    }
  }
}

fn get_multipart_boundary(headers: &Option<HashMap<String, Vec<String>>>) -> Result<String, String> {
  let header_map = get_http_header_map(headers);
  let content_type = header_map.get(http::header::CONTENT_TYPE)
    .ok_or_else(|| "no content-type".to_owned())?
    .to_str()
    .map_err(|e| format!("invalid content-type: {}", e))?;

  let mime: mime::Mime = content_type.parse().map_err(|e| format!("invalid content-type: {}", e))?;

  if mime.type_() != mime::MULTIPART || mime.subtype() != mime::FORM_DATA {
    return Err("expected content-type to be multipart/form-data".to_string());
  }

  let boundary = mime.get_param(mime::BOUNDARY).ok_or_else(|| "no boundary in content-type".to_string())?;

  Ok(boundary.as_str().to_owned())
}

fn get_http_header_map(h: &Option<HashMap<String, Vec<String>>>) -> HeaderMap {
  let mut headers = HeaderMap::new();
  if let Some(h) = h {
    for (key, values) in h {
      for value in values {
        if let (Ok(header_name), Ok(header_value)) = (HeaderName::from_bytes(key.as_bytes()), value.try_into()) {
          headers.append(header_name, header_value);
        }
      }
    }
  };
  headers
}

#[cfg(test)]
mod tests {
  use std::str;

  use bytes::{Bytes, BytesMut};
  use expectest::prelude::*;
  use hamcrest2::prelude::*;
  use maplit::*;
  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::request::Request;

  use crate::{CoreMatchingContext, DiffConfig, Mismatch};
  use crate::binary_utils::{match_content_type, match_mime_multipart};

  fn mismatch(m: &Mismatch) -> &str {
    match m {
      Mismatch::BodyMismatch { mismatch, .. } => mismatch.as_str(),
      Mismatch::BodyTypeMismatch { mismatch, .. } => mismatch.as_str(),
      _ => ""
    }
  }

  #[test]
  fn match_mime_multipart_error_when_not_multipart() {
    let body = Bytes::from("not a multipart body");
    let request = Request {
      headers: Some(hashmap!{}),
      body: OptionalBody::Present(body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&request, &request, &context);

    let mismatches = result.unwrap_err();
    assert_that!(&mismatches, len(2));
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Failed to parse the expected body as a MIME multipart body: \'no content-type\'",
      "Failed to parse the actual body as a MIME multipart body: \'no content-type\'"
    ]));
  }

  #[test]
  fn match_mime_multipart_equal() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_missing_part() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect(mismatches.iter()).to_not(be_empty());
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected a MIME part \'age\' but was missing", "Expected a MIME part \'file\' but was missing"
    ]));
  }

  #[test]
  fn match_mime_multipart_different_values() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
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
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'name': Expected 'Baxter' to be equal to 'Fred'",
      "MIME part 'age': Expected '1 month' to be equal to '2 months'",
      "MIME part 'file': Expected binary file (16 bytes) starting with [97, 44, 98, 44, 99, 44, 100, 13, 10, 52, 44, 53, 44, 54, 44, 55] to be equal to (16 bytes) starting with [49, 44, 50, 44, 51, 44, 52, 13, 10, 52, 44, 53, 44, 54, 44, 55]"
    ]));
  }

  #[test]
  fn match_mime_multipart_with_matching_rule() {
    let expected_body = Bytes::from("--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      matching_rules: matchingrules! {
        "body" => {
          "$.name" => [ MatchingRule::Regex(s!("^\\w+$")) ],
          "$.age" => [ MatchingRule::Regex(s!("^\\d+ months?+$")) ]
        }
      },
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
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
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap(), &hashmap!{});

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  fn match_mime_multipart_different_content_type() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
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
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "Expected MIME part 'file' with content type 'text/csv' but was 'text/html'"
    ]));
  }

  #[test]
  #[cfg(not(target_os = "windows"))] // Requires shared mime-info db, not available on Windows
  fn match_mime_multipart_content_type_matcher() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
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

    let mut actual_body = BytesMut::from("--4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: image/png\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.htm\"\r\n\r\n");
    actual_body.extend_from_slice(&bytes);
    actual_body.extend_from_slice("\r\n--4567--\r\n".as_bytes());
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body.freeze(), None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap(), &hashmap!{});

    let result = match_mime_multipart(&expected, &actual, &context);

    expect!(result).to(be_ok());
  }

  #[test]
  #[cfg(not(target_os = "windows"))] // Requires shared mime-info db, not available on Windows
  fn match_mime_multipart_content_type_matcher_with_mismatch() {
    let expected_body = Bytes::from("--1234\r\n\
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
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      matching_rules: matchingrules! {
        "body" => {
          "$.file" => [ MatchingRule::ContentType("application/jpeg".into()) ]
        }
      },
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
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
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &expected.matching_rules.rules_for_category("body").unwrap(), &hashmap!{});

    let result = match_mime_multipart(&expected, &actual, &context);

    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part \'file\': Expected binary contents to have content type \'application/jpeg\' but detected contents was \'text/plain\'"
    ]));
  }

  #[test]
  #[cfg(not(target_os = "windows"))] // Requires shared mime-info db, not available on Windows
  fn match_content_type_equals() {
    expect!(match_content_type("some text".as_bytes(), "text/plain")).to(be_ok());

    let bytes: [u8; 48] = [
      0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43,
      0x00, 0x10, 0x0b, 0x0c, 0x0e, 0x0c, 0x0a, 0x10, 0x0e, 0x0d, 0x0e, 0x12, 0x11, 0x10, 0x13, 0x18, 0x28, 0x1a, 0x18, 0x16, 0x16, 0x18, 0x31, 0x23
    ];
    expect!(match_content_type(&bytes, "image/jpeg")).to(be_ok());
  }

  #[test]
  #[cfg(not(target_os = "windows"))] // Requires shared mime-info db, not available on Windows
  fn match_content_type_common_text_types() {
    expect!(match_content_type("{\"val\": \"some text\"}".as_bytes(), "application/json")).to(be_ok());
    expect!(match_content_type("<xml version=\"1.0\"><a/>".as_bytes(), "application/xml")).to(be_ok());
  }
}
