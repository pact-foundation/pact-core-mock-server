use anyhow::anyhow;
use cucumber::{StatsWriter, World};

use crate::consumer::ConsumerWorld;
use crate::provider::ProviderWorld;

mod consumer;
mod provider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  println!();
  let consumer_result = ConsumerWorld::cucumber()
    .fail_on_skipped()
    .before(|_feature, _, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
    }))
    .filter_run("pact-compatibility-suite/features/V1", |feature, _rule, _scenario| {
      feature.tags.iter().any(|tag| tag == "consumer")
    })
    .await;

  println!();
  let provider_result = ProviderWorld::cucumber()
    .fail_on_skipped()
    .before(|_feature, _, scenario, world| Box::pin(async move {
      // world.scenario_id = scenario.name.clone();
    }))
    .filter_run("pact-compatibility-suite/features/V1", |feature, _rule, _scenario| {
      feature.tags.iter().any(|tag| tag == "provider")
    })
    .await;

  if consumer_result.execution_has_failed() || provider_result.execution_has_failed() {
    Err(anyhow!("Test run has failed"))
  } else {
    Ok(())
  }
}
