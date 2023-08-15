use cucumber::World;
use tracing_subscriber::EnvFilter;

use crate::v4_steps::V4World;

mod shared_steps;
mod v4_steps;

#[tokio::main]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  V4World::cucumber()
    .fail_on_skipped()
    .after(|_feature, _, _scenario, _status, world| Box::pin(async move {
      if let Some(world) = world {
        let mut ms = world.provider_server.lock().unwrap();
        let _ = ms.shutdown();
      }
    }))
    .filter_run_and_exit("pact-compatibility-suite/features/V4", |feature, _rule, _scenario| {
      // feature.tags.iter().all(|tag| tag != "provider" && tag != "message")
      true
    })
    .await;
}
