//! Functions to support processing request/response bodies

use pact_matching::models::matchingrules::{Category, MatchingRule, RuleLogic};
use pact_matching::models::generators::{Generators, Generator, GeneratorCategory};
use serde_json::{Value, Map};
use pact_matching::models::json_utils::json_to_string;

fn process_array(array: &[Value], matching_rules: &mut Category, generators: &mut Generators, path: &str, type_matcher: bool) -> Value {
  Value::Array(array.iter().enumerate().map(|(index, val)| {
    let updated_path = if type_matcher {
      path.to_owned() + "[*]"
    } else {
      path.to_owned() + "[" + &index.to_string() + "]"
    };
    match val {
      Value::Object(ref map) => process_object(map, matching_rules, generators, &updated_path, false),
      Value::Array(ref array) => process_array(array, matching_rules, generators, &updated_path, false),
      _ => val.clone()
    }
  }).collect())
}

fn process_object(obj: &Map<String, Value>, matching_rules: &mut Category, generators: &mut Generators, path: &str, type_matcher: bool) -> Value {
  if obj.contains_key("pact:matcher:type") {
    if let Some(rule) = MatchingRule::from_integration_json(obj) {
      matching_rules.add_rule(&path.to_string(), rule, &RuleLogic::And);
    }
    if let Some(gen) = obj.get("pact:generator:type") {
      if let Some(generator) = Generator::from_map(&json_to_string(gen), obj) {
        generators.add_generator_with_subcategory(&GeneratorCategory::BODY, path, generator);
      }
    }
    match obj.get("value") {
      Some(val) => match val {
        Value::Object(ref map) => process_object(map, matching_rules, generators, path, true),
        Value::Array(array) => process_array(array, matching_rules, generators, path, true),
        _ => val.clone()
      },
      None => Value::Null
    }
  } else {
    Value::Object(obj.iter().map(|(key, val)| {
      let updated_path = if type_matcher {
        path.to_owned() + ".*"
      } else {
        path.to_owned() + "." + key
      };
      (key.clone(), match val {
        Value::Object(ref map) => process_object(map, matching_rules, generators, &updated_path, false),
        Value::Array(ref array) => process_array(array, matching_rules, generators, &updated_path, false),
        _ => val.clone()
      })
    }).collect())
  }
}

/// Process a JSON body with embedded matching rules and generators
pub fn process_json(body: String, matching_rules: &mut Category, generators: &mut Generators) -> String {
  match serde_json::from_str(&body) {
    Ok(json) => match json {
      Value::Object(ref map) => process_object(map, matching_rules, generators, &"$".to_string(), false).to_string(),
      Value::Array(ref array) => process_array(array, matching_rules, generators, &"$".to_string(), false).to_string(),
      _ => body
    },
    Err(_) => body
  }
}
