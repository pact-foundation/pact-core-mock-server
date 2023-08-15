use anyhow::anyhow;
use cucumber::{given, then, when};
use itertools::Itertools;
use pact_models::json_utils::json_to_string;
use pact_models::pact::Pact;
use pact_models::PactSpecification;
use serde_json::Value;

use pact_consumer::builders::{InteractionBuilder, PactBuilder};

use crate::shared_steps::IndexType;
use crate::v4_steps::V4World;

#[given("an HTTP interaction is being defined for a consumer test")]
fn an_http_integration_is_being_defined_for_a_consumer_test(world: &mut V4World) {
  world.builder = PactBuilder::new_v4("V4 consumer", "V4 provider");
  world.integration_builder = InteractionBuilder::new("interaction for a consumer test", "");
}

#[given(expr = "a key of {string} is specified for the HTTP interaction")]
fn a_key_of_is_specified(world: &mut V4World, key: String) {
  world.integration_builder.with_key(key);
}

#[given("the HTTP interaction is marked as pending")]
fn the_interaction_is_marked_as_pending(world: &mut V4World) {
  world.integration_builder.pending(true);
}

#[given(expr = "a comment {string} is added to the HTTP interaction")]
fn a_comment_is_added(world: &mut V4World, value: String) {
  world.integration_builder.comment(value);
}

#[when("the Pact file for the test is generated")]
fn the_pact_file_for_the_test_is_generated(world: &mut V4World) {
  world.builder.push_interaction(&world.integration_builder.build_v4());
  if let Some(message_builder) = world.message_builder.as_ref() {
    world.builder.push_interaction(&message_builder.build());
  }
  world.pact = world.builder.build().as_v4_pact().unwrap();
  world.pact_json = world.pact.to_json(PactSpecification::V4).unwrap();
}

#[then(expr = "the {numType} interaction in the Pact file will have a type of {string}")]
fn the_interaction_in_the_pact_file_will_have_a_type_of(
  world: &mut V4World,
  index: IndexType,
  i_type: String
) -> anyhow::Result<()> {
  let interactions = world.pact_json["interactions"].as_array()
    .unwrap()
    .iter().sorted_by(|a, b| Ord::cmp(&json_to_string(b.get("type").unwrap()), &json_to_string(a.get("type").unwrap())))
    .collect_vec();

  let interaction = interactions[index.val()].as_object().unwrap();
  if let Some(interaction_type) = interaction.get("type") {
    if json_to_string(interaction_type) == i_type {
      Ok(())
    } else {
      Err(anyhow!("Expected interaction type attribute {} but got {}", i_type, interaction_type))
    }
  } else {
    Err(anyhow!("Interaction in Pact JSON has no type attribute"))
  }
}

#[then(expr = "the {numType} interaction in the Pact file will have {string} = {string}")]
fn the_first_interaction_in_the_pact_file_will_have(
  world: &mut V4World,
  index: IndexType,
  name: String,
  value: String
) -> anyhow::Result<()> {
  let interactions = world.pact_json["interactions"].as_array().unwrap();
  let interaction = interactions[index.val()].as_object().unwrap();
  let json: Value = serde_json::from_str(value.as_str()).unwrap();
  if let Some(actual_value) = interaction.get(name.as_str()) {
    if json == *actual_value {
      Ok(())
    } else {
      Err(anyhow!("Expected interaction {} attribute {} but got {}", name, value, actual_value))
    }
  } else {
    Err(anyhow!("Interaction in Pact JSON has no {} attribute", name))
  }
}
