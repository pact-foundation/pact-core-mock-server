//! Functions to support processing request/response bodies

use std::path::Path;

use bytes::Bytes;
use log::*;
use maplit::*;
use serde_json::{Map, Value};

use pact_models::bodies::OptionalBody;
use pact_models::generators::{Generator, GeneratorCategory, Generators};
use pact_models::json_utils::{json_to_num, json_to_string};
use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory, RuleLogic, Category};
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};

const CONTENT_TYPE_HEADER: &str = "Content-Type";

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
      Value::Object(ref map) => process_object(map, matching_rules, generators, item_path, false, skip_matchers),
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
  type_matcher: bool,
  skip_matchers: bool
) -> Value {
  trace!(">>> process_object(obj={obj:?}, matching_rules={matching_rules:?}, generators={generators:?}, path={path}, type_matcher={type_matcher}, skip_matchers={skip_matchers})");
  debug!("Path = {path}");
  let result = if obj.contains_key("pact:matcher:type") {
    debug!("detected pact:matcher:type, will configure a matcher");
    if !skip_matchers {
      let matching_rule = matcher_from_integration_json(obj);
      trace!("matching_rule = {matching_rule:?}");
      if let Some(rule) = &matching_rule {
        matching_rules.add_rule(path.clone(), rule.clone(), RuleLogic::And);
      }
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
      let (value, skip_matchers) = if let Some(rule) = matching_rule {
        match rule {
          MatchingRule::ArrayContains(_) => (obj.get("variants"), true),
          _ => (obj.get("value"), false)
        }
      } else {
        (obj.get("value"), false)
      };
      match value {
        Some(val) => match val {
          Value::Object(ref map) => process_object(map, matching_rules, generators, path, true, skip_matchers),
          Value::Array(array) => process_array(array, matching_rules, generators, path, true, skip_matchers),
          _ => val.clone()
        },
        None => Value::Null
      }
    } else {
      debug!("Skipping the matching rule (skip_matchers == true)");
      match obj.get("value") {
        Some(val) => match val {
          Value::Object(ref map) => process_object(map, matching_rules, generators, path, false, skip_matchers),
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
      let item_path = path.join(key);
      (key.clone(), match val {
        Value::Object(ref map) => process_object(map, matching_rules, generators, item_path, false, skip_matchers),
        Value::Array(ref array) => process_array(array, matching_rules, generators, item_path, false, skip_matchers),
        _ => val.clone()
      })
    }).collect())
  };
  trace!("-> result = {result:?}");
  result
}

/// Builds a `MatchingRule` from a `Value` struct used by language integrations
pub fn matcher_from_integration_json(m: &Map<String, Value>) -> Option<MatchingRule> {
  match m.get("pact:matcher:type") {
    Some(value) => {
      let val = json_to_string(value);
      match val.as_str() {
        "regex" => m.get(&val).map(|s| MatchingRule::Regex(json_to_string(s))),
        "equality" => Some(MatchingRule::Equality),
        "include" => m.get("value").map(|s| MatchingRule::Include(json_to_string(s))),
        "type" => match (json_to_num(m.get("min").cloned()), json_to_num(m.get("max").cloned())) {
          (Some(min), Some(max)) => Some(MatchingRule::MinMaxType(min, max)),
          (Some(min), None) => Some(MatchingRule::MinType(min)),
          (None, Some(max)) => Some(MatchingRule::MaxType(max)),
          _ => Some(MatchingRule::Type)
        },
        "number" => Some(MatchingRule::Number),
        "integer" => Some(MatchingRule::Integer),
        "decimal" => Some(MatchingRule::Decimal),
        "real" => Some(MatchingRule::Decimal),
        "min" => json_to_num(m.get(&val).cloned()).map(MatchingRule::MinType),
        "max" => json_to_num(m.get(&val).cloned()).map(MatchingRule::MaxType),
        "timestamp" => m.get("format").or_else(|| m.get(&val)).map(|s| MatchingRule::Timestamp(json_to_string(s))),
        "date" => m.get("format").or_else(|| m.get(&val)).map(|s| MatchingRule::Date(json_to_string(s))),
        "time" => m.get("format").or_else(|| m.get(&val)).map(|s| MatchingRule::Time(json_to_string(s))),
        "null" => Some(MatchingRule::Null),
        "values" => Some(MatchingRule::Values),
        "contentType" => m.get("value").map(|s| MatchingRule::ContentType(json_to_string(s))),
        "arrayContains" => match m.get("variants") {
          Some(Value::Array(variants)) => {
            let values = variants.iter().enumerate().map(|(index, variant)| {
              let mut category = MatchingRuleCategory::empty("body");
              let mut generators = Generators::default();
              match variant {
                Value::Object(map) => {
                  process_object(map, &mut category, &mut generators, DocPath::root(), false, false);
                }
                _ => warn!("arrayContains: JSON for variant {} is not correctly formed: {}", index, variant)
              }
              (index, category, generators.categories.get(&GeneratorCategory::BODY).cloned().unwrap_or_default())
            }).collect();
            Some(MatchingRule::ArrayContains(values))
          }
          _ => None
        }
        _ => None
      }
    },
    _ => None
  }
}

/// Process a JSON body with embedded matching rules and generators
pub fn process_json(body: String, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> String {
  trace!("process_json");
  match serde_json::from_str(&body) {
    Ok(json) => match json {
      Value::Object(ref map) => process_object(map, matching_rules, generators, DocPath::root(), false, false).to_string(),
      Value::Array(ref array) => process_array(array, matching_rules, generators, DocPath::root(), false, false).to_string(),
      _ => body
    },
    Err(_) => body
  }
}

/// Process a JSON body with embedded matching rules and generators
pub fn process_json_value(body: &Value, matching_rules: &mut MatchingRuleCategory, generators: &mut Generators) -> String {
  match body {
    Value::Object(ref map) => process_object(map, matching_rules, generators, DocPath::root(), false, false).to_string(),
    Value::Array(ref array) => process_array(array, matching_rules, generators, DocPath::root(), false, false).to_string(),
    _ => body.to_string()
  }
}

/// Setup the request as a multipart form upload
pub fn request_multipart(request: &mut HttpRequest, boundary: &str, body: OptionalBody, content_type: &str, part_name: &str) {
  request.body = body;
  match request.headers {
    Some(ref mut headers) => {
      headers.insert(CONTENT_TYPE_HEADER.to_string(), vec![format!("multipart/form-data; boundary={}", boundary)]);
    },
    None => {
      request.headers = Some(hashmap! {
        CONTENT_TYPE_HEADER.to_string() => vec![format!("multipart/form-data; boundary={}", boundary)]
      });
    }
  };
  let mut path = DocPath::root();
  path.push_field(part_name);
  request.matching_rules.add_category("body")
    .add_rule(path, MatchingRule::ContentType(content_type.into()), RuleLogic::And);
  request.matching_rules.add_category("header")
    .add_rule(DocPath::new_unwrap("Content-Type"),
              MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), RuleLogic::And);
}

/// Setup the response as a multipart form upload
pub fn response_multipart(response: &mut HttpResponse, boundary: &str, body: OptionalBody, content_type: &str, part_name: &str) {
  response.body = body;
  match response.headers {
    Some(ref mut headers) => {
      headers.insert(CONTENT_TYPE_HEADER.to_string(), vec![format!("multipart/form-data; boundary={}", boundary)]);
    },
    None => {
      response.headers = Some(hashmap! {
        CONTENT_TYPE_HEADER.to_string() => vec![format!("multipart/form-data; boundary={}", boundary)]
      });
    }
  }
  let mut path = DocPath::root();
  path.push_field(part_name);
  response.matching_rules.add_category("body")
    .add_rule(path, MatchingRule::ContentType(content_type.into()), RuleLogic::And);
  response.matching_rules.add_category("header")
    .add_rule(DocPath::new_unwrap("Content-Type"),
              MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), RuleLogic::And);
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
  use expectest::prelude::{be_equal_to, expect};
  use serde_json::json;

  use pact_models::{generators, matchingrules_list};
  use pact_models::generators::{Generator, Generators};
  use pact_models::matchingrules::{MatchingRule, MatchingRuleCategory};
  use pact_models::path_exp::DocPath;

  use crate::mock_server::bodies::process_object;

  #[test]
  fn process_object_with_normal_json_test() {
    let json = json!({
      "a": "b",
      "c": [100, 200, 300]
    });
    let mut matching_rules = MatchingRuleCategory::default();
    let mut generators = Generators::default();
    let result = process_object(json.as_object().unwrap(), &mut matching_rules,
                                &mut generators, DocPath::root(), false, false);

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
                                &mut generators, DocPath::root(), false, false);

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
                                &mut generators, DocPath::root(), false, false);

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
                                &mut generators, DocPath::root(), false, false);

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
                                &mut generators, DocPath::root(), false, false);

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
}
