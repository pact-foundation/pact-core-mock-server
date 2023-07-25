use cucumber::World;
use tracing_subscriber::EnvFilter;

use crate::v3_steps::V3World;

mod shared_steps;
mod v3_steps;

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  V3World::cucumber()
    .fail_on_skipped()
    .filter_run_and_exit("pact-compatibility-suite/features/V3", |feature, _rule, _scenario| {
      feature.tags.iter().all(|tag| tag != "provider")
    })
    .await;
}
