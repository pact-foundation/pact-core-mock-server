use std::collections::HashMap;
use std::convert::Infallible;
use std::convert::TryInto;
use std::str::from_utf8;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bytes::Bytes;
use futures::stream::once;
use http::header::{HeaderMap, HeaderName};
use itertools::Itertools;
use multer::Multipart;
use onig::Regex;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, detect_content_type_from_bytes};
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::{MatchingRule, RuleLogic};
use pact_models::path_exp::{DocPath, PathToken};
use pact_models::v4::http_parts::HttpRequest;
use serde_json::Value;
use tracing::{debug, error, warn};

use crate::{BodyMatchResult, CoreMatchingContext, HeaderMatchingContext, MatchingContext, Mismatch};
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
    Value::String(s) => BASE64.decode(s.as_str()).unwrap_or_else(|_| s.clone().into_bytes()),
    _ => data.to_string().into_bytes()
  }
}

pub fn match_octet_stream(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<super::Mismatch>> {
  let mut mismatches = vec![];
  let expected_body = expected.body().value().unwrap_or_default();
  let actual_body = actual.body().value().unwrap_or_default();
  debug!("matching binary contents ({} bytes)", actual_body.len());
  let path = DocPath::root();
  if context.matcher_is_defined(&path) {
    let matchers = context.select_best_matcher(&path);
    if matchers.is_empty() {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: Some(expected_body),
        actual: Some(actual_body),
        mismatch: format!("No matcher found for category 'body' and path '{}'", path),
      })
    } else {
      let results = matchers.rules.iter().map(|rule|
        expected_body.matches_with(&actual_body, rule, matchers.cascaded)).collect::<Vec<anyhow::Result<()>>>();
      match matchers.rule_logic {
        RuleLogic::And => for result in results {
          if let Err(err) = result {
            mismatches.push(Mismatch::BodyMismatch {
              path: "$".into(),
              expected: Some(expected_body.clone()),
              actual: Some(actual_body.clone()),
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
                  expected: Some(expected_body.clone()),
                  actual: Some(actual_body.clone()),
                  mismatch: err.to_string(),
                })
              }
            }
          }
        }
      }
    }
  } else if expected_body != actual_body {
    let actual_ct = actual.content_type().unwrap_or_default();
    let expected_ct = expected.content_type().unwrap_or_default();
    mismatches.push(Mismatch::BodyMismatch {
      path: "$".into(),
      expected: Some(expected_body.clone()),
      actual: Some(actual_body.clone()),
      mismatch: format!("Actual body [{}, {} bytes, starting with {}] is not equal to the expected body [{}, {} bytes, starting with {}]",
        actual_ct, actual_body.len(), display_bytes(&actual_body, 32),
        expected_ct, expected_body.len(), display_bytes(&expected_body, 32))
    });
  }

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

fn display_bytes(bytes: &Bytes, max_bytes: usize) -> String {
  if bytes.len() <= max_bytes {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
  } else {
    bytes
      .slice(0..max_bytes)
      .iter()
      .map(|b| format!("{:02x}", b))
      .collect()
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

  fn index(&self) -> usize {
    match self {
      Self::Field(field) => field.index,
      Self::File(file) => file.index,
    }
  }
}

struct MimeField {
  index: usize,
  name: String,
  data: Bytes,
  headers: HeaderMap
}

impl MimeField {
  pub(crate) fn decode_data(&self) -> anyhow::Result<Bytes> {
    if let Some(encoding) = self.headers.get("Content-Transfer-Encoding") {
      let encoding = String::from_utf8_lossy(encoding.as_bytes());
      if encoding.to_lowercase() == "base64" {
        BASE64.decode(self.data.as_ref())
          .map(|data| Bytes::from(data))
          .map_err(|err| anyhow!(err))
      } else {
        warn!("Ignoring encoding '{}' for part '{}'", encoding, self.name);
        Ok(self.data.clone())
      }
    } else {
      Ok(self.data.clone())
    }
  }
}

#[derive(Debug)]
struct MimeFile {
  index: usize,
  name: String,
  content_type: Option<mime::Mime>,
  filename: String,
  data: Bytes,
  headers: HeaderMap
}

impl MimeFile {
  pub(crate) fn decode_data(&self) -> anyhow::Result<Bytes> {
    if let Some(encoding) = self.headers.get("Content-Transfer-Encoding") {
      let encoding = String::from_utf8_lossy(encoding.as_bytes());
      if encoding.to_lowercase() == "base64" {
        BASE64.decode(self.data.as_ref())
          .map(|data| Bytes::from(data))
          .map_err(|err| anyhow!(err))
      } else {
        warn!("Ignoring encoding '{}' for part '{}'('{}')", encoding, self.name, self.filename);
        Ok(self.data.clone())
      }
    } else {
      Ok(self.data.clone())
    }
  }
}

pub fn match_mime_multipart(
  expected: &(dyn HttpPart + Send + Sync),
  actual: &(dyn HttpPart + Send + Sync),
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<super::Mismatch>> {
  let expected_body = expected.body().clone();
  let actual_body = actual.body().clone();
  let expected_headers = expected.headers().clone();
  let actual_headers = actual.headers().clone();
  let context = CoreMatchingContext::clone_from(context);

  let (sender, receiver) = channel();
  thread::spawn(move || {
    match tokio::runtime::Handle::try_current() {
      Ok(rt) => {
        debug!("Spawning task on existing Tokio runtime");
        rt.block_on(async move {
          let results = match_mime_multipart_inner(&context,
                                                   &expected_body, &actual_body, &expected_headers, &actual_headers).await;
          if let Err(err) = sender.send(results) {
            error!("Failed to send results back via channel: {}", err);
          }
        });
      },
      Err(err) => {
        debug!("Could not get the tokio runtime, will try start a new one: {}", err);
        tokio::runtime::Builder::new_multi_thread()
          .enable_all()
          .build()
          .expect("Could not start a Tokio runtime for running async tasks")
          .block_on(async move {
            let results = match_mime_multipart_inner(&context,
                                                     &expected_body, &actual_body, &expected_headers, &actual_headers).await;
            if let Err(err) = sender.send(results) {
              error!("Failed to send results back via channel: {}", err);
            }
          })
      }
    }
  });

  let mismatches = receiver.recv_timeout(Duration::from_secs(30))
    .map_err(|err| {
      vec![
        Mismatch::BodyMismatch {
          path: "$".into(),
          expected: expected.body().value(),
          actual: actual.body().value(),
          mismatch: format!("Timeout error, failed to parse the expected body as a MIME multipart body: {}", err)
        }
      ]
    })?;

  if mismatches.is_empty() {
    Ok(())
  } else {
    Err(mismatches.clone())
  }
}

async fn match_mime_multipart_inner(
  context: &CoreMatchingContext,
  expected_body: &OptionalBody,
  actual_body: &OptionalBody,
  expected_headers: &Option<HashMap<String, Vec<String>>>,
  actual_headers: &Option<HashMap<String, Vec<String>>>
) -> Vec<Mismatch> {
  let mut mismatches = vec![];
  debug!("matching MIME multipart contents");

  let actual_parts = parse_multipart(actual_body.value().unwrap_or_default(), actual_headers).await;
  let expected_parts = parse_multipart(expected_body.value().unwrap_or_default(), expected_headers).await;

  if expected_parts.is_err() || actual_parts.is_err() {
    if let Err(e) = expected_parts {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: expected_body.value(),
        actual: actual_body.value(),
        mismatch: format!("Failed to parse the expected body as a MIME multipart body: '{}'", e)
      });
    }
    if let Err(e) = actual_parts {
      mismatches.push(Mismatch::BodyMismatch {
        path: "$".into(),
        expected: expected_body.value(),
        actual: actual_body.value(),
        mismatch: format!("Failed to parse the actual body as a MIME multipart body: '{}'", e)
      });
    }
  } else {
    let actual_parts = actual_parts.unwrap();
    let expected_parts = expected_parts.unwrap();

    debug!("Expected has {} part(s), actual has {} part(s)", expected_parts.len(), actual_parts.len());

    for expected_part in expected_parts {
      let name = expected_part.name();

      debug!("Comparing MIME multipart {}:'{}'", expected_part.index(), expected_part.name());
      match actual_parts.iter().find(|part| {
        let name = part.name();
        if name.is_empty() {
          part.index() == expected_part.index()
        } else {
          name == expected_part.name()
        }
      }) {
        Some(actual_part) => for error in match_mime_part(&expected_part, actual_part, context).await
          .err().unwrap_or_default() {
          mismatches.push(error);
        },
        None => {
          debug!("MIME multipart '{}' is missing in the actual body", name);
          mismatches.push(Mismatch::BodyMismatch {
            path: "$".into(),
            expected: Some(Bytes::from(name.clone())),
            actual: None,
            mismatch: format!("Expected a MIME part '{}' but was missing", name)
          });
        }
      }
    }
  }
  mismatches
}

async fn match_mime_part(
  expected: &MimePart,
  actual: &MimePart,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  let key = expected.name();

  match (expected, actual) {
    (MimePart::Field(expected_field), MimePart::Field(actual_field)) => {
      match_field(key, &expected_field, &actual_field, context)
    },
    (MimePart::File(expected_file), MimePart::File(actual_file)) => {
      match_file_part(key, expected_file, actual_file, context).await
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
  expected: &MimeField,
  actual: &MimeField,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  debug!("Comparing MIME part '{}' as a field", key);
  let path = if key.is_empty() {
    DocPath::root().join(expected.index.to_string())
  } else {
    DocPath::root().join(key)
  };
  let expected_str = match expected.decode_data() {
    Ok(data) => String::from_utf8_lossy(data.as_ref()).to_string(),
    Err(err) => {
      error!("Failed to decode mine part '{}': {}", key, err);
      String::from_utf8_lossy(expected.data.as_ref()).to_string()
    }
  };
  let actual_str = match actual.decode_data() {
    Ok(data) => String::from_utf8_lossy(data.as_ref()).to_string(),
    Err(err) => {
      error!("Failed to decode mine part '{}': {}", key, err);
      String::from_utf8_lossy(actual.data.as_ref()).to_string()
    }
  };

  let header_result = match_headers(&path, &expected.headers, &actual.headers, context);
  debug!("Comparing headers at path '{}' -> {:?}", path, header_result);

  let matcher_result = if context.matcher_is_defined(&path) {
    debug!("Calling match_values for path $.{}", key);
    match_values(&path, &context.select_best_matcher(&path), expected_str.as_str(), actual_str.as_str())
  } else {
    expected_str.matches_with(actual_str.as_str(), &MatchingRule::Equality, false).map_err(|err|
      vec![format!("MIME part '{}': {}", key, err)]
    )
  };
  debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected_str, actual_str, path, matcher_result);

  let mut results = vec![];
  if let Err(header_mismatches) = header_result {
    results.extend(header_mismatches);
  }
  if let Err(messages) = matcher_result {
    results.extend(messages.iter().map(|message| {
      Mismatch::BodyMismatch {
        path: path.to_string(),
        expected: Some(expected.data.clone()),
        actual: Some(actual.data.clone()),
        mismatch: message.clone()
      }
    }));
  }

  if results.is_empty() {
    Ok(())
  } else {
    Err(results)
  }
}

pub(crate) fn match_headers(
  path: &DocPath,
  expected: &HeaderMap,
  actual: &HeaderMap,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  let mut results = vec![];
  let header_context = HeaderMatchingContext::new(context);

  for key in expected.keys() {
    let key_path = path.join(key.to_string());
    let expected_value = expected.get(key).unwrap().clone();
    let expected_value_bin = expected_value.as_bytes();
    let expected_value_str = String::from_utf8_lossy(expected_value_bin).to_string();

    // TODO: Replace with DocPath::last_field when pact_models 1.1.8 released
    let part_name = last_field(path).unwrap_or("unknown part");

    if let Some(actual_value) = actual.get(key) {
      let actual_value_bin = actual_value.as_bytes();
      let actual_value_str = String::from_utf8_lossy(actual_value_bin).to_string();
      let matcher_result = if header_context.direct_matcher_defined(&key_path, &Default::default()) {
        debug!("Matcher is defines, calling match_values for path {}", key_path);
        match_values(&key_path, &header_context.select_best_matcher(&key_path),
                     expected_value_str.as_str(), actual_value_str.as_str())
      } else if key == "content-disposition" {
        Ok(())
      } else {
        expected_value_str.matches_with(actual_value_str.as_str(), &MatchingRule::Equality, false).map_err(|err|
          vec![format!("header '{}': {}", key, err)]
        )
      };
      debug!("Comparing '{:?}' to '{:?}' at path '{}' -> {:?}", expected_value_str, actual_value_str, key_path, matcher_result);
      if let Err(mismatches) = matcher_result {
        results.extend(mismatches.iter().map(|m| {
          Mismatch::BodyMismatch {
            path: key_path.to_string(),
            expected: Some(Bytes::from(expected_value_str.clone())),
            actual: Some(Bytes::from(actual_value_str.clone())),
            mismatch: format!("MIME part '{}': {}", part_name, m)
          }
        }));
      }
    } else if key == "content-type" || key == "content-disposition" || key == "content-transfer-encoding" {
      debug!("Ignoring missing content-* headers: {}", key);
    } else {
      results.push(Mismatch::BodyMismatch {
        path: key_path.to_string(),
        expected: Some(Bytes::from(expected_value_str.clone())),
        actual: None,
        mismatch: format!("MIME part '{}': Expected multipart header '{}' with value '{}' but was missing",
          part_name, key, expected_value_str)
      });
    }
  }

  if results.is_empty() {
    Ok(())
  } else {
    Err(results)
  }
}

// TODO: Replace with DocPath::last_field when pact_models 1.1.8 released
fn last_field(path: &DocPath) -> Option<&str> {
  for token in path.tokens().iter().rev() {
    if let PathToken::Field(ref field) = token {
      return Some(field);
    }
  }
  return None;
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

async fn match_file_part(
  key: &str,
  expected: &MimeFile,
  actual: &MimeFile,
  context: &(dyn MatchingContext + Send + Sync)
) -> Result<(), Vec<Mismatch>> {
  let part_name = if key.is_empty() {
    expected.index.to_string()
  } else {
    key.to_string()
  };
  debug!("Comparing MIME part '{}' as binary data", part_name);
  let path = if key.is_empty() {
    DocPath::root().join(expected.index.to_string())
  } else {
    DocPath::root().join(key)
  };

  let header_result = match_headers(&path, &expected.headers, &actual.headers, context);
  debug!("Comparing headers at path '{}' -> {:?}", path, header_result);

  debug!("Expected part headers: {:?}", expected.headers);
  debug!("Expected part body: [{:?}]", expected.data);
  debug!("Actual part headers: {:?}", actual.headers);
  debug!("Actual part body: [{:?}]", actual.data);

  // TODO: Replace with ContentType::from(mime) when pact_models 1.1.8 is released
  let expected_content_type = expected.content_type.as_ref().map(|mime| ContentType {
    main_type: mime.type_().to_string(),
    sub_type: mime.subtype().to_string(),
    attributes: mime.params().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
    suffix: mime.suffix().map(|name| name.to_string()),
    .. ContentType::default()
  }). unwrap_or_default();
  let actual_content_type = actual.content_type.as_ref().map(|mime| ContentType {
    main_type: mime.type_().to_string(),
    sub_type: mime.subtype().to_string(),
    attributes: mime.params().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
    suffix: mime.suffix().map(|name| name.to_string()),
    .. ContentType::default()
  }). unwrap_or_default();

  debug!("Comparing mime part '{}': {} -> {}", part_name, expected_content_type, actual_content_type);
  let matcher_result = if expected_content_type.is_unknown() || actual_content_type.is_unknown() ||
      expected_content_type.is_equivalent_to(&actual_content_type) ||
      expected_content_type.is_equivalent_to(&actual_content_type.base_type()) {
    let expected_part = HttpRequest {
      body: OptionalBody::Present(expected.decode_data().unwrap_or_else(|_| expected.data.clone()), Some(expected_content_type.clone()), None),
      .. HttpRequest::default()
    };
    let actual_part = HttpRequest {
      body: OptionalBody::Present(actual.decode_data().unwrap_or_else(|_| actual.data.clone()), Some(actual_content_type.clone()), None),
      .. HttpRequest::default()
    };
    let mut rule_category = context.matchers().clone();
    rule_category.rules = rule_category.rules.iter().filter_map(|(p, rules)| {
      let p_vec = p.to_vec();
      let path_slice = p_vec.iter().map(|p| p.as_str()).collect_vec();
      if path.matches_path(&path_slice) {
        let mut child_path = DocPath::root();
        for path_part in p.tokens().iter().dropping(path.len()) {
          child_path.push(path_part.clone());
        }
        Some((child_path, rules.clone()))
      } else {
        None
      }
    }).collect();
    let context = context.clone_with(&rule_category);
    super::compare_bodies(&expected_content_type, &expected_part, &actual_part, context.as_ref()).await
  } else {
    BodyMatchResult::BodyTypeMismatch {
      expected_type: expected_content_type.to_string(),
      actual_type: actual_content_type.to_string(),
      message: format!("MIME part '{}': Expected a body of '{}' but the actual content type was '{}'",
                       part_name, expected_content_type, actual_content_type),
      expected: Some(expected.data.clone()),
      actual: Some(actual.data.clone())
    }
  };

  debug!("Comparing file part '{:?}' to '{:?}' at path '{}' -> {:?}", expected, actual, path.to_string(), matcher_result);

  let mut results = vec![];
  if let Err(header_mismatches) = header_result {
    results.extend(header_mismatches);
  }
  results.extend(matcher_result.mismatches().iter().map(|m| {
    if let Mismatch::BodyMismatch { path, expected, actual, mismatch } = m {
      Mismatch::BodyMismatch {
        path: path.clone(),
        expected: expected.clone(),
        actual: actual.clone(),
        mismatch: format!("MIME part '{}': {}", part_name, mismatch)
      }
    } else {
      m.clone()
    }
  }));

  if results.is_empty() {
    Ok(())
  } else {
    Err(results)
  }
}

async fn parse_multipart(
  body: Bytes,
  headers: &Option<HashMap<String, Vec<String>>>
) -> anyhow::Result<Vec<MimePart>> {
  let boundary = get_multipart_boundary(headers)?;
  let stream = once(async move { Result::<Bytes, Infallible>::Ok(body) });
  let mut multipart = Multipart::new(stream, boundary);

  let mut parts = vec![];
  while let Some((index, field)) = multipart.next_field_with_idx().await? {
    let name = field.name().map(|s| s.to_string()).unwrap_or_default();
    let content_type = field.content_type().cloned();
    let headers = field.headers().clone();

    if headers.contains_key("Content-Disposition") {
      if let Some(filename) = field.file_name() {
        parts.push(MimePart::File(MimeFile {
          index,
          name,
          content_type,
          filename: filename.to_string(),
          data: field.bytes().await?,
          headers
        }));
      } else {
        parts.push(MimePart::Field(MimeField {
          index,
          name,
          data: field.bytes().await?,
          headers
        }));
      }
    } else {
      parts.push(MimePart::File(MimeFile {
        index,
        name,
        content_type,
        filename: String::default(),
        data: field.bytes().await?,
        headers
      }));
    }
  }

  Ok(parts)
}

fn get_multipart_boundary(headers: &Option<HashMap<String, Vec<String>>>) -> anyhow::Result<String> {
  let header_map = get_http_header_map(headers);
  let content_type = header_map.get(http::header::CONTENT_TYPE)
    .ok_or_else(|| anyhow!("no content-type header"))?
    .to_str()
    .map_err(|e| anyhow!("invalid content-type: {}", e))?;

  let mime: mime::Mime = content_type.parse()
    .map_err(|e| anyhow!("invalid content-type: {}", e))?;

  let boundary = mime.get_param(mime::BOUNDARY)
    .ok_or_else(|| anyhow!("no boundary in content-type"))?;

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
  use http::header::HeaderMap;
  use maplit::*;
  use pact_models::{matchingrules, matchingrules_list};
  use pact_models::bodies::OptionalBody;
  use pact_models::matchingrules::MatchingRule;
  use pact_models::path_exp::DocPath;
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

  #[test_log::test]
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
      "Failed to parse the expected body as a MIME multipart body: \'no content-type header\'",
      "Failed to parse the actual body as a MIME multipart body: \'no content-type header\'"
    ]));
  }

  #[test_log::test]
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

  #[test_log::test]
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

  #[test_log::test(tokio::test(flavor = "multi_thread", worker_threads = 2))]
  async fn match_mime_multipart_different_values() {
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
      "MIME part 'file': Expected body '1,2,3,4\r\n4,5,6,7' to match 'a,b,c,d\r\n4,5,6,7' using equality but did not match"
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
          "$.name" => [ MatchingRule::Regex("^\\w+$".to_string()) ],
          "$.age" => [ MatchingRule::Regex("^\\d+ months?+$".to_string()) ]
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
      "MIME part 'file': header 'content-type': Expected 'text/csv' to be equal to 'text/html'",
      "MIME part 'file': Expected a body of 'text/csv' but the actual content type was 'text/html'"
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

    //<["MIME part 'file': Expected binary contents to have content type 'application/jpeg' but detected contents was 'text/plain'"]>
    //<["Actual body [application/jpeg, 16 bytes, starting with 612c622c632c640d0a342c352c362c37] is not equal to the expected body [application/jpeg, 16 bytes, starting with 312c322c332c340d0a342c352c362c37]"]>
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

  #[test]
  fn ignores_missing_content_type_header_which_is_optional() {
    let expected_body = Bytes::from("--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn returns_a_mismatch_when_the_actual_body_is_empty() {
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
      Content-Disposition: form-data; name=\"name\"\r\n\r\n\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567\r\n\
      Content-Type: text/csv\r\n\
      Content-Disposition: form-data; name=\"file\"; filename=\"009.csv\"\r\n\r\n\
      1,2,3,4\r\n\
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
      "MIME part 'name': Expected 'Baxter' to be equal to ''"
    ]));
  }

  #[test_log::test]
  fn returns_a_mismatch_when_the_headers_dont_match() {
    let expected_body = Bytes::from(
      "--1234\r\n\
      Content-Type: text/plain\r\n\
      X-Test: one\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --1234--\r\n"
    );
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from(
      "--4567\r\n\
      Content-Type: text/plain\r\n\
      X-Test: two\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --4567\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"age\"\r\n\r\n1 month\r\n\
      --4567--\r\n"
    );
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'name': header 'x-test': Expected 'one' to be equal to 'two'"
    ]));
  }

  #[test_log::test]
  fn match_headers_test() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("x-two", "123".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());
    actual.insert("x-two", "123".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_headers_missing_header() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("x-two", "123".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'one': Expected multipart header 'x-two' with value '123' but was missing"
    ]));
  }

  #[test_log::test]
  fn match_headers_ignores_missing_content_type_header() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("Content-Type", "text/plain".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_headers_different_value() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("x-two", "123".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());
    actual.insert("x-two", "456".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    let mismatches = result.unwrap_err();
    expect!(mismatches.iter().map(|m| mismatch(m)).collect::<Vec<&str>>()).to(be_equal_to(vec![
      "MIME part 'one': header 'x-two': Expected '123' to be equal to '456'"
    ]));
  }

  #[test_log::test]
  fn match_headers_with_a_matcher() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::new(DiffConfig::AllowUnexpectedKeys,
      &matchingrules_list! {
        "body"; "$.one['x-two']" => [ MatchingRule::Regex("^[0-9]+$".to_string()) ]
      }, &hashmap!{});

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("x-two", "123".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());
    actual.insert("x-two", "456".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test_log::test]
  fn match_headers_ignores_content_disposition() {
    let path = DocPath::new_unwrap("$.one");
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let mut expected = HeaderMap::new();
    expected.insert("x-one", "example.com".parse().unwrap());
    expected.insert("Content-Disposition", "form-data; name=\"file\"; filename=\"009.csv\"".parse().unwrap());

    let mut actual = HeaderMap::new();
    actual.insert("x-one", "example.com".parse().unwrap());
    actual.insert("content-disposition", "form-data; name=\"file\"; filename=\"008.csv\"".parse().unwrap());

    let result = super::match_headers(&path, &expected, &actual, &context);
    expect!(result).to(be_ok());
  }

  #[test]
  fn supports_content_transfer_encoding_header() {
    let expected_body = Bytes::from("--1234\r\n\
      Content-Type: text/plain\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nBaxter\r\n\
      --1234--\r\n");
    let expected = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=1234".into() ] }),
      body: OptionalBody::Present(expected_body, None, None),
      ..Request::default()
    };
    let actual_body = Bytes::from("--4567\r\n\
      Content-Type: text/plain\r\n\
      content-transfer-encoding: BASE64\r\n\
      Content-Disposition: form-data; name=\"name\"\r\n\r\nQmF4dGVy\r\n\
      --4567--\r\n");
    let actual = Request {
      headers: Some(hashmap!{ "Content-Type".into() => vec![ "multipart/form-data; boundary=4567".into() ] }),
      body: OptionalBody::Present(actual_body, None, None),
      ..Request::default()
    };
    let context = CoreMatchingContext::with_config(DiffConfig::AllowUnexpectedKeys);

    let result = match_mime_multipart(&expected, &actual, &context);
    expect!(result).to(be_ok());
  }
}
