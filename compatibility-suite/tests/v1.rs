use cucumber::{given, World};

#[derive(Debug, Default, World)]
pub struct PactWorld {

}

fn main() {
  futures::executor::block_on(PactWorld::run("pact-compatibility-suite/features/V1"));
}
