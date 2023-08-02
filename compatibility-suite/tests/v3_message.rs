use cucumber::World;
use tracing_subscriber::EnvFilter;

use v3_steps::message::V3MessageWorld;

mod shared_steps;
mod v3_steps;

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  V3MessageWorld::cucumber()
    .fail_on_skipped()
    .max_concurrent_scenarios(1)
    .before(|_, _, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
    }))
    .filter_run_and_exit("pact-compatibility-suite/features/V3", |feature, _rule, _scenario| {
      feature.tags.iter().any(|tag| tag == "message")
    })
    .await;
}
