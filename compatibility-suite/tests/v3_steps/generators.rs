use std::collections::HashMap;
use std::fmt::Write;
use std::fs::File;
use std::io::BufReader;

use anyhow::anyhow;
use cucumber::{given, then, when};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_models::generators::{Generators, GeneratorTestMode};
use pact_models::json_utils::json_to_string;
use pact_models::path_exp::{DocPath, PathToken};
use pact_models::request::Request;
use pact_models::response::Response;
use regex::Regex;
use serde_json::Value;

use pact_matching::{generate_request, generate_response};

use crate::shared_steps::{assert_value_type, setup_body};
use crate::v3_steps::V3World;

#[given(expr = "a request configured with the following generators:")]
fn a_request_configured_with_the_following_generators(world: &mut V3World, step: &Step) {
  let mut request = Request {
    path: "/path/one".to_string(),
    .. Request::default()
  };

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => setup_body(value, &mut request, None),
          "generators" => {
            let json: Value = if value.starts_with("JSON:") {
              serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
            } else {
              let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
                .expect(format!("could not load fixture '{}'", value).as_str());
              let reader = BufReader::new(f);
              serde_json::from_reader(reader).unwrap()
            };
            let mut generators = Generators::default();
            generators.load_from_map(json.as_object().unwrap()).unwrap();
            request.generators = generators;
          }
          _ => {}
        }
      }
    }
  }

  world.original_body = request.body.clone();
  world.request = request;
}

#[given(expr = "a response configured with the following generators:")]
fn a_response_configured_with_the_following_generators(world: &mut V3World, step: &Step) {
  let mut response = Response::default();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => setup_body(value, &mut response, None),
          "generators" => {
            let json: Value = if value.starts_with("JSON:") {
              serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
            } else {
              let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
                .expect(format!("could not load fixture '{}'", value).as_str());
              let reader = BufReader::new(f);
              serde_json::from_reader(reader).unwrap()
            };
            let mut generators = Generators::default();
            generators.load_from_map(json.as_object().unwrap()).unwrap();
            response.generators = generators;
          }
          _ => {}
        }
      }
    }
  }

  world.original_body = response.body.clone();
  world.response = response;
}

#[given(expr = "the generator test mode is set as {string}")]
fn the_generator_test_mode_is_set_as(world: &mut V3World, mode: String) {
  world.generator_test_mode = if mode == "Consumer" {
    GeneratorTestMode::Consumer
  } else {
    GeneratorTestMode::Provider
  };
}

#[when("the request is prepared for use")]
async fn the_request_prepared_for_use(world: &mut V3World) {
  let context = world.generator_context.iter()
    .map(|(k, v)| (k.as_str(), v.clone()))
    .collect();
  world.generated_request = generate_request(&world.request.as_v4_request(),
    &world.generator_test_mode, &context).await.as_v3_request();
  world.generated_body = world.generated_request.body.clone();
}

#[when("the response is prepared for use")]
async fn the_response_is_prepared_for_use(world: &mut V3World) {
  let context = world.generator_context.iter()
    .map(|(k, v)| (k.as_str(), v.clone()))
    .collect();
  world.generated_response = generate_response(&world.response.as_v4_response(),
    &world.generator_test_mode, &context).await.as_v3_response();
  world.generated_body = world.generated_response.body.clone();
}

#[when(expr = "the request is prepared for use with a {string} context:")]
async fn the_request_is_prepared_for_use_with_a_context(
  world: &mut V3World,
  step: &Step,
  context_field: String
) {
  // FUCK! That is all I can say about this at this point.
  let context = if let Some(table) = step.table.as_ref() {
    let value = table.rows.first().unwrap().first().unwrap();
    let json: Value = serde_json::from_str(value).unwrap();
    let attributes = json.as_object().unwrap();
    let map = attributes.iter()
      .map(|(k, v)| (k.clone(), v.clone()))
      .collect::<HashMap<_, _>>();
    if context_field == "providerState" {
      map
    } else if context_field == "mockServer" {
      hashmap!{
        context_field.to_string() => Value::Object(map.iter()
          .map(|(k, v)| {
            if k == "href" {
              ("url".to_string(), v.clone())
            } else {
              (k.clone(), v.clone())
            }
          }).collect())
      }
    } else {
      hashmap!{
        context_field.to_string() => Value::Object(map.iter()
          .map(|(k, v)| (k.clone(), v.clone())).collect())
      }
    }
  } else {
    world.generator_context.clone()
  };

  let context = context.iter()
    .map(|(k, v)| (k.as_str(), v.clone()))
    .collect::<HashMap<_, _>>();
  world.generated_request = generate_request(&world.request.as_v4_request(),
    &world.generator_test_mode, &context).await.as_v3_request();
  world.generated_body = world.generated_request.body.clone();
}

#[then(expr = "the body value for {string} will have been replaced with a(n) {string}")]
fn the_body_value_for_will_have_been_replaced_with_a_value(
  world: &mut V3World,
  path: String,
  value_type: String
) -> anyhow::Result<()> {
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(world.original_body.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(as_json_pointer(&path).as_str()).unwrap();
  let json: Value = serde_json::from_str(world.generated_body.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(as_json_pointer(&path).as_str()).unwrap();

  if element == original_element {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  }

  assert_value_type(value_type, element)
}

// TODO: Replace this with version from pact_models
pub fn as_json_pointer(path: &DocPath) -> String {
  let mut buffer = String::new();

  for token in path.tokens() {
    match token {
      PathToken::Root => {},
      PathToken::Field(v) => {
        let parsed = v.replace('~', "~0")
          .replace('/', "~1");
        let _ = write!(buffer, "/{}", parsed);
      }
      PathToken::Index(i) => {
        buffer.push('/');
        buffer.push_str(i.to_string().as_str());
      }
      PathToken::Star => {
        panic!("* can not be converted to a JSON pointer");
      }
      PathToken::StarIndex => {
        panic!("* can not be converted to a JSON pointer");
      }
    }
  }

  buffer
}

#[then(expr = "the body value for {string} will have been replaced with {string}")]
fn the_body_value_for_will_have_been_replaced_with_value(
  world: &mut V3World,
  path: String,
  value: String
) -> anyhow::Result<()> {
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(world.original_body.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(as_json_pointer(&path).as_str()).unwrap();
  let json: Value = serde_json::from_str(world.generated_body.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(as_json_pointer(&path).as_str()).unwrap();

  if element == original_element {
    Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  } else if json_to_string(&element) == value {
    Ok(())
  } else {
    Err(anyhow!("Expected value ({:?}) to be equal to {}", element, value))
  }
}

#[then(expr = "the request {string} will be set as {string}")]
fn the_request_will_be_set_as(
  world: &mut V3World,
  request_part: String,
  value: String
) -> anyhow::Result<()> {
  match request_part.as_str() {
    "path" => {
      if world.generated_request.path == value {
        Ok(())
      } else {
        Err(anyhow!("Expected path to be {} but was {}", value, world.generated_request.path))
      }
    }
    _ => Err(anyhow!("Invalid HTTP part: {}", request_part))
  }
}

#[then(expr = "the request {string} will match {string}")]
fn the_request_will_match(
  world: &mut V3World,
  request_part: String,
  regex: String
) -> anyhow::Result<()> {
  let regex = Regex::new(regex.as_str()).unwrap();
  let key_regex = Regex::new(r"\[(.*)]").unwrap();
  if request_part.as_str() == "path" {
    if regex.is_match(world.generated_request.path.as_str()) {
      Ok(())
    } else {
      Err(anyhow!("Expected path to match {} but was {}", regex, world.generated_request.path))
    }
  } else if request_part.starts_with("header") {
    let header = key_regex.captures(request_part.as_str()).unwrap().get(1).unwrap().as_str();
    if let Some(headers) = &world.generated_request.headers {
      if let Some(value) = headers.get(header) {
        if value.iter().all(|v| regex.is_match(v.as_ref())) {
          Ok(())
        } else {
          Err(anyhow!("Request header {} has a value that does not match {}", header, regex))
        }
      } else {
        Err(anyhow!("Request does not have header {} set", header))
      }
    } else {
      Err(anyhow!("Request does not have any headers set"))
    }
  } else if request_part.starts_with("queryParameter") {
    let parameter = key_regex.captures(request_part.as_str()).unwrap().get(1).unwrap().as_str();
    if let Some(query) = &world.generated_request.query {
      if let Some(value) = query.get(parameter) {
        if value.iter().all(|v| regex.is_match(v.as_ref())) {
          Ok(())
        } else {
          Err(anyhow!("Request query parameter {} has a value that does not match {}", parameter, regex))
        }
      } else {
        Err(anyhow!("Request does not have query parameter {} set", parameter))
      }
    } else {
      Err(anyhow!("Request does not have any query parameters set"))
    }
  } else {
    Err(anyhow!("Invalid HTTP part: {}", request_part))
  }
}

#[then(expr = "the response {string} will not be {string}")]
fn the_response_will_not_be(
  world: &mut V3World,
  response_part: String,
  value: String
) -> anyhow::Result<()> {
  match response_part.as_str() {
    "status" => {
      if world.generated_response.status != value.parse::<u16>().unwrap() {
        Ok(())
      } else {
        Err(anyhow!("Expected status to be NOT be {} but was", value))
      }
    }
    _ => Err(anyhow!("Invalid HTTP part: {}", response_part))
  }
}

#[then(expr = "the response {string} will match {string}")]
fn the_response_will_match(
  world: &mut V3World,
  response_part: String,
  regex: String
) -> anyhow::Result<()> {
  let regex = Regex::new(regex.as_str()).unwrap();
  let key_regex = Regex::new(r"\[(.*)]").unwrap();
  if response_part.as_str() == "status" {
    if regex.is_match(world.generated_response.status.to_string().as_str()) {
      Ok(())
    } else {
      Err(anyhow!("Expected status to match {} but was {}", regex, world.generated_response.status))
    }
  } else if response_part.starts_with("header") {
    let header = key_regex.captures(response_part.as_str()).unwrap().get(1).unwrap().as_str();
    if let Some(headers) = &world.generated_response.headers {
      if let Some(value) = headers.get(header) {
        if value.iter().all(|v| regex.is_match(v.as_ref())) {
          Ok(())
        } else {
          Err(anyhow!("Response header {} has a value that does not match {}", header, regex))
        }
      } else {
        Err(anyhow!("Response does not have header {} set", header))
      }
    } else {
      Err(anyhow!("Response does not have any headers set"))
    }
  } else {
    Err(anyhow!("Invalid HTTP part: {}", response_part))
  }
}