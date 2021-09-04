//! V4 specification Pact

use std::cmp::Ordering;
use std::collections::{BTreeMap, HashSet};
use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use itertools::EitherOrBoth::{Both, Left, Right};
use itertools::Itertools;
use maplit::btreemap;
use serde_json::{json, Value};

use crate::{Consumer, PactSpecification, Provider};
#[cfg(not(target_family = "wasm"))] use crate::file_utils::with_read_lock;
use crate::interaction::Interaction;
use crate::json_utils::json_to_string;
use crate::message_pact::MessagePact;
use crate::pact::{Pact, ReadWritePact};
use crate::PACT_RUST_VERSION;
use crate::sync_pact::RequestResponsePact;
use crate::v4::interaction::{interactions_from_json, V4Interaction};
use crate::v4::V4InteractionType;
use crate::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

/// V4 spec Struct that represents a pact between the consumer and provider of a service.
#[derive(Debug, Clone)]
pub struct V4Pact {
  /// Consumer side of the pact
  pub consumer: Consumer,
  /// Provider side of the pact
  pub provider: Provider,
  /// List of messages between the consumer and provider.
  pub interactions: Vec<Box<dyn V4Interaction>>,
  /// Metadata associated with this pact.
  pub metadata: BTreeMap<String, Value>
}

impl V4Pact {
  fn metadata_to_json(&self) -> Value {
    let mut md_map: serde_json::Map<String, Value> = self.metadata.iter()
      .map(|(k, v)| {
        let key = match k.as_str() {
          "pact-specification" => "pactSpecification".to_string(),
          "pact-rust" => "pactRust".to_string(),
          _ => k.clone()
        };
        (key, v.clone())
      })
      .collect();

    md_map.insert("pactSpecification".to_string(), json!({"version" : PactSpecification::V4.version_str()}));
    md_map.insert("pactRust".to_string(), json!({"version" : PACT_RUST_VERSION.unwrap_or("unknown")}));
    Value::Object(md_map)
  }

  /// If this Pact has any interactions of the given type
  pub fn has_interactions(&self, interaction_type: V4InteractionType) -> bool {
    self.interactions.iter().any(|interaction| interaction.v4_type() == interaction_type)
  }

  /// If this Pact has different types of interactions
  pub fn has_mixed_interactions(&self) -> bool {
    let interaction_types: HashSet<_> = self.interactions.iter().map(|i| i.v4_type()).collect();
    interaction_types.len() > 1
  }
}

impl Pact for V4Pact {
  fn consumer(&self) -> Consumer {
    self.consumer.clone()
  }

  fn provider(&self) -> Provider {
    self.provider.clone()
  }

  fn interactions(&self) -> Vec<&dyn Interaction> {
    self.interactions.iter().map(|i| i.to_super()).collect()
  }

  fn metadata(&self) -> BTreeMap<String, BTreeMap<String, String>> {
    self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), json_to_string(v))).collect())),
        _ => None
      }
    }).filter(|v| v.is_some())
      .map(|v| v.unwrap())
      .collect()
  }

  fn to_json(&self, pact_spec: PactSpecification) -> anyhow::Result<Value> {
    match pact_spec {
      PactSpecification::V4 => Ok(json!({
        "consumer": self.consumer.to_json(),
        "provider": self.provider.to_json(),
        "interactions": Value::Array(self.interactions.iter().map(|i| i.to_json()).collect()),
        "metadata": self.metadata_to_json()
      })),
      _ => if self.has_mixed_interactions() {
        Err(anyhow!("A Pact with mixed interaction types can't be downgraded to {:?}", pact_spec))
      } else if self.interactions.is_empty() || self.has_interactions(V4InteractionType::Synchronous_HTTP) {
        self.as_request_response_pact()?.to_json(pact_spec)
      } else if self.has_interactions(V4InteractionType::Asynchronous_Messages) {
        self.as_message_pact()?.to_json(pact_spec)
      } else {
        let interaction = self.interactions.first().unwrap();
        Err(anyhow!("A Pact with {} interactions can't be downgraded to {:?}", interaction.type_of(), pact_spec))
      }
    }
  }

  fn as_request_response_pact(&self) -> anyhow::Result<RequestResponsePact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_request_response())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    let metadata = self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), v.to_string())).collect())),
        _ => None
      }
    }).filter(|val| val.is_some())
      .map(|val| val.unwrap())
      .collect();
    Ok(RequestResponsePact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      interactions,
      metadata,
      specification_version: PactSpecification::V3
    })
  }

  fn as_message_pact(&self) -> anyhow::Result<MessagePact> {
    let interactions = self.interactions.iter()
      .map(|i| i.as_message())
      .filter(|i| i.is_some())
      .map(|i| i.unwrap())
      .collect();
    let metadata = self.metadata.iter().map(|(k, v)| {
      match v {
        Value::Object(map) => Some((k.clone(), map.iter()
          .map(|(k, v)| (k.clone(), v.to_string())).collect())),
        _ => None
      }
    }).filter(|val| val.is_some())
      .map(|val| val.unwrap())
      .collect();
    Ok(MessagePact {
      consumer: self.consumer.clone(),
      provider: self.provider.clone(),
      messages: interactions,
      metadata,
      specification_version: PactSpecification::V3
    })
  }

  fn as_v4_pact(&self) -> anyhow::Result<V4Pact> {
    Ok(self.clone())
  }

  fn specification_version(&self) -> PactSpecification {
    PactSpecification::V4
  }

  fn boxed(&self) -> Box<dyn Pact + Send> {
    Box::new(self.clone())
  }

  fn arced(&self) -> Arc<dyn Pact + Send> {
    Arc::new(self.clone())
  }

  fn thread_safe(&self) -> Arc<Mutex<dyn Pact + Send + Sync>> {
    Arc::new(Mutex::new(self.clone()))
  }

  fn add_interaction(&mut self, interaction: &dyn Interaction) -> anyhow::Result<()> {
    match interaction.as_v4() {
      None => Err(anyhow!("Can only add interactions that can be converted to V4 to this Pact")),
      Some(interaction) => {
        self.interactions.push(interaction);
        Ok(())
      }
    }
  }
}

impl Default for V4Pact {
  fn default() -> Self {
    V4Pact {
      consumer: Default::default(),
      provider: Default::default(),
      interactions: vec![],
      metadata: Default::default()
    }
  }
}

impl ReadWritePact for V4Pact {
  #[cfg(not(target_family = "wasm"))]
  fn read_pact(path: &Path) -> anyhow::Result<V4Pact> {
    let json = with_read_lock(path, 3, &mut |f| {
      serde_json::from_reader::<_, Value>(f).context("Failed to parse Pact JSON")
    })?;
    let metadata = meta_data_from_json(&json);
    let consumer = match json.get("consumer") {
      Some(v) => Consumer::from_json(v),
      None => Consumer { name: "consumer".into() }
    };
    let provider = match json.get("provider") {
      Some(v) => Provider::from_json(v),
      None => Provider { name: "provider".into() }
    };
    Ok(V4Pact {
      consumer,
      provider,
      interactions: interactions_from_json(&json, &*path.to_string_lossy()),
      metadata
    })
  }

  fn merge(&self, other: &dyn Pact) -> anyhow::Result<Box<dyn Pact>> {
    if self.consumer.name == other.consumer().name && self.provider.name == other.provider().name {
      Ok(Box::new(V4Pact {
        consumer: self.consumer.clone(),
        provider: self.provider.clone(),
        interactions: self.interactions.iter()
          .merge_join_by(other.interactions().iter().map(|i| i.as_v4().unwrap()), |a, b| {
            match (a.key(), b.key()) {
              (Some(key_a), Some(key_b)) => Ord::cmp(&key_a, &key_b),
              (_, _) => {
                let type_a = a.type_of();
                let type_b = b.type_of();
                let cmp = Ord::cmp(&type_a, &type_b);
                if cmp == Ordering::Equal {
                  let cmp = Ord::cmp(&a.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>(),
                                     &b.provider_states().iter().map(|p| p.name.clone()).collect::<Vec<String>>());
                  if cmp == Ordering::Equal {
                    Ord::cmp(&a.description(), &b.description())
                  } else {
                    cmp
                  }
                } else {
                  cmp
                }
              }
            }
          })
          .map(|either| {
            match either {
              Left(i) => i.clone(),
              Right(i) => i.boxed_v4(),
              Both(i, _) => i.clone()
            }
          })
          .collect(),
        metadata: self.metadata.clone()
      }))
    } else {
      Err(anyhow!("Unable to merge pacts, as they have different consumers or providers"))
    }
  }

  fn default_file_name(&self) -> String {
    format!("{}-{}.json", self.consumer.name, self.provider.name)
  }
}

impl PactJsonVerifier for V4Pact {
  fn verify_json(_path: &str, pact_json: &Value, _strict: bool, _spec_version: PactSpecification) -> Vec<PactFileVerificationResult> {
    let mut results = vec![];

    match pact_json {
      Value::Object(_values) => {

      }
      _ => results.push(PactFileVerificationResult::new("/", ResultLevel::ERROR,
                                                        &format!("Must be an Object, got {}", json_type_of(pact_json))))
    }

    results
  }
}

/// Creates a V4 Pact from the provided JSON struct
pub fn from_json(source: &str, pact_json: &Value) -> anyhow::Result<Box<dyn Pact>> {
  let metadata = meta_data_from_json(pact_json);
  let consumer = match pact_json.get("consumer") {
    Some(v) => Consumer::from_json(v),
    None => Consumer { name: "consumer".into() }
  };
  let provider = match pact_json.get("provider") {
    Some(v) => Provider::from_json(v),
    None => Provider { name: "provider".into() }
  };
  Ok(Box::new(V4Pact {
    consumer,
    provider,
    interactions: interactions_from_json(pact_json, source),
    metadata
  }))
}

fn meta_data_from_json(pact_json: &Value) -> BTreeMap<String, Value> {
  match pact_json.get("metadata") {
    Some(v) => match *v {
      Value::Object(ref obj) => obj.iter()
        .map(|(k, v)| (k.clone(), v.clone())).collect(),
      _ => btreemap!{}
    },
    None => btreemap!{}
  }
}

#[cfg(test)]
mod tests {
  use std::{env, fs, io};
  use std::fs::File;
  use std::io::Read;

  use expectest::prelude::*;
  use maplit::*;
  use serde_json::json;

  use crate::{Consumer, PACT_RUST_VERSION, PactSpecification, Provider};
  use crate::bodies::OptionalBody;
  use crate::matchingrules;
  use crate::matchingrules::MatchingRule;
  use crate::pact::{Pact, ReadWritePact, write_pact};
  use crate::provider_states::ProviderState;
  use crate::v4::async_message::AsynchronousMessage;
  use crate::v4::http_parts::{HttpRequest, HttpResponse};
  use crate::v4::V4InteractionType;
  use crate::v4::message_parts::MessageContents;
  use crate::v4::pact::{from_json, V4Pact};
  use crate::v4::sync_message::SynchronousMessages;
  use crate::v4::synch_http::SynchronousHttp;

  #[test]
  fn load_empty_pact() {
    let pact_json = json!({});
    let pact = from_json("", &pact_json).unwrap();
    expect!(pact.provider().name).to(be_equal_to("provider"));
    expect!(pact.consumer().name).to(be_equal_to("consumer"));
    expect!(pact.interactions().iter()).to(have_count(0));
    expect!(pact.metadata().iter()).to(have_count(0));
    expect!(pact.specification_version()).to(be_equal_to(PactSpecification::V4));
  }

  #[test]
  fn load_basic_pact() {
    let pact_json = json!({
    "provider": {
        "name": "Alice Service"
    },
    "consumer": {
        "name": "Consumer"
    },
    "interactions": [
      {
        "type": "Synchronous/HTTP",
        "description": "a retrieve Mallory request",
        "request": {
          "method": "GET",
          "path": "/mallory",
          "query": "name=ron&status=good"
        },
        "response": {
          "status": 200,
          "headers": {
            "Content-Type": "text/html"
          },
          "body": {
            "content": "\"That is some good Mallory.\""
          }
        }
      }
    ]
  });
    let pact = from_json("", &pact_json).unwrap();
    expect!(&pact.provider().name).to(be_equal_to("Alice Service"));
    expect!(&pact.consumer().name).to(be_equal_to("Consumer"));
    expect!(pact.interactions().iter()).to(have_count(1));
    let interaction = pact.interactions()[0];
    expect!(interaction.description()).to(be_equal_to("a retrieve Mallory request"));
    expect!(interaction.provider_states().iter()).to(be_empty());
    expect!(pact.specification_version()).to(be_equal_to(PactSpecification::V4));
    expect!(pact.metadata().iter()).to(have_count(0));

    let v4pact = pact.as_v4_pact().unwrap();
    let interaction = &v4pact.interactions[0];
    expect!(interaction.pending()).to(be_false());
    match interaction.as_v4_http() {
      Some(SynchronousHttp { request, response, pending, .. }) => {
        expect!(request).to(be_equal_to(HttpRequest {
          method: "GET".into(),
          path: "/mallory".into(),
          query: Some(hashmap!{ "name".to_string() => vec!["ron".to_string()], "status".to_string() => vec!["good".to_string()] }),
          headers: None,
          body: OptionalBody::Missing,
          .. HttpRequest::default()
        }));
        expect!(response).to(be_equal_to(HttpResponse {
          status: 200,
          headers: Some(hashmap!{ "Content-Type".to_string() => vec!["text/html".to_string()] }),
          body: OptionalBody::Present("\"That is some good Mallory.\"".into(), Some("text/html".into())),
          .. HttpResponse::default()
        }));
        expect!(pending).to(be_false());
      }
      _ => panic!("Was expecting an HTTP pact")
    }
  }

  #[test]
  fn load_pact_encoded_query_string() {
    let pact_json = json!({
      "provider" : {
        "name" : "test_provider"
      },
      "consumer" : {
        "name" : "test_consumer"
      },
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "request" : {
          "query" : "datetime=2011-12-03T10%3A15%3A30%2B01%3A00&description=hello+world%21"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {
        "pactSpecification" : {
          "version" : "4.0"
        }
      }
    });
    let pact = from_json("", &pact_json).unwrap();

    expect!(pact.interactions().iter()).to(have_count(1));

    let v4pact = pact.as_v4_pact().unwrap();
    match v4pact.interactions[0].as_v4_http() {
      Some(SynchronousHttp { request, .. }) => {
        expect!(&request.query).to(be_equal_to(
          &Some(hashmap!{ "datetime".to_string() => vec!["2011-12-03T10:15:30+01:00".to_string()],
            "description".to_string() => vec!["hello world!".to_string()] })));
      }
      _ => panic!("Was expecting an HTTP pact")
    }
  }

  #[test]
  fn load_pact_converts_methods_to_uppercase() {
    let pact_json = json!({
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    });
    let pact = from_json("", &pact_json).unwrap();
    expect!(pact.interactions().iter()).to(have_count(1));

    let v4pact = pact.as_v4_pact().unwrap();
    match v4pact.interactions[0].as_v4_http() {
      Some(SynchronousHttp { request, .. }) => {
        expect!(&request.method).to(be_equal_to("GET"));
      }
      _ => panic!("Was expecting an HTTP pact")
    }
  }

  fn read_pact_file(file: &str) -> io::Result<String> {
    let mut f = File::open(file)?;
    let mut buffer = String::new();
    f.read_to_string(&mut buffer)?;
    Ok(buffer)
  }

  #[test]
  fn write_pact_test() {
    let pact = V4Pact { consumer: Consumer { name: "write_pact_test_consumer".to_string() },
      provider: Provider { name: "write_pact_test_provider".to_string() },
      interactions: vec![
        Box::new(SynchronousHttp {
          id: None,
          key: None,
          description: "Test Interaction".to_string(),
          provider_states: vec![ProviderState { name: "Good state to be in".to_string(), params: hashmap!{} }],
          .. Default::default()
        })
      ],
      .. V4Pact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "296966511eff169a",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_synchronous_message_pact_test() {
    let pact = V4Pact {
      consumer: Consumer { name: "write_pact_test_consumer".into() },
      provider: Provider { name: "write_pact_test_provider".into() },
      interactions: vec![
        Box::new(SynchronousMessages {
          id: None,
          key: None,
          description: "Test Interaction".into(),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          request: MessageContents { contents: "\"this is a message\"".into(), .. MessageContents::default() },
          response: vec![MessageContents { contents: "\"this is a response\"".into(), .. MessageContents::default() }],
          .. Default::default()
        })
      ],
      .. V4Pact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "b341297869a4287d",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "contents": {{
          "content": "\"this is a message\"",
          "contentType": "*/*",
          "encoded": false
        }}
      }},
      "response": [
        {{
          "contents": {{
            "content": "\"this is a response\"",
            "contentType": "*/*",
            "encoded": false
          }}
        }}
      ],
      "type": "Synchronous/Messages"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_test_should_merge_pacts() {
    let pact = V4Pact {
      consumer: Consumer { name: "merge_consumer".into() },
      provider: Provider { name: "merge_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction 2".into(),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          .. SynchronousHttp::default()
        })
      ],
      metadata: btreemap!{}
    };
    let pact2 = V4Pact {
      consumer: Consumer { name: "merge_consumer".into() },
      provider: Provider { name: "merge_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          .. SynchronousHttp::default()
        })
      ],
      metadata: btreemap!{}
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V4, true);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V4, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "merge_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "296966511eff169a",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }},
    {{
      "description": "Test Interaction 2",
      "key": "d3e13a43bc0744ac",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "merge_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_test_should_overwrite_pact_with_same_key() {
    let pact = V4Pact {
      consumer: Consumer { name: "write_pact_test_consumer".into() },
      provider: Provider { name: "write_pact_test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          .. SynchronousHttp::default()
        })
      ],
      metadata: btreemap!{}
    };
    let pact2 = V4Pact {
      consumer: Consumer { name: "write_pact_test_consumer".into() },
      provider: Provider { name: "write_pact_test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          response: HttpResponse { status: 400, .. HttpResponse::default() },
          .. SynchronousHttp::default()
        })
      ],
      metadata: btreemap!{}
    };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), dir.as_path(), PactSpecification::V4, true);
    let result2 = write_pact(pact2.boxed(), dir.as_path(), PactSpecification::V4, false);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(result2).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "key": "1234567890",
      "pending": false,
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 400
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn pact_merge_does_not_merge_different_consumers() {
    let pact = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{}
    };
    let pact2 = V4Pact { consumer: Consumer { name: "test_consumer2".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{}
    };
    expect!(pact.merge(&pact2)).to(be_err());
  }

  #[test]
  fn pact_merge_does_not_merge_different_providers() {
    let pact = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider".to_string() },
      interactions: vec![],
      metadata: btreemap!{}
    };
    let pact2 = V4Pact { consumer: Consumer { name: "test_consumer".to_string() },
      provider: Provider { name: "test_provider2".to_string() },
      interactions: vec![],
      metadata: btreemap!{}
    };
    expect!(pact.merge(&pact2)).to(be_err());
  }

// #[test]
// fn pact_merge_does_not_merge_where_there_are_conflicting_interactions() {
//   let pact = RequestResponsePact { consumer: Consumer { name: ".to_string()est_consumer") },
//     provider: Provider { name: ".to_string()est_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: ".to_string()est Interaction"),
//         provider_states: vec![ProviderState { name: ".to_string()ood state to be in"), params: hashmap!{} }],
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   let pact2 = RequestResponsePact { consumer: Consumer { name: ".to_string()est_consumer") },
//     provider: Provider { name: ".to_string()est_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: ".to_string()est Interaction"),
//         provider_states: vec![ProviderState { name: ".to_string()ood state to be in"), params: hashmap!{} }],
//         request: Request { path: ".to_string()other"), .. Request::default() },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     metadata: btreemap!{},
//     specification_version: PactSpecification::V1_1
//   };
//   expect!(pact.merge(&pact2)).to(be_err());
// }

  #[test]
  fn pact_merge_removes_duplicates() {
    let pact = V4Pact {
      consumer: Consumer { name: "test_consumer".into() },
      provider: Provider { name: "test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          response: HttpResponse { status: 400, .. HttpResponse::default() },
          .. SynchronousHttp::default()
        })
      ],
      .. V4Pact::default()
    };
    let pact2 = V4Pact {
      consumer: Consumer { name: "test_consumer".into() },
      provider: Provider { name: "test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          response: HttpResponse { status: 400, .. HttpResponse::default() },
          .. SynchronousHttp::default()
        }),
        Box::new(SynchronousHttp {
          description: "Test Interaction 2".into(),
          key: Some("1234567891".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          response: HttpResponse { status: 400, .. HttpResponse::default() },
          .. SynchronousHttp::default()
        })
      ],
      .. V4Pact::default()
    };

    let merged_pact = pact.merge(&pact2);
    expect!(merged_pact.unwrap().interactions().len()).to(be_equal_to(2));

    let merged_pact2 = pact.merge(&pact.clone());
    expect!(merged_pact2.unwrap().interactions().len()).to(be_equal_to(1));
  }

  #[test]
  fn write_v2_pact_test_with_matchers() {
    let pact = V4Pact {
      consumer: Consumer { name: "write_pact_test_consumer".into() },
      provider: Provider { name: "write_pact_test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          request: HttpRequest {
            matching_rules: matchingrules!{
            "body" => {
              "$" => [ MatchingRule::Type ]
            }
          },
            .. HttpRequest::default()
          },
          .. SynchronousHttp::default()
        })
      ],
      .. V4Pact::default() };

    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), &dir, PactSpecification::V2, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerState": "Good state to be in",
      "request": {{
        "matchingRules": {{
          "$.body": {{
            "match": "type"
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "2.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn write_pact_v3_test_with_matchers() {
    let pact = V4Pact { consumer: Consumer { name: "write_pact_test_consumer_v3".to_string() },
      provider: Provider { name: "write_pact_test_provider_v3".to_string() },
      interactions: vec![
        Box::new(SynchronousHttp {
          description: "Test Interaction".into(),
          key: Some("1234567890".into()),
          provider_states: vec![ProviderState { name: "Good state to be in".into(), params: hashmap!{} }],
          request: HttpRequest {
            matching_rules: matchingrules!{
            "body" => {
              "$" => [ MatchingRule::Type ]
            },
            "header" => {
              "HEADER_A" => [ MatchingRule::Include("ValA".to_string()), MatchingRule::Include("ValB".to_string()) ]
            }
          },
            .. HttpRequest::default()
          },
          .. SynchronousHttp::default()
        })
      ],
      .. V4Pact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), &dir, PactSpecification::V3, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or("".to_string());
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_pact_test_consumer_v3"
  }},
  "interactions": [
    {{
      "description": "Test Interaction",
      "providerStates": [
        {{
          "name": "Good state to be in"
        }}
      ],
      "request": {{
        "matchingRules": {{
          "body": {{
            "$": {{
              "combine": "AND",
              "matchers": [
                {{
                  "match": "type"
                }}
              ]
            }}
          }},
          "header": {{
            "HEADER_A": {{
              "combine": "AND",
              "matchers": [
                {{
                  "match": "include",
                  "value": "ValA"
                }},
                {{
                  "match": "include",
                  "value": "ValB"
                }}
              ]
            }}
          }}
        }},
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }}
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "3.0.0"
    }}
  }},
  "provider": {{
    "name": "write_pact_test_provider_v3"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
  }

// #[test]
// fn write_v3_pact_test() {
//   let pact = RequestResponsePact { consumer: Consumer { name: ".to_string()rite_pact_test_consumer") },
//     provider: Provider { name: ".to_string()rite_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: ".to_string()est Interaction"),
//         provider_states: vec![ProviderState { name: ".to_string()ood state to be in"), params: hashmap!{} }],
//         request: Request {
//           query: Some(hashmap!{
//                         ".to_string()") => vec![".to_string()"), ".to_string()"), ".to_string()")],
//                         ".to_string()") => vec![".to_string()ill"), ".to_string()ob")],
//                     }),
//           .. Request::default()
//         },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default() };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V3);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(".to_string()));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "write_pact_test_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction",
//       "providerStates": [
//         {{
//           "name": "Good state to be in"
//         }}
//       ],
//       "request": {{
//         "method": "GET",
//         "path": "/",
//         "query": {{
//           "a": [
//             "1",
//             "2",
//             "3"
//           ],
//           "b": [
//             "bill",
//             "bob"
//           ]
//         }}
//       }},
//       "response": {{
//         "status": 200
//       }}
//     }}
//   ],
//   "metadata": {{
//     "pactRust": {{
//       "version": "{}"
//     }},
//     "pactSpecification": {{
//       "version": "3.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }
//
// #[test]
// fn write_pact_test_with_generators() {
//   let pact = RequestResponsePact { consumer: Consumer { name: ".to_string()rite_pact_test_consumer") },
//     provider: Provider { name: ".to_string()rite_pact_test_provider") },
//     interactions: vec![
//       RequestResponseInteraction {
//         description: ".to_string()est Interaction with generators"),
//         provider_states: vec![ProviderState { name: ".to_string()ood state to be in"), params: hashmap!{} }],
//         request: Request {
//           generators: generators!{
//                         "BODY" => {
//                           "$" => Generator::RandomInt(1, 10)
//                         },
//                         "HEADER" => {
//                           "A" => Generator::RandomString(20)
//                         }
//                     },
//           .. Request::default()
//         },
//         .. RequestResponseInteraction::default()
//       }
//     ],
//     .. RequestResponsePact::default() };
//   let mut dir = env::temp_dir();
//   let x = rand::random::<u16>();
//   dir.push(format!("pact_test_{}", x));
//   dir.push(pact.default_file_name());
//
//   let result = pact.write_pact(dir.as_path(), PactSpecification::V3);
//
//   let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or(".to_string()));
//   fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());
//
//   expect!(result).to(be_ok());
//   expect!(pact_file).to(be_equal_to(format!(r#"{{
//   "consumer": {{
//     "name": "write_pact_test_consumer"
//   }},
//   "interactions": [
//     {{
//       "description": "Test Interaction with generators",
//       "providerStates": [
//         {{
//           "name": "Good state to be in"
//         }}
//       ],
//       "request": {{
//         "generators": {{
//           "body": {{
//             "$": {{
//               "max": 10,
//               "min": 1,
//               "type": "RandomInt"
//             }}
//           }},
//           "header": {{
//             "A": {{
//               "size": 20,
//               "type": "RandomString"
//             }}
//           }}
//         }},
//         "method": "GET",
//         "path": "/"
//       }},
//       "response": {{
//         "status": 200
//       }}
//     }}
//   ],
//   "metadata": {{
//     "pactRust": {{
//       "version": "{}"
//     }},
//     "pactSpecification": {{
//       "version": "3.0.0"
//     }}
//   }},
//   "provider": {{
//     "name": "write_pact_test_provider"
//   }}
// }}"#, super::VERSION.unwrap())));
// }

  #[test]
  fn write_v4_pact_test_with_comments() {
    let pact = V4Pact { consumer: Consumer { name: "write_v4pact_test_consumer".to_string() },
      provider: Provider { name: "write_v4pact_test_provider".into() },
      interactions: vec![
        Box::new(SynchronousHttp {
          id: None,
          key: None,
          description: "Test Interaction".into(),
          comments: hashmap! {
          "text".to_string() => json!([
            "This allows me to specify just a bit more information about the interaction",
            "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
            "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
          ]),
          "testname".to_string() => json!("example_test.groovy")
        },
          .. Default::default()
        })
      ],
      .. V4Pact::default() };
    let mut dir = env::temp_dir();
    let x = rand::random::<u16>();
    dir.push(format!("pact_test_{}", x));
    dir.push(pact.default_file_name());

    let result = write_pact(pact.boxed(), &dir, PactSpecification::V4, true);

    let pact_file = read_pact_file(dir.as_path().to_str().unwrap()).unwrap_or_default();
    fs::remove_dir_all(dir.parent().unwrap()).unwrap_or(());

    expect!(result).to(be_ok());
    expect!(pact_file).to(be_equal_to(format!(r#"{{
  "consumer": {{
    "name": "write_v4pact_test_consumer"
  }},
  "interactions": [
    {{
      "comments": {{
        "testname": "example_test.groovy",
        "text": [
          "This allows me to specify just a bit more information about the interaction",
          "It has no functional impact, but can be displayed in the broker HTML page, and potentially in the test output",
          "It could even contain the name of the running test on the consumer side to help marry the interactions back to the test case"
        ]
      }},
      "description": "Test Interaction",
      "key": "7e202f73d7d6d607",
      "pending": false,
      "request": {{
        "method": "GET",
        "path": "/"
      }},
      "response": {{
        "status": 200
      }},
      "type": "Synchronous/HTTP"
    }}
  ],
  "metadata": {{
    "pactRust": {{
      "version": "{}"
    }},
    "pactSpecification": {{
      "version": "4.0"
    }}
  }},
  "provider": {{
    "name": "write_v4pact_test_provider"
  }}
}}"#, super::PACT_RUST_VERSION.unwrap())));
  }

  #[test]
  fn has_interactions_test() {
    let pact1 = V4Pact {
      interactions: vec![],
      .. V4Pact::default() };
    let pact2 = V4Pact {
      interactions: vec![
        Box::new(SynchronousHttp::default())
      ],
      .. V4Pact::default() };
    let pact3 = V4Pact {
      interactions: vec![
        Box::new(AsynchronousMessage::default())
      ],
      .. V4Pact::default() };
    let pact4 = V4Pact {
      interactions: vec![
        Box::new(SynchronousMessages::default())
      ],
      .. V4Pact::default() };
    let pact5 = V4Pact {
      interactions: vec![
        Box::new(SynchronousHttp::default()),
        Box::new(SynchronousMessages::default())
      ],
      .. V4Pact::default() };

    expect!(pact1.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
    expect!(pact1.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
    expect!(pact1.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

    expect!(pact2.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_true());
    expect!(pact2.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
    expect!(pact2.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

    expect!(pact3.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
    expect!(pact3.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_true());
    expect!(pact3.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_false());

    expect!(pact4.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_false());
    expect!(pact4.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
    expect!(pact4.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_true());

    expect!(pact5.has_interactions(V4InteractionType::Synchronous_HTTP)).to(be_true());
    expect!(pact5.has_interactions(V4InteractionType::Asynchronous_Messages)).to(be_false());
    expect!(pact5.has_interactions(V4InteractionType::Synchronous_Messages)).to(be_true());
  }

  #[test]
  fn has_mixed_interactions_test() {
    let pact1 = V4Pact {
      interactions: vec![],
      .. V4Pact::default() };
    let pact2 = V4Pact {
      interactions: vec![
        Box::new(SynchronousHttp::default())
      ],
      .. V4Pact::default() };
    let pact3 = V4Pact {
      interactions: vec![
        Box::new(AsynchronousMessage::default())
      ],
      .. V4Pact::default() };
    let pact4 = V4Pact {
      interactions: vec![
        Box::new(SynchronousMessages::default())
      ],
      .. V4Pact::default() };
    let pact5 = V4Pact {
      interactions: vec![
        Box::new(SynchronousHttp::default()),
        Box::new(SynchronousMessages::default())
      ],
      .. V4Pact::default() };

    expect!(pact1.has_mixed_interactions()).to(be_false());
    expect!(pact2.has_mixed_interactions()).to(be_false());
    expect!(pact3.has_mixed_interactions()).to(be_false());
    expect!(pact4.has_mixed_interactions()).to(be_false());
    expect!(pact5.has_mixed_interactions()).to(be_true());
  }

  #[test]
  fn load_pending_pact() {
    let pact_json = json!({
      "interactions" : [ {
        "type": "Synchronous/HTTP",
        "description" : "test interaction",
        "pending": true,
        "request" : {
          "method" : "get"
        },
        "response" : {
          "status" : 200
        }
      } ],
      "metadata" : {}
    });
    let pact = from_json("", &pact_json).unwrap();
    expect!(pact.interactions().iter()).to(have_count(1));

    let v4pact = pact.as_v4_pact().unwrap();
    let interaction = &v4pact.interactions[0];
    expect(interaction.pending()).to(be_true());
    match interaction.as_v4_http() {
      Some(SynchronousHttp { request, .. }) => {
        expect!(&request.method).to(be_equal_to("GET"));
      }
      _ => panic!("Was expecting an HTTP pact")
    }
  }
}
