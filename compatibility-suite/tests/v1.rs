use cucumber::{given, World};

#[derive(Debug, Default, World)]
pub struct PactWorld {

}

fn main() {
  futures::executor::block_on(
    PactWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit("pact-compatibility-suite/features/V1"));
}
