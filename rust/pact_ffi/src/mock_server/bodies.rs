//! Functions to support processing request/response bodies

use std::path::Path;

use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use lazy_static::lazy_static;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentTypeHint;
use pact_models::generators::{Generator, GeneratorCategory, Generators};
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::{Category, MatchingRule, MatchingRuleCategory, RuleLogic};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use regex::Regex;
use serde_json::{Map, Value};
use tracing::{debug, error, trace, warn};

const CONTENT_TYPE_HEADER: &str = "Content-Type";

lazy_static! {
  static ref MULTIPART_MARKER: Regex = Regex::new("\\-\\-([a-zA-Z0-9'\\(\\)+_,-.\\/:=? ]*)\r\n").unwrap();
}

/// Process an array with embedded matching rules and generators
pub fn process_array(
  array: &[Value],
  matching_rules: &mut MatchingRuleCategory,
  generators: &mut Generators,
  path: DocPath,
  type_matcher: bool,
  skip_matchers: bool
) -> Value {
  trace!(">>> process_array(array={array:?}, matching_rules={matching_rules:?}, generators={generators:?}, path={path}, type_matcher={type_matcher}, skip_matchers={skip_matchers})");
  debug!("Path = {path}");
  Value::Array(array.iter().enumerate().map(|(index, val)| {
    let mut item_path = path.clone();
    if type_matcher {
      item_path.push_star_index();
    } else {
      item_path.push_index(index);
    }
    match val {
      Value::Object(ref map) => process_object(map, matching_rules, generators, item_path, skip_matchers),
      Value::Array(ref array) => process_array(array, matching_rules, generators, item_path, false, skip_matchers),
      _ => val.clone()
    }
  }).collect())
}

/// Process an object (map) with embedded matching rules and generators
pub fn process_object(
  obj: &Map<String, Value>,
  matching_rules: &mut MatchingRuleCategory,
  generators: &mut Generators,
  path: DocPath,
  skip_matchers: bool
) -> Value {
  trace!(">>> process_object(obj={obj:?}, matching_rules={matching_rules:?}, generators={generators:?}, path={path}, skip_matchers={skip_matchers})");
  debug!("Path = {path}");
  let result = if let Some(matcher_type) = obj.get("pact:matcher:type") {
    debug!("detected pact:matcher:type, will configure a matcher");
    if !skip_matchers {
      let matcher_type = json_to_string(matcher_type);
      let matching_rule = match matcher_type.as_str() {
        "arrayContains" | "array-contains" => match obj.get("variants") {
          Some(Value::Array(variants)) => {
            let values = variants.iter().enumerate().map(|(index, variant)| {
              let mut category = MatchingRuleCategory::empty("body");
              let mut generators = Generators::default();
              match variant {
                Value::Object(map) => {
                  process_object(map, &mut category, &mut generators, DocPath::root(), false);
                }
                _ => warn!("arrayContains: JSON for variant {} is not correctly formed: {}", index, variant)
              }
              (index, category, generators.categories.get(&GeneratorCategory::BODY).cloned().unwrap_or_default())
            }).collect();
            Ok(MatchingRule::ArrayContains(values))
          }
          _ => Err(anyhow!("ArrayContains 'variants' attribute is missing or not an array"))
        },
        _ => {
          let attributes = Value::Object(obj.clone());
          MatchingRule::create(matcher_type.as_str(), &attributes)
        }
      };

      trace!("matching_rule = {matching_rule:?}");
      match &matching_rule {
        Ok(rule) => matching_rules.add_rule(path.clone(), rule.clone(), RuleLogic::And),
        Err(err) => error!("Failed to parse matching rule from JSON - {}", err)
      };
      if let Some(gen) = obj.get("pact:generator:type") {
        debug!("detected pact:generator:type, will configure a generators");
        if let Some(generator) = Generator::from_map(&json_to_string(gen), obj) {
          let category = match matching_rules.name {
            Category::BODY => &GeneratorCategory::BODY,
            Category::HEADER => &GeneratorCategory::HEADER,
            Category::PATH => &GeneratorCategory::PATH,
            Category::QUERY => &GeneratorCategory::QUERY,
            _ => {
              warn!("invalid generator category {} provided, defaulting to body", matching_rules.name);
              &GeneratorCategory::BODY
            }
          };
          generators.add_generator_with_subcategory(category, path.clone(), generator);
        }
      }
      let (value, skip_matchers) = if let Ok(rule) = &matching_rule {
        match rule {
          MatchingRule::ArrayContains(_) => (obj.get("variants"), true),
          _ => (obj.get("value"), false)
        }
      } else {
        (obj.get("value"), false)
      };
      match value {
        Some(val) => match val {
          Value::Object(ref map) => process_object(map, matching_rules, generators, path, skip_matchers),
          Value::Array(array) => process_array(array, matching_rules, generators, path, true, skip_matchers),
          _ => val.clone()
        },
        None => Value::Null
      }
    } else {
      debug!("Skipping the matching rule (skip_matchers == true)");
      match obj.get("value") {
        Some(val) => match val {
          Value::Object(ref map) => process_object(map, matching_rules, generators, path, skip_matchers),
          Value::Array(array) => process_array(array, matching_rules, generators, path, false, skip_matchers),
          _ => val.clone()
        },
        None => Value::Null
      }
    }
  } else {
    debug!("Configuring a normal object");
    Value::Object(obj.iter()
      .filter(|(key, _)| !key.starts_with("pact:"))
      .map(|(key, val)| {
        let path_vec = path.to_vec();
        let path_slice = path_vec.iter().map(|p| p.as_str()).collect::<Vec<_>>();
        let matchers_for_path = matching_rules.resolve_matchers_for_path(path_slice.as_slice());
        let item_path = if matchers_for_path.values_matcher_defined() {
          path.join("*")
        } else {
          path.join(key)
        };
        (key.clone(), match val {
          Value::Object(ref map) => process_object(map, matching_rules, generators, item_path, skip_matchers),
          Value::Array(ref array) => process_array(array, matching_rules, generators, item_path, false, skip_matchers),
          _ => val.clone()
        })
    }).collect())
  };
  trace!("-> result = {result:?}");
  result
}

/// Builds a `MatchingRule` from a `Value` struct used by language integrations
#[deprecated(note = "Replace with MatchingRule::create")]
pub fn matcher_from_integration_json(m: &Map<String, Value>) -> Option<MatchingRule> {
  match m.get("pact:matcher:type") {
    Some(value) => {
      let val = json_to_string(value);
      MatchingRule::create(val.as_str(), &Value::Object(m.clone()))
        .map_err(|err| error!("Failed to create matching rule from JSON '{:?}': {}", m, err))
        .ok()
    },
    _ => None
  }
}

/// Process a JSON body with embedded matching rules and generators
pub fn process_json(body: String, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> String {
  trace!("process_json");
  match serde_json::from_str(&body) {
    Ok(json) => match json {
      Value::Object(ref map) => process_object(map, matching_rules, generators, DocPath::root(), false).to_string(),
      Value::Array(ref array) => process_array(array, matching_rules, generators, DocPath::root(), false, false).to_string(),
      _ => body
    },
    Err(_) => body
  }
}

/// Process a JSON body with embedded matching rules and generators
pub fn process_json_value(body: &Value, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> String {
  match body {
    Value::Object(ref map) => process_object(map, matching_rules, generators, DocPath::root(), false).to_string(),
    Value::Array(ref array) => process_array(array, matching_rules, generators, DocPath::root(), false, false).to_string(),
    _ => body.to_string()
  }
}

/// Setup the request as a multipart form upload
pub fn request_multipart(
  request: &mut HttpRequest,
  boundary: &str,
  body: OptionalBody,
  content_type: &str,
  part_name: &str
) {
  if let Some(parts) = add_part_to_multipart(&request.body, &body, boundary) {
    // Exiting part with the same boundary marker found, just add the new part to the end
    // This assumes that the previous call will have correctly setup headers and matching rules etc.
    debug!("Found existing multipart with the same boundary marker, will append to it");
    request.body = OptionalBody::Present(parts, request.body.content_type(), get_content_type_hint(&request.body));
  } else {
    // Either no existing multipart exists, or there is one with a different marker, so we
    // overwrite it.
    let multipart = format!("multipart/form-data; boundary={}", boundary);
    request.set_header(CONTENT_TYPE_HEADER, &[multipart.as_str()]);
    request.body = body;

    request.matching_rules.add_category("header")
      .add_rule(DocPath::new_unwrap("Content-Type"),
                MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), RuleLogic::And);
  }

  let mut path = DocPath::root();
  path.push_field(part_name);
  request.matching_rules.add_category("body")
    .add_rule(path, MatchingRule::ContentType(content_type.into()), RuleLogic::And);
}

fn add_part_to_multipart(body: &OptionalBody, new_part: &OptionalBody, boundary: &str) -> Option<Bytes> {
  if let Some(boundary_marker) = contains_existing_multipart(body) {
    let existing_parts = body.value().unwrap_or_default();
    let end_marker = format!("--{}--\r\n", boundary_marker);
    let base = existing_parts.strip_suffix(end_marker.as_bytes()).unwrap_or(&existing_parts);
    let new_part = part_body_replace_marker(new_part, boundary, &boundary_marker.as_str());

    let mut bytes = BytesMut::from(base);
    bytes.extend(new_part);
    Some(bytes.freeze())
  } else {
    None
  }
}

/// Replace multipart marker in body
pub fn part_body_replace_marker(body: &OptionalBody, boundary: &str, new_boundary: &str) -> Bytes {
  let marker = format!("--{}\r\n", new_boundary);
  let end_marker = format!("--{}--\r\n", new_boundary);

  let marker_to_replace = format!("--{}\r\n", boundary);
  let end_marker_to_replace = format!("--{}--\r\n", boundary);
  let body = body.value().unwrap_or_default();
  let body = body.strip_prefix(marker_to_replace.as_bytes()).unwrap_or(&body);
  let body = body.strip_suffix(end_marker_to_replace.as_bytes()).unwrap_or(&body);

  let mut bytes = BytesMut::new();
  bytes.extend(marker.as_bytes());
  bytes.extend(body);
  bytes.extend(end_marker.as_bytes());
  bytes.freeze()
}

/// Get content type hint from body
pub fn get_content_type_hint(body: &OptionalBody) -> Option<ContentTypeHint> {
  match &body {
    OptionalBody::Present(_, _, hint) => *hint,
    _ => None
  }
}

fn contains_existing_multipart(body: &OptionalBody) -> Option<String> {
  if let OptionalBody::Present(body, ..) = &body {
    let body_str = String::from_utf8_lossy(&body);
    if let Some(captures) = MULTIPART_MARKER.captures(&body_str) {
      captures.get(1).map(|marker| marker.as_str().to_string())
    } else {
      None
    }
  } else {
    None
  }
}

/// Setup the response as a multipart form upload
pub fn response_multipart(
  response: &mut HttpResponse,
  boundary: &str,
  body: OptionalBody,
  content_type: &str,
  part_name: &str
) {
  if let Some(parts) = add_part_to_multipart(&response.body, &body, boundary) {
    // Exiting part with the same boundary marker found, just add the new part to the end
    // This assumes that the previous call will have correctly setup headers and matching rules etc.
    debug!("Found existing multipart with the same boundary marker, will append to it");
    response.body = OptionalBody::Present(parts, response.body.content_type(), get_content_type_hint(&response.body));
  } else {
    // Either no existing multipart exists, or there is one with a different marker, so we
    // overwrite it.
    let multipart = format!("multipart/form-data; boundary={}", boundary);
    response.set_header(CONTENT_TYPE_HEADER, &[multipart.as_str()]);
    response.body = body;

    response.matching_rules.add_category("header")
      .add_rule(DocPath::new_unwrap("Content-Type"),
                MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), RuleLogic::And);
  }

  let mut path = DocPath::root();
  path.push_field(part_name);
  response.matching_rules.add_category("body")
    .add_rule(path, MatchingRule::ContentType(content_type.into()), RuleLogic::And);
}

/// Representation of a multipart body
#[derive(Clone, Debug)]
pub struct MultipartBody {
  /// The actual body
  pub body: OptionalBody,

  /// The boundary used in the multipart encoding
  pub boundary: String,
}

/// Loads an example file as a MIME Multipart body
pub fn file_as_multipart_body(file: &str, part_name: &str) -> Result<MultipartBody, String> {
  let mut multipart = multipart::client::Multipart::from_request(multipart::mock::ClientRequest::default()).unwrap();

  multipart.write_file(part_name, Path::new(file)).map_err(format_multipart_error)?;
  let http_buffer = multipart.send().map_err(format_multipart_error)?;

  Ok(MultipartBody {
    body: OptionalBody::Present(Bytes::from(http_buffer.buf), Some("multipart/form-data".into()), None),
    boundary: http_buffer.boundary
  })
}

/// Create an empty MIME Multipart body
pub fn empty_multipart_body() -> Result<MultipartBody, String> {
  let multipart = multipart::client::Multipart::from_request(multipart::mock::ClientRequest::default()).unwrap();
  let http_buffer = multipart.send().map_err(format_multipart_error)?;

  Ok(MultipartBody {
    body: OptionalBody::Present(Bytes::from(http_buffer.buf), Some("multipart/form-data".into()), None),
    boundary: http_buffer.boundary
  })
}

fn format_multipart_error(e: std::io::Error) -> String {
  format!("convert_ptr_to_mime_part_body: Failed to generate multipart body: {}", e)
}

#[cfg(test)]
mod test {
  use expectest::prelude::*;
  use maplit::hashmap;
  use pact_models::{generators, HttpStatus, matchingrules_list};
  use pact_models::content_types::ContentType;
  use pact_models::generators::{Generator, Generators};
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};
  use pact_models::matchingrules::expressions::{MatchingRuleDefinition, ValueType};
  use pact_models::path_exp::DocPath;
  use serde_json::json;
  use pretty_assertions::assert_eq;

  #[allow(deprecated)]
  use crate::mock_server::bodies::{matcher_from_integration_json, process_object};
  use super::*;

  #[test]
  fn process_object_with_normal_json_test() {
    let json = json!({
      "a": "b",
      "c": [100, 200, 300]
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json));
  }

  #[test]
  fn process_object_with_matching_rule_test() {
    let json = json!({
      "a": {
        "pact:matcher:type": "regex",
        "regex": "\\w+",
        "value": "b"
      },
      "c": [100, 200, {
        "pact:matcher:type": "integer",
        "pact:generator:type": "RandomInt",
        "value": 300
      }]
    });
    let mut matching_rules = MatchingRuleCategory::empty("body");
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!({
      "a": "b",
      "c": [100, 200, 300]
    })));
    expect!(matching_rules).to(be_equal_to(matchingrules_list!{
      "body";
      "$.a" => [ MatchingRule::Regex("\\w+".into()) ],
      "$.c[2]" => [ MatchingRule::Integer ]
    }));
    expect!(generators).to(be_equal_to(generators! {
      "BODY" => {
        "$.c[2]" => Generator::RandomInt(0, 10)
      }
    }));
  }

  #[test]
  fn process_object_with_primitive_json_value() {
    let json = json!({
      "pact:matcher:type": "regex",
      "regex": "\\w+",
      "value": "b"
    });
    let mut matching_rules = MatchingRuleCategory::empty("body");
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!("b")));
    expect!(matching_rules).to(be_equal_to(matchingrules_list!{
      "body";
      "$" => [ MatchingRule::Regex("\\w+".into()) ]
    }));
    expect!(generators).to(be_equal_to(Generators::default()));
  }

  // Issue #179
  #[test_log::test]
  fn process_object_with_nested_object_has_the_same_property_name_as_a_parent_object() {
    let json = json!({
      "result": {
        "pact:matcher:type": "type",
        "value": {
          "details": {
            "pact:matcher:type": "type",
            "value": [
              {
                "type": {
                  "pact:matcher:type": "regex",
                  "value": "Information",
                  "regex": "(None|Information|Warning|Error)"
                }
              }
            ],
            "min": 1
          },
          "findings": {
            "pact:matcher:type": "type",
            "value": [
              {
                "details": {
                  "pact:matcher:type": "type",
                  "value": [
                    {
                      "type": {
                        "pact:matcher:type": "regex",
                        "value": "Information",
                        "regex": "(None|Information|Warning|Error)"
                      }
                    }
                  ],
                  "min": 1
                },
                "type": {
                  "pact:matcher:type": "regex",
                  "value": "Unspecified",
                  "regex": "(None|Unspecified)"
                }
              }
            ],
            "min": 1
          }
        }
      }
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!({
      "result": {
        "details": [
          {
            "type": "Information"
          }
        ],
        "findings": [
          {
            "details": [
              {
                "type": "Information"
              }
            ],
            "type": "Unspecified"
          }
        ]
      }
    })));
    expect!(matching_rules.to_v3_json().to_string()).to(be_equal_to(matchingrules_list!{
      "body";
      "$.result" => [ MatchingRule::Type ],
      "$.result.details" => [ MatchingRule::MinType(1) ],
      "$.result.details[*].type" => [ MatchingRule::Regex("(None|Information|Warning|Error)".into()) ],
      "$.result.findings" => [ MatchingRule::MinType(1) ],
      "$.result.findings[*].details" => [ MatchingRule::MinType(1) ],
      "$.result.findings[*].details[*].type" => [ MatchingRule::Regex("(None|Information|Warning|Error)".into()) ],
      "$.result.findings[*].type" => [ MatchingRule::Regex("(None|Unspecified)".into()) ]
    }.to_v3_json().to_string()));
    expect!(generators).to(be_equal_to(Generators::default()));
  }

  // Issue #179
  #[test_log::test]
  fn process_object_with_nested_object_with_type_matchers_and_decimal_matcher() {
    let json = json!({
      "pact:matcher:type": "type",
      "value": {
        "name": {
          "pact:matcher:type": "type",
          "value": "APL"
        },
        "price": {
          "pact:matcher:type": "decimal",
          "value": 1.23
        }
      }
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!({
      "name": "APL",
      "price": 1.23
    })));
    expect!(matching_rules).to(be_equal_to(matchingrules_list!{
      "body";
      "$" => [ MatchingRule::Type ],
      "$.name" => [ MatchingRule::Type ],
      "$.price" => [ MatchingRule::Decimal ]
    }));
    expect!(generators).to(be_equal_to(Generators::default()));
  }

  // Issue #299
  #[test_log::test]
  fn process_object_with_each_value_matcher_on_object() {
    let json = json!({
      "pact:matcher:type": "each-value",
      "value": {
        "price": 1.23
      },
      "rules": [
        {
          "pact:matcher:type": "decimal"
        }
      ]
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
      &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!({
      "price": 1.23
    })));
    expect!(matching_rules).to(be_equal_to(matchingrules_list!{
      "body";
      "$" => [ MatchingRule::EachValue(MatchingRuleDefinition::new("{\"price\":1.23}".to_string(),
        ValueType::Unknown, MatchingRule::Decimal, None)) ]
    }));
    expect!(generators).to(be_equal_to(Generators::default()));
  }

  // Issue #299
  #[test_log::test]
  fn process_object_with_each_key_matcher_on_object() {
    let json = json!({
      "pact:matcher:type": "each-key",
      "value": {
        "123": "cool book"
      },
      "rules": [
        {
          "pact:matcher:type": "regex",
          "regex": "\\d+"
        }
      ]
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
      &mut generators, DocPath::root(), false);

    expect!(result).to(be_equal_to(json!({
      "123": "cool book"
    })));
    expect!(matching_rules).to(be_equal_to(matchingrules_list!{
      "body";
      "$" => [ MatchingRule::EachKey(MatchingRuleDefinition::new("{\"123\":\"cool book\"}".to_string(),
        ValueType::Unknown, MatchingRule::Regex("\\d+".to_string()), None)) ]
    }));
    expect!(generators).to(be_equal_to(Generators::default()));
  }

  #[test_log::test]
  #[allow(deprecated)]
  fn matcher_from_integration_json_test() {
    expect!(matcher_from_integration_json(&Map::default())).to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "Other" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "regex" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "regex", "regex": "[a-z]" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Regex("[a-z]".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "equality" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Equality));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "include" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "include", "value": "[a-z]" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Include("[a-z]".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "type" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Type));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "type", "min": 100 }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::MinType(100)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "type", "max": 100 }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::MaxType(100)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "type", "min": 10, "max": 100 }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::MinMaxType(10, 100)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "number" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Number));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "integer" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Integer));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "decimal" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Decimal));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "real" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Decimal));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "min" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "min", "min": 100 }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::MinType(100)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "max" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "max", "max": 100 }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::MaxType(100)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "timestamp" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "timestamp", "format": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "timestamp", "timestamp": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "datetime" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "datetime", "format": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "datetime", "datetime": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Timestamp("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "date" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Date("".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "date", "format": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Date("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "date", "date": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Date("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "time" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Time("".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "time", "format": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Time("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "time", "time": "yyyy-MM-dd" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Time("yyyy-MM-dd".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "null" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Null));

    // V4 matching rules
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "boolean" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Boolean));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "contentType" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "contentType", "value": "text/plain" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::ContentType("text/plain".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "content-type" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "content-type", "value": "text/plain" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::ContentType("text/plain".to_string())));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "arrayContains" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "arrayContains", "variants": "text" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "arrayContains", "variants": [] }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::ArrayContains(vec![])));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "array-contains" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "array-contains", "variants": "text" }).as_object().unwrap()))
      .to(be_none());
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "array-contains", "variants": [] }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::ArrayContains(vec![])));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "values" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Values));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "statusCode" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::StatusCode(HttpStatus::Success)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "statusCode", "status": [200] }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::StatusCode(HttpStatus::StatusCodes(vec![200]))));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "status-code" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::StatusCode(HttpStatus::Success)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "status-code", "status": "success" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::StatusCode(HttpStatus::Success)));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "notEmpty" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::NotEmpty));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "not-empty" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::NotEmpty));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "semver" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::Semver));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "eachKey" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::EachKey(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![],
        generator: None,
      })));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "each-key" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::EachKey(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![],
        generator: None,
      })));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "eachValue" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::EachValue(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![],
        generator: None,
      })));
    expect!(matcher_from_integration_json(&json!({ "pact:matcher:type": "each-value" }).as_object().unwrap()))
      .to(be_some().value(MatchingRule::EachValue(MatchingRuleDefinition {
        value: "".to_string(),
        value_type: ValueType::Unknown,
        rules: vec![],
        generator: None,
      })));
  }

  #[test_log::test]
  fn request_multipart_test() {
    let mut request = HttpRequest::default();
    let body = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\nContent-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n");
    let ct = ContentType::parse("application/json").unwrap();

    request_multipart(&mut request, "ABCD", OptionalBody::Present(body, Some(ct.clone()), None), &ct.to_string(), "part-1");

    expect!(request.headers.unwrap()).to(be_equal_to(hashmap!{
      "Content-Type".to_string() => vec!["multipart/form-data; boundary=ABCD".to_string()]
    }));
    assert_eq!("--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\n\
Content-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n",
               request.body.value_as_string().unwrap());
  }

  // Issue #314
  #[test_log::test]
  fn request_multipart_allows_multiple_parts() {
    let mut request = HttpRequest::default();
    let body1 = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\nContent-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n");
    let ct1 = ContentType::parse("application/json").unwrap();
    let body2 = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-2\"; filename=\"2.txt\"\r\nContent-Type: text/plain\r\n\r\nTEXT\r\n--ABCD--\r\n");
    let ct2 = ContentType::parse("text/plain").unwrap();

    request_multipart(&mut request, "ABCD", OptionalBody::Present(body1, Some(ct1.clone()), None), &ct1.to_string(), "part-1");
    request_multipart(&mut request, "ABCD", OptionalBody::Present(body2, Some(ct2.clone()), None), &ct2.to_string(), "part-2");

    expect!(request.headers.unwrap()).to(be_equal_to(hashmap!{
      "Content-Type".to_string() => vec!["multipart/form-data; boundary=ABCD".to_string()]
    }));
    assert_eq!("--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\n\
Content-Type: application/json\r\n\r\n{}\r\n--ABCD\r\nContent-Disposition: form-data; \
name=\"part-2\"; filename=\"2.txt\"\r\nContent-Type: text/plain\r\n\r\nTEXT\r\n--ABCD--\r\n",
               request.body.value_as_string().unwrap());
  }

  #[test_log::test]
  fn response_multipart_test() {
    let mut response = HttpResponse::default();
    let body = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\nContent-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n");
    let ct = ContentType::parse("application/json").unwrap();

    response_multipart(&mut response, "ABCD", OptionalBody::Present(body, Some(ct.clone()), None), &ct.to_string(), "part-1");

    expect!(response.headers.unwrap()).to(be_equal_to(hashmap!{
      "Content-Type".to_string() => vec!["multipart/form-data; boundary=ABCD".to_string()]
    }));
    assert_eq!("--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\n\
Content-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n",
               response.body.value_as_string().unwrap());
  }

  // Issue #314
  #[test_log::test]
  fn response_multipart_allows_multiple_parts() {
    let mut response = HttpResponse::default();
    let body1 = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\nContent-Type: application/json\r\n\r\n{}\r\n--ABCD--\r\n");
    let ct1 = ContentType::parse("application/json").unwrap();
    let body2 = Bytes::from_static(b"--ABCD\r\nContent-Disposition: form-data; name=\"part-2\"; filename=\"2.txt\"\r\nContent-Type: text/plain\r\n\r\nTEXT\r\n--ABCD--\r\n");
    let ct2 = ContentType::parse("text/plain").unwrap();

    response_multipart(&mut response, "ABCD", OptionalBody::Present(body1, Some(ct1.clone()), None), &ct1.to_string(), "part-1");
    response_multipart(&mut response, "ABCD", OptionalBody::Present(body2, Some(ct2.clone()), None), &ct2.to_string(), "part-2");

    expect!(response.headers.unwrap()).to(be_equal_to(hashmap!{
      "Content-Type".to_string() => vec!["multipart/form-data; boundary=ABCD".to_string()]
    }));
    assert_eq!("--ABCD\r\nContent-Disposition: form-data; name=\"part-1\"; filename=\"1.json\"\r\n\
Content-Type: application/json\r\n\r\n{}\r\n--ABCD\r\nContent-Disposition: form-data; \
name=\"part-2\"; filename=\"2.txt\"\r\nContent-Type: text/plain\r\n\r\nTEXT\r\n--ABCD--\r\n",
               response.body.value_as_string().unwrap());
  }
}
