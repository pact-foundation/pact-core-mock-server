use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use cucumber::World;
use itertools::Itertools;
use rocket::http::{ContentType, Header};
use rocket::Responder;
use rocket::serde::json::Json;
use serde::Deserialize;
use serde_json::json;
use tracing::debug;
use tracing_subscriber::EnvFilter;

use v3_steps::message::V3MessageWorld;

use crate::v3_steps::message::MESSAGES;

mod shared_steps;
mod v3_steps;

#[derive(Deserialize, Default, Debug)]
struct MessageDetails {
  description: String
}

#[derive(Responder)]
struct MessageResponder<'a> {
  payload: Option<Vec<u8>>,
  content_type: ContentType,
  metadata: Header<'a>
}

#[rocket::post("/", data = "<request>")]
async fn messages(request: Json<MessageDetails>) -> Option<MessageResponder<'static>> {
  let details = request.into_inner();
  debug!("Got request = {:?}", details);
  let guard = MESSAGES.lock().unwrap();
  guard.get(details.description.as_str())
    .map(|message| {
      let metadata = json!(message.metadata).to_string();
      MessageResponder {
        payload: message.contents.value().map(|data| data.to_vec()),
        content_type: message.message_content_type()
          .map(|ct| ContentType::parse_flexible(ct.to_string().as_str()))
          .flatten()
          .unwrap_or(ContentType::Plain),
        metadata: Header::new("pact-message-metadata", BASE64.encode(metadata))
      }
    })
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
  let format = tracing_subscriber::fmt::format().pretty();
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .event_format(format)
    .init();

  let server = rocket::build()
    .mount("/", rocket::routes![messages])
    .ignite()
    .await.expect("Could not start the Rocket server");
  let shutdown = server.shutdown();
  let port = server.config().port;
  tokio::spawn(server.launch());

  V3MessageWorld::cucumber()
    .fail_on_skipped()
    .max_concurrent_scenarios(1)
    .before(move |_, _, scenario, world| {
      Box::pin(async move {
        world.scenario_id = scenario.name.clone();
        world.message_proxy_port = port;
      })
    })
    .after(|_feature, _, _scenario, _status, world| {
      Box::pin(async move {
        if let Some(world) = world {
          let mut guard = MESSAGES.lock().unwrap();
          let keys = guard.keys().cloned().collect_vec();
          for key in keys {
            if key.starts_with(world.scenario_id.as_str()) {
              guard.remove(key.as_str());
            }
          }
        }
      })
    })
    .filter_run_and_exit("pact-compatibility-suite/features/V3", |feature, _rule, _scenario| {
      feature.tags.iter().any(|tag| tag == "message")
    })
    .await;

  shutdown.notify();
}
