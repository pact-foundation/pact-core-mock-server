use cucumber::given;
use pact_consumer::builders::MessageInteractionBuilder;
use crate::v4_steps::V4World;

#[given("a message interaction is being defined for a consumer test")]
fn a_message_integration_is_being_defined_for_a_consumer_test(world: &mut V4World) {
  world.message_builder = Some(MessageInteractionBuilder::new("a message"));
}

#[given(expr = "a key of {string} is specified for the message interaction")]
fn message_a_key_of_is_specified(world: &mut V4World, key: String) {
  let builder = world.message_builder.as_mut().unwrap();
  builder.with_key(key);
}

#[given("the message interaction is marked as pending")]
fn the_message_interaction_is_marked_as_pending(world: &mut V4World) {
  let builder = world.message_builder.as_mut().unwrap();
  builder.pending(true);
}

#[given(expr = "a comment {string} is added to the message interaction")]
fn message_a_comment_is_added(world: &mut V4World, value: String) {
  let builder = world.message_builder.as_mut().unwrap();
  builder.comment(value);
}
