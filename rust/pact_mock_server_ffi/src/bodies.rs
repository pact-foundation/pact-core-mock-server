//! Functions to support processing request/response bodies

use pact_matching::models::matchingrules::{Category, MatchingRule, RuleLogic};
use pact_matching::models::generators::{Generators, Generator, GeneratorCategory};
use serde_json::{Value, Map};
use pact_matching::models::json_utils::json_to_string;
use pact_matching::models::{Request, OptionalBody, Response};
use maplit::*;
use hyper::header::Headers;
use formdata::{FormData, FilePart};
use std::path::Path;
use log::*;

const CONTENT_TYPE_HEADER: &str = "Content-Type";

/// Process an array with embedded matching rules and generators
pub fn process_array(array: &[Value], matching_rules: &mut Category, generators: &mut Generators, path: &str, type_matcher: bool) -> Value {
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

/// Process an object (map) with embedded matching rules and generators
pub fn process_object(obj: &Map<String, Value>, matching_rules: &mut Category, generators: &mut Generators, path: &str, type_matcher: bool) -> Value {
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

/// Process a JSON body with embedded matching rules and generators
pub fn process_json_value(body: &Value, matching_rules: &mut Category, generators: &mut Generators) -> String {
  match body {
    Value::Object(ref map) => process_object(map, matching_rules, generators, &"$".to_string(), false).to_string(),
    Value::Array(ref array) => process_array(array, matching_rules, generators, &"$".to_string(), false).to_string(),
    _ => body.to_string()
  }
}

/// Setup the request as a multipart form upload
pub fn request_multipart(request: &mut Request, boundary: &str, body: OptionalBody, content_type: &str, part_name: &str) {
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
  request.matching_rules.add_category("body")
    .add_rule(format!("$['{}']", part_name), MatchingRule::ContentType(content_type.into()), &RuleLogic::And);
  request.matching_rules.add_category("header")
    .add_rule("Content-Type", MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), &RuleLogic::And);
}

/// Setup the response as a multipart form upload
pub fn response_multipart(response: &mut Response, boundary: &str, body: OptionalBody, content_type: &str, part_name: &str) {
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
  response.matching_rules.add_category("body")
    .add_rule(format!("$['{}']", part_name), MatchingRule::ContentType(content_type.into()), &RuleLogic::And);
  response.matching_rules.add_category("header")
    .add_rule("Content-Type", MatchingRule::Regex(r"multipart/form-data;(\s*charset=[^;]*;)?\s*boundary=.*".into()), &RuleLogic::And);
}

/// Loads an example file as a MIME Multipart body
pub fn file_as_multipart_body(file: &str, part_name: &str, boundary: &str) -> Result<OptionalBody, String> {
  let headers = Headers::new();
  let formdata = FormData {
    fields: vec![],
    files: vec![(part_name.to_string(), FilePart::new(headers, Path::new(file)))]
  };
  let mut buffer: Vec<u8> = vec![];
  match formdata::write_formdata(&mut buffer, &boundary.as_bytes().to_vec(), &formdata) {
    Ok(_) => Ok(OptionalBody::Present(buffer.clone())),
    Err(err) => {
      warn!("convert_ptr_to_mime_part_body: Failed to generate multipart body: {}", err);
      Err(format!("convert_ptr_to_mime_part_body: Failed to generate multipart body: {}", err))
    }
  }
}
