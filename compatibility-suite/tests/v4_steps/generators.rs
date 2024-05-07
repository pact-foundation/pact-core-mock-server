use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

use anyhow::anyhow;
use cucumber::{given, then, when};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_models::generators::{Generators, GeneratorTestMode};
use pact_models::json_utils::json_to_string;
use pact_models::path_exp::DocPath;
use pact_models::v4::http_parts::HttpRequest;
use serde_json::Value;

use pact_matching::generate_request;

use crate::shared_steps::{assert_value_type, setup_body};
use crate::v4_steps::V4World;

#[given(expr = "a request configured with the following generators:")]
fn a_request_configured_with_the_following_generators(world: &mut V4World, step: &Step) {
  let mut request = HttpRequest {
    path: "/path/one".to_string(),
    .. HttpRequest::default()
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

#[given(expr = "the generator test mode is set as {string}")]
fn the_generator_test_mode_is_set_as(world: &mut V4World, mode: String) {
  world.generator_test_mode = if mode == "Consumer" {
    GeneratorTestMode::Consumer
  } else {
    GeneratorTestMode::Provider
  };
}

#[when(expr = "the request is prepared for use with a {string} context:")]
async fn the_request_is_prepared_for_use_with_a_context(
  world: &mut V4World,
  step: &Step,
  context_field: String
) {
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
  world.generated_request = generate_request(&world.request, &world.generator_test_mode, &context).await;
  world.generated_body = world.generated_request.body.clone();
}

#[when("the request is prepared for use")]
async fn the_request_prepared_for_use(world: &mut V4World) {
  let context = world.generator_context.iter()
    .map(|(k, v)| (k.as_str(), v.clone()))
    .collect();
  world.generated_request = generate_request(&world.request, &world.generator_test_mode, &context).await;
  world.generated_body = world.generated_request.body.clone();
}

#[then(expr = "the body value for {string} will have been replaced with {string}")]
fn the_body_value_for_will_have_been_replaced_with_value(
  world: &mut V4World,
  path: String,
  value: String
) -> anyhow::Result<()> {
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(world.original_body.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();
  let json: Value = serde_json::from_str(world.generated_body.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();

  if element == original_element {
    Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  } else if json_to_string(&element) == value {
    Ok(())
  } else {
    Err(anyhow!("Expected value ({:?}) to be equal to {}", element, value))
  }
}

#[then(expr = "the body value for {string} will have been replaced with a(n) {string}")]
fn the_body_value_for_will_have_been_replaced_with_a_value(
  world: &mut V4World,
  path: String,
  value_type: String
) -> anyhow::Result<()> {
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(world.original_body.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();
  let json: Value = serde_json::from_str(world.generated_body.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();

  if element == original_element {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  }

  assert_value_type(value_type, element)
}
