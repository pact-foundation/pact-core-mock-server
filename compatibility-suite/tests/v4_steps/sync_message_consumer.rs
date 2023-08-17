use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use anyhow::anyhow;
use cucumber::gherkin::Step;

use cucumber::{given, then, when};
use pact_models::generators::Generators;
use pact_models::http_parts::HttpPart;
use pact_models::pact::{Pact, read_pact};
use pact_models::PactSpecification;
use pact_models::path_exp::DocPath;
use pact_models::v4::message_parts::MessageContents;
use serde_json::Value;

use pact_consumer::builders::SyncMessageInteractionBuilder;
use crate::shared_steps::{assert_value_type, IndexType};
use crate::v4_steps::message_provider::setup_body;

use crate::v4_steps::V4World;

#[given("a synchronous message interaction is being defined for a consumer test")]
fn a_synchronous_message_interaction_is_being_defined_for_a_consumer_test(world: &mut V4World) {
  world.sync_message_builder = Some(SyncMessageInteractionBuilder::new("synchronous message interaction"));
}

#[given(expr = "a key of {string} is specified for the synchronous message interaction")]
fn a_key_of_is_specified_for_the_synchronous_message_interaction(world: &mut V4World, key: String) {
  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.with_key(key);
}

#[given("the synchronous message interaction is marked as pending")]
fn the_synchronous_message_interaction_is_marked_as_pending(world: &mut V4World) {
  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.pending(true);
}

#[given(expr = "a comment {string} is added to the synchronous message interaction")]
fn a_comment_is_added_to_the_synchronous_message_interaction(world: &mut V4World, comment: String) {
  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.comment(comment);
}

#[given(expr = "the message request payload contains the {string} JSON document")]
fn the_message_request_payload_contains_the_json_document(
  world: &mut V4World,
  fixture: String
) -> anyhow::Result<()> {
  let mut fixture = File::open(format!("pact-compatibility-suite/fixtures/{}.json", fixture))?;
  let mut buffer = Vec::new();
  fixture.read_to_end(&mut buffer)?;

  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.request_body(buffer, Some("application/json".into()));

  Ok(())
}

#[given(expr = "the message response payload contains the {string} document")]
fn the_message_response_payload_contains_the_document(
  world: &mut V4World,
  fixture: String
) -> anyhow::Result<()> {
  let mut response = MessageContents::default();
  setup_body(&fixture, &mut response);
  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.response_contents(&response);
  Ok(())
}

#[given("the message request contains the following metadata:")]
fn the_message_request_contains_the_following_metadata(world: &mut V4World, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (h.clone(), index))
      .collect::<HashMap<String, usize>>();
    for values in table.rows.iter().skip(1) {
      let key = values.get(*headers.get("key").unwrap()).unwrap();
      let value = values.get(*headers.get("value").unwrap()).unwrap();
      let json: Value = if value.starts_with("JSON:") {
        serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
      } else {
        Value::String(value.clone())
      };
      let builder = world.sync_message_builder.as_mut().unwrap();
      builder.request_metadata(key, json);
    }
  }
}

#[given(expr = "a provider state {string} for the synchronous message is specified")]
fn a_provider_state_for_the_synchronous_message_is_specified(
  world: &mut V4World,
  state: String
) {
  let builder = world.sync_message_builder.as_mut().unwrap();
  builder.given(state);
}

#[given(expr = "a provider state {string} for the synchronous message is specified with the following data:")]
fn a_provider_state_for_the_synchronous_message_is_specified_with_the_following_data(
  world: &mut V4World,
  step: &Step,
  state: String
) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    let params = table.rows.get(1).unwrap().iter().enumerate().map(|(i, v)| {
      let key = headers.get(&i).unwrap();
      let json: Value = serde_json::from_str(v).unwrap();
      (key.clone(), json)
    }).collect();
    let builder = world.sync_message_builder.as_mut().unwrap();
    builder.given_with_params(state, &Value::Object(params));
  }
}

#[given("the message request is configured with the following:")]
fn the_message_request_is_configured_with_the_following(
  world: &mut V4World,
  step: &Step
) {
  let builder = world.sync_message_builder.as_mut().unwrap();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    let mut message = MessageContents::default();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => {
            setup_body(value, &mut message);
            builder.request_contents(&message);
          },
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
            message.generators = generators.clone();
          }
          "metadata" => {
            let json: Value = serde_json::from_str(value).unwrap();
            message.metadata.extend(json.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())));
          }
          _ => {}
        }
      }
    }
    builder.request_contents(&message);
  }
}

#[given("the message response is configured with the following:")]
fn the_message_response_is_configured_with_the_following(
  world: &mut V4World,
  step: &Step
) {
  let builder = world.sync_message_builder.as_mut().unwrap();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    let mut message = MessageContents::default();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => {
            setup_body(value, &mut message);
            builder.request_contents(&message);
          },
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
            message.generators = generators.clone();
          }
          "metadata" => {
            let json: Value = serde_json::from_str(value).unwrap();
            message.metadata.extend(json.as_object().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())));
          }
          _ => {}
        }
      }
    }
    builder.response_contents(&message);
  }
}

#[when("the message is successfully processed")]
fn the_message_is_successfully_processed(world: &mut V4World) {
  if let Some(integration_builder) = world.integration_builder.as_ref() {
    world.builder.push_interaction(&integration_builder.build_v4());
  }
  if let Some(message_builder) = world.message_builder.as_ref() {
    world.builder.push_interaction(&message_builder.build());
  }
  if let Some(message_builder) = world.sync_message_builder.as_ref() {
    world.builder.push_interaction(&message_builder.build());
  }
  world.pact = world.builder.build().as_v4_pact().unwrap();
  world.pact_json = world.pact.to_json(PactSpecification::V4).unwrap();
  let dir = PathBuf::from("target/compatibility-suite/v4").join(&world.scenario_id);
  world.received_sync_messages = world.builder.with_output_dir(dir).synchronous_messages().collect();
}

#[then(expr = "the received message payload will contain the {string} document")]
fn the_received_message_payload_will_contain_the_document(
  world: &mut V4World,
  fixture: String
) -> anyhow::Result<()> {
  let mut message = MessageContents::default();
  setup_body(&fixture, &mut message);
  if world.received_sync_messages.iter().find(|m| {
    if let Some(response) = m.response.first() {
      response.contents.value() == message.contents.value()
    } else {
      false
    }
  }).is_some() {
    Ok(())
  } else {
    Err(anyhow!("The required message was not received"))
  }
}

#[then(expr = "the received message content type will be {string}")]
fn the_received_message_content_type_will_be(
  world: &mut V4World,
  content_type: String
) -> anyhow::Result<()> {
  if world.received_sync_messages.iter().find(|m| {
    if let Some(response) = m.response.first() {
      response.contents.content_type().unwrap_or_default().to_string() == content_type
    } else {
      false
    }
  }).is_some() {
    Ok(())
  } else {
    Err(anyhow!("The required message was not received"))
  }
}

#[then("the consumer test will have passed")]
fn the_consumer_test_will_have_passed(_world: &mut V4World) {
  // no-op
}

#[then("a Pact file for the message interaction will have been written")]
fn a_pact_file_for_the_message_interaction_will_have_been_written(world: &mut V4World) -> anyhow::Result<()> {
  let dir = PathBuf::from("target/compatibility-suite/v4").join(&world.scenario_id);
  let pact_file = dir.join("C-P.json");
  if pact_file.exists() {
    let pact = read_pact(&pact_file)?;
    if pact.specification_version() == PactSpecification::V4 {
      world.pact = pact.as_v4_pact().unwrap();
      Ok(())
    } else {
      Err(anyhow!("Expected Pact file to be V4 Pact, but was {}", pact.specification_version()))
    }
  } else {
    Err(anyhow!("No pact file found: {}", pact_file.to_string_lossy()))
  }
}

#[then(expr = "the pact file will contain {int} interaction")]
fn the_pact_file_will_contain_interaction(world: &mut V4World, num: usize) -> anyhow::Result<()> {
  if world.pact.interactions.len() == num {
    Ok(())
  } else {
    Err(anyhow!("Pact had {} interactions", world.pact.interactions.len()))
  }
}

#[then(expr = "the first interaction in the pact file will contain the {string} document as the request")]
fn the_first_interaction_in_the_pact_file_will_contain_the_document_as_the_request(
  world: &mut V4World,
  fixture: String
) -> anyhow::Result<()> {
  let mut message = MessageContents::default();
  setup_body(&fixture, &mut message);
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let result = if message.content_type().unwrap_or_default().is_json() {
    let json1: Value = serde_json::from_slice(&*message.contents.value().unwrap_or_default()).unwrap();
    let json2: Value = serde_json::from_slice(&*interaction.request.contents.value().unwrap_or_default()).unwrap();
    json1 == json2
  } else {
    interaction.request.contents == message.contents
  };
  if result {
    Ok(())
  } else {
    Err(anyhow!("The required message was not found"))
  }
}

#[then(expr = "the first interaction in the pact file request content type will be {string}")]
fn the_first_interaction_in_the_pact_file_request_content_type_will_be(
  world: &mut V4World,
  content_type: String
) -> anyhow::Result<()> {
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if interaction.request.content_type().unwrap_or_default().to_string() == content_type {
    Ok(())
  } else {
    Err(anyhow!("The required message was not found"))
  }
}

#[then(expr = "the first interaction in the pact file will contain the {string} document as a response")]
fn the_first_interaction_in_the_pact_file_will_contain_the_document_as_a_response(
  world: &mut V4World,
  fixture: String
) -> anyhow::Result<()> {
  let mut message = MessageContents::default();
  setup_body(&fixture, &mut message);
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let result = if message.content_type().unwrap_or_default().is_json() {
    let json1: Value = serde_json::from_slice(&*message.contents.value().unwrap_or_default()).unwrap();
    let json2: Value = serde_json::from_slice(&*interaction.response[0].contents.value().unwrap_or_default()).unwrap();
    json1 == json2
  } else {
    interaction.response[0].contents == message.contents
  };
  if result {
    Ok(())
  } else {
    Err(anyhow!("The required message was not found"))
  }
}

#[then(expr = "the first interaction in the pact file response content type will be {string}")]
fn the_first_interaction_in_the_pact_file_response_content_type_will_be(
  world: &mut V4World,
  content_type: String
) -> anyhow::Result<()> {
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if interaction.response[0].content_type().unwrap_or_default().to_string() == content_type {
    Ok(())
  } else {
    Err(anyhow!("The required message was not found"))
  }
}

#[then(expr = "the first interaction in the pact file will contain {int} response messages")]
fn the_first_interaction_in_the_pact_file_will_contain_response_messages(
  world: &mut V4World,
  num: usize
) -> anyhow::Result<()> {
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if interaction.response.len() == num {
    Ok(())
  } else {
    Err(anyhow!("The message only had {} response messages", interaction.response.len()))
  }
}

#[then(expr = "the first interaction in the pact file will contain the {string} document as the {numType} response message")]
fn the_first_interaction_in_the_pact_file_will_contain_the_document_as_the_first_response_message(
  world: &mut V4World,
  fixture: String,
  index: IndexType
) -> anyhow::Result<()> {
  let mut message = MessageContents::default();
  setup_body(&fixture, &mut message);
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let result = if message.content_type().unwrap_or_default().is_json() {
    let json1: Value = serde_json::from_slice(&*message.contents.value().unwrap_or_default()).unwrap();
    let json2: Value = serde_json::from_slice(&*interaction.response[index.val()].contents.value().unwrap_or_default()).unwrap();
    json1 == json2
  } else {
    interaction.response[index.val()].contents == message.contents
  };
  if result {
    Ok(())
  } else {
    Err(anyhow!("The required message was not found"))
  }
}

#[then(expr = "the first message in the pact file will contain the request message metadata {string} == {string}")]
fn the_first_message_in_the_pact_file_will_contain_the_request_message_metadata(
  world: &mut V4World,
  key: String,
  value: String
) -> anyhow::Result<()> {
  let json: Value = if value.starts_with("JSON:") {
    let value_str = value.strip_prefix("JSON:")
      .unwrap_or(value.as_str())
      .trim()
      .replace("\\\"", "\"");
    serde_json::from_str(value_str.as_str()).unwrap()
  } else {
    Value::String(value.clone())
  };
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if json == *interaction.request.metadata.get(&key).unwrap() {
    Ok(())
  } else {
    Err(anyhow!("The required message was not received"))
  }
}

#[then(expr = "the first message in the pact file will contain {int} provider state(s)")]
fn the_first_message_in_the_pact_file_will_contain_provider_states(
  world: &mut V4World,
  states: usize
) -> anyhow::Result<()> {
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if interaction.provider_states.len() == states {
    Ok(())
  } else {
    Err(anyhow!("The message had {} provider states", interaction.provider_states.len()))
  }
}

#[then(expr = "the first message in the Pact file will contain provider state {string}")]
fn the_first_message_in_the_pact_file_will_contain_provider_state(
  world: &mut V4World,
  state: String
) -> anyhow::Result<()> {
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  if interaction.provider_states.iter().find(|ps| ps.name == state).is_some() {
    Ok(())
  } else {
    Err(anyhow!("The message did not have '{}' provider state", state))
  }
}

#[then(expr = "the provider state {string} for the message will contain the following parameters:")]
fn the_provider_state_for_the_message_will_contain_the_following_parameters(
  world: &mut V4World,
  step: &Step,
  state: String
) -> anyhow::Result<()> {
  let table = step.table.as_ref().unwrap();
  let params_str = table.rows.get(1).unwrap().first().unwrap();
  let params = serde_json::from_str::<Value>(params_str.as_str())
    .unwrap()
    .as_object()
    .unwrap()
    .iter()
    .map(|(k, v)| (k.clone(), v.clone()))
    .collect();
  let interaction = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let provider_state = interaction.provider_states.iter().find(|ps| ps.name == state).unwrap();
  if provider_state.params == params {
    Ok(())
  } else {
    Err(anyhow!("Expected provider state '{}' to have parameters {:?} but were {:?}", state,
      params, provider_state.params))
  }
}

#[then(expr = "the message request contents for {string} will have been replaced with a(n) {string}")]
fn the_message_contents_for_will_have_been_replaced_with_an(
  world: &mut V4World,
  path: String,
  value_type: String
) -> anyhow::Result<()> {
  let message = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(message.request.contents.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();
  let json: Value = serde_json::from_str(world.received_sync_messages.first().unwrap().request.contents.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();

  if element == original_element {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  }

  assert_value_type(value_type, element)
}

#[then(expr = "the message response contents for {string} will have been replaced with a(n) {string}")]
fn the_message_response_contents_for_will_have_been_replaced_with_an(
  world: &mut V4World,
  path: String,
  value_type: String
) -> anyhow::Result<()> {
  let message = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let response = &message.response[0];
  let path = DocPath::new(path).unwrap();
  let original_json: Value = serde_json::from_str(response.contents.value_as_string().unwrap().as_str()).unwrap();
  let original_element = original_json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();
  let received_response = &world.received_sync_messages.first().unwrap().response[0];
  let json: Value = serde_json::from_str(received_response.contents.value_as_string().unwrap().as_str()).unwrap();
  let element = json.pointer(path.as_json_pointer().unwrap().as_str()).unwrap();

  if element == original_element {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_element))
  }

  assert_value_type(value_type, element)
}

#[then(expr = "the received message request metadata will contain {string} == {string}")]
fn the_received_message_request_metadata_will_contain(
  world: &mut V4World,
  key: String,
  value: String
) -> anyhow::Result<()> {
  let json: Value = if value.starts_with("JSON:") {
    let value_str = value.strip_prefix("JSON:")
      .unwrap_or(value.as_str())
      .trim()
      .replace("\\\"", "\"");
    serde_json::from_str(value_str.as_str()).unwrap()
  } else {
    Value::String(value.clone())
  };
  if world.received_sync_messages.iter().find(|m| {
    if let Some(value) = m.request.metadata.get(&key) {
      *value == json
    } else {
      false
    }
  }).is_some() {
    Ok(())
  } else {
    Err(anyhow!("The required message was not received"))
  }
}

#[then(expr = "the received message request metadata will contain {string} replaced with a(n) {string}")]
fn the_received_message_request_metadata_will_contain_replaced_with_an(
  world: &mut V4World,
  key: String,
  value_type: String
) -> anyhow::Result<()> {
  let message = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let original_json = message.request.metadata.get(&key).unwrap();
  let received = &world.received_sync_messages.first().unwrap().request;
  let json = received.metadata.get(&key).unwrap();

  if json == original_json {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_json))
  }

  assert_value_type(value_type, json)
}

#[then(expr = "the received message response metadata will contain {string} == {string}")]
fn the_received_message_response_metadata_will_contain(
  world: &mut V4World,
  key: String,
  value: String
) -> anyhow::Result<()> {
  let json: Value = if value.starts_with("JSON:") {
    let value_str = value.strip_prefix("JSON:")
      .unwrap_or(value.as_str())
      .trim()
      .replace("\\\"", "\"");
    serde_json::from_str(value_str.as_str()).unwrap()
  } else {
    Value::String(value.clone())
  };
  if world.received_sync_messages.iter().find(|m| {
    if let Some(value) = m.response[0].metadata.get(&key) {
      *value == json
    } else {
      false
    }
  }).is_some() {
    Ok(())
  } else {
    Err(anyhow!("The required message was not received"))
  }
}

#[then(expr = "the received message response metadata will contain {string} replaced with a(n) {string}")]
fn the_received_message_response_metadata_will_contain_replaced_with_an(
  world: &mut V4World,
  key: String,
  value_type: String
) -> anyhow::Result<()> {
  let message = world.pact.interactions[0].as_v4_sync_message().unwrap();
  let response = &message.response[0];
  let original_json = response.metadata.get(&key).unwrap();
  let received_response = &world.received_sync_messages.first().unwrap().response[0];
  let json = received_response.metadata.get(&key).unwrap();

  if json == original_json {
    return Err(anyhow!("Expected original ({:?}) to have been replaced", original_json))
  }

  assert_value_type(value_type, json)
}
