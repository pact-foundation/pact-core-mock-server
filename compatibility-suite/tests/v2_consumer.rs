use cucumber::World;
use tracing_subscriber::EnvFilter;

use crate::shared_steps::consumer::ConsumerWorld;

pub mod shared_steps;

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  ConsumerWorld::cucumber()
    .fail_on_skipped()
    .before(|_feature, _, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
    }))
    .filter_run_and_exit("pact-compatibility-suite/features/V2", |feature, _rule, scenario| {
      feature.tags.iter().any(|tag| tag == "consumer") &&
        !scenario.tags.iter().any(|t| t == "wip")
    })
    .await;
}
