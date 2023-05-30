use anyhow::anyhow;
use cucumber::{StatsWriter, World};
use tracing_subscriber::EnvFilter;

use crate::v1::consumer::ConsumerWorld;
use crate::v1::provider::ProviderWorld;

pub mod v1 {
  pub mod common;
  pub mod consumer;
  pub mod provider;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

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
    .max_concurrent_scenarios(1)
    .after(|_feature, _, _scenario, _status, world| Box::pin(async move {
      if let Some(world) = world {
        {
          let mut ms = world.provider_server.lock().unwrap();
          let _ = ms.shutdown();
        }
        for broker in &world.mock_brokers {
          let mut ms = broker.lock().unwrap();
          let _ = ms.shutdown();
        }
      }
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
