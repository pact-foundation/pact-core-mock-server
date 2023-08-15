use cucumber::given;
use pact_consumer::builders::MessageInteractionBuilder;
use crate::v4_steps::V4World;

#[given("a message interaction is being defined for a consumer test")]
fn a_message_integration_is_being_defined_for_a_consumer_test(world: &mut V4World) {
  world.message_builder = Some(MessageInteractionBuilder::new("a message"));
}
