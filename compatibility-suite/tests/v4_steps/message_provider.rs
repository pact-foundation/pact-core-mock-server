use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use bytes::Bytes;
use cucumber::gherkin::Step;

use cucumber::given;
use lazy_static::lazy_static;
use maplit::hashmap;
use pact_models::bodies::OptionalBody;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::content_types::{ContentType, JSON, XML};
use pact_models::pact::Pact;
use pact_models::v4::async_message::AsynchronousMessage;
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::message_parts::MessageContents;
use pact_models::v4::pact::V4Pact;
use pact_models::xml_utils::parse_bytes;
use serde_json::json;

use pact_verifier::{PactSource, ProviderInfo, ProviderTransport};
use crate::shared_steps::{determine_content_type, element_text};

use crate::v4_steps::V4World;

lazy_static!{
  pub static ref MESSAGES: Arc<Mutex<HashMap<String, AsynchronousMessage>>> = Arc::new(Mutex::new(hashmap![]));
}

fn setup_body(body: &String, message: &mut MessageContents) {
  if !body.is_empty() {
    if body.starts_with("JSON:") {
      message.metadata.insert("contentType".to_string(), json!("application/json"));
      message.contents = OptionalBody::Present(Bytes::from(body.strip_prefix("JSON:").unwrap_or(body).trim().to_string()),
                                                        Some(JSON.clone()), None);
    } else if body.starts_with("XML:") {
      message.metadata.insert("contentType".to_string(), json!("application/xml"));
      message.contents = OptionalBody::Present(Bytes::from(body.strip_prefix("XML:").unwrap_or(body).trim().to_string()),
                                                        Some(XML.clone()), None);
    } else if body.starts_with("file:") {
      if body.ends_with("-body.xml") {
        let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
        let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
          .expect(format!("could not load fixture '{}'", body).as_str());
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
          .expect(format!("could not read fixture '{}'", body).as_str());
        let fixture = parse_bytes(buffer.as_slice())
          .expect(format!("could not parse fixture as XML: '{}'", body).as_str());
        let root = fixture.as_document().root();
        let body_node = root.children().iter().find_map(|n| n.element()).unwrap();
        let content_type = element_text(body_node, "contentType").unwrap_or("text/plain".to_string());
        message.metadata.insert("contentType".to_string(), json!(content_type));
        message.contents = OptionalBody::Present(Bytes::from(element_text(body_node, "contents").unwrap_or_default()),
                                                          ContentType::parse(content_type.as_str()).ok(), None);
      } else {
        let content_type = determine_content_type(body, message);
        message.metadata.insert("contentType".to_string(), json!(content_type.to_string()));

        let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
        let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
          .expect(format!("could not load fixture '{}'", body).as_str());
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)
          .expect(format!("could not read fixture '{}'", body).as_str());
        message.contents = OptionalBody::Present(Bytes::from(buffer),
                                                          Some(content_type), None);
      }
    } else {
      let content_type = determine_content_type(body, message);
      message.metadata.insert("contentType".to_string(), json!(content_type.to_string()));
      let body = Bytes::from(body.clone());
      message.contents = OptionalBody::Present(body, Some(content_type), None);
    }
  }
}

#[given(expr = "a provider is started that can generate the {string} message with {string}")]
#[allow(deprecated)]
fn a_provider_is_started_that_can_generate_the_message(
  world: &mut V4World,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = AsynchronousMessage {
    description: key.clone(),
    .. AsynchronousMessage::default()
  };
  setup_body(&fixture, &mut message.contents);

  {
    let mut guard = MESSAGES.lock().unwrap();
    guard.insert(key, message);
  }

  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "localhost".to_string(),
    port: Some(world.message_proxy_port),
    transports: vec![ProviderTransport {
      port: Some(world.message_proxy_port),
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };
}

#[given(expr = "a Pact file for {string}:{string} is to be verified, but is marked pending")]
fn a_pact_file_for_is_to_be_verified_but_is_marked_pending(
  world: &mut V4World,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = AsynchronousMessage {
    description: key.clone(),
    pending: true,
    .. AsynchronousMessage::default()
  };
  setup_body(&fixture, &mut message.contents);

  let pact = V4Pact {
    consumer: Consumer { name: format!("c_{}", name) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ message.boxed_v4() ],
    .. V4Pact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V4).unwrap().to_string()));
}

#[given(expr = "a Pact file for {string}:{string} is to be verified with the following comments:")]
fn a_pact_file_for_is_to_be_verified_with_the_following_comments(
  world: &mut V4World,
  step: &Step,
  name: String,
  fixture: String
) {
  let key = format!("{}:{}", world.scenario_id, name);
  let mut message = AsynchronousMessage {
    description: key.clone(),
    .. AsynchronousMessage::default()
  };
  setup_body(&fixture, &mut message.contents);

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for row in table.rows.iter().skip(1) {
      let data: HashMap<String, String> = row.iter().enumerate().map(|(i, v)| (headers[i].clone(), v.clone())).collect();
      match data["type"].as_str() {
        "text" => {
          match message.comments.entry("text".to_string()) {
            Entry::Occupied(mut entry) => {
              let array = entry.get_mut().as_array_mut().unwrap();
              array.push(json!(data["comment"]));
            }
            Entry::Vacant(entry) => {
              entry.insert(json!([ data["comment"] ]));
            }
          }
        }
        "testname" => {
          message.comments.insert("testname".to_string(), json!(data["comment"]));
        },
        _ => {}
      }
    }
  }

  let pact = V4Pact {
    consumer: Consumer { name: format!("c_{}", name) },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ message.boxed_v4() ],
    .. V4Pact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V4).unwrap().to_string()));
}
