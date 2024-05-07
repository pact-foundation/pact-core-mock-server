use cucumber::given;

use pact_consumer::builders::{InteractionBuilder, PactBuilder};

use crate::v4_steps::V4World;

#[given("an HTTP interaction is being defined for a consumer test")]
fn an_http_integration_is_being_defined_for_a_consumer_test(world: &mut V4World) {
  world.builder = PactBuilder::new_v4("V4 consumer", "V4 provider");
  world.integration_builder = Some(InteractionBuilder::new("interaction for a consumer test", ""));
}

#[given(expr = "a key of {string} is specified for the HTTP interaction")]
fn a_key_of_is_specified(world: &mut V4World, key: String) {
  let builder = world.integration_builder.as_mut().unwrap();
  builder.with_key(key);
}

#[given("the HTTP interaction is marked as pending")]
fn the_interaction_is_marked_as_pending(world: &mut V4World) {
  let builder = world.integration_builder.as_mut().unwrap();
  builder.pending(true);
}

#[given(expr = "a comment {string} is added to the HTTP interaction")]
fn a_comment_is_added(world: &mut V4World, value: String) {
  let builder = world.integration_builder.as_mut().unwrap();
  builder.comment(value);
}
