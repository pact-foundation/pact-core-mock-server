//! V4 Pact interaction

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;

use anyhow::anyhow;
use log::warn;
use serde_json::Value;

use crate::interaction::Interaction;
use crate::json_utils::json_to_string;
use crate::v4::async_message::AsynchronousMessage;
use crate::v4::sync_message::SynchronousMessages;
use crate::v4::synch_http::SynchronousHttp;
use crate::v4::V4InteractionType;

/// V4 Interaction trait
pub trait V4Interaction: Interaction + Send + Sync {
  /// Convert the interaction to a JSON Value
  fn to_json(&self) -> Value;

  /// Convert the interaction to its super trait
  fn to_super(&self) -> &dyn Interaction;

  /// Key for this interaction
  fn key(&self) -> Option<String>;

  /// Clones this interaction and wraps it in a box
  fn boxed_v4(&self) -> Box<dyn V4Interaction>;

  /// Annotations and comments associated with this interaction
  fn comments(&self) -> HashMap<String, Value>;

  /// Mutable access to the annotations and comments associated with this interaction
  fn comments_mut(&mut self) -> &mut HashMap<String, Value>;

  /// Type of this V4 interaction
  fn v4_type(&self) -> V4InteractionType;
}

impl Display for dyn V4Interaction {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    if let Some(i) = self.as_v4_http() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_async_message() {
      std::fmt::Display::fmt(&i, f)
    } else if let Some(i) = self.as_v4_sync_message() {
      std::fmt::Display::fmt(&i, f)
    } else {
      Err(fmt::Error)
    }
  }
}

impl Clone for Box<dyn V4Interaction> {
  fn clone(&self) -> Self {
    if let Some(http) = self.as_v4_http() {
      Box::new(http)
    } else if let Some(message) = self.as_v4_async_message() {
      Box::new(message)
    } else if let Some(message) = self.as_v4_sync_message() {
      Box::new(message)
    } else {
      panic!("Internal Error - Tried to clone an interaction that was not valid")
    }
  }
}

impl PartialEq for Box<dyn V4Interaction> {
  fn eq(&self, other: &Self) -> bool {
    if let Some(http) = self.as_v4_http() {
      if let Some(other) = other.as_v4_http() {
        http == other
      } else {
        false
      }
    } else if let Some(message) = self.as_v4_async_message() {
      if let Some(other) = other.as_v4_async_message() {
        message == other
      } else {
        false
      }
    } else if let Some(message) = self.as_v4_sync_message() {
      if let Some(other) = other.as_v4_sync_message() {
        message == other
      } else {
        false
      }
    } else {
      false
    }
  }
}

/// Load V4 format interactions from JSON struct
pub fn interactions_from_json(json: &Value, source: &str) -> Vec<Box<dyn V4Interaction>> {
  match json.get("interactions") {
    Some(v) => match *v {
      Value::Array(ref array) => array.iter().enumerate().map(|(index, ijson)| {
        interaction_from_json(source, index, ijson).ok()
      }).filter(|i| i.is_some())
        .map(|i| i.unwrap())
        .collect(),
      _ => vec![]
    },
    None => vec![]
  }
}

/// Create an interaction from a JSON struct
pub fn interaction_from_json(source: &str, index: usize, ijson: &Value) -> anyhow::Result<Box<dyn V4Interaction>> {
  match ijson.get("type") {
    Some(i_type) => match V4InteractionType::from_str(json_to_string(i_type).as_str()) {
      Ok(i_type) => {
        match i_type {
          V4InteractionType::Synchronous_HTTP => SynchronousHttp::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Asynchronous_Messages => AsynchronousMessage::from_json(ijson, index).map(|i| i.boxed_v4()),
          V4InteractionType::Synchronous_Messages => SynchronousMessages::from_json(ijson, index).map(|i| i.boxed_v4())
        }
      },
      Err(_) => {
        warn!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source);
        Err(anyhow!("Interaction {} has an incorrect type attribute '{}'. It will be ignored. Source: {}", index, i_type, source))
      }
    },
    None => {
      warn!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source);
      Err(anyhow!("Interaction {} has no type attribute. It will be ignored. Source: {}", index, source))
    }
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use maplit::hashmap;
  use serde_json::json;

  use crate::provider_states::ProviderState;
  use crate::v4::interaction::interaction_from_json;

  #[test]
  fn loading_interaction_from_json() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP",
      "description": "String",
      "providerStates": [{ "name": "provider state" }]
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.description()).to(be_equal_to("String"));
    expect!(interaction.provider_states()).to(be_equal_to(vec![
      ProviderState { name: "provider state".into(), params: hashmap!{} } ]));
  }

  #[test]
  fn defaults_to_number_if_no_description() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP"
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.description()).to(be_equal_to("Interaction 0"));
  }

  #[test]
  fn defaults_to_empty_if_no_provider_state() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP"
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.provider_states().iter()).to(be_empty());
  }

  #[test]
  fn defaults_to_none_if_provider_state_null() {
    let interaction_json = json!({
      "type": "Synchronous/HTTP",
      "description": "String",
      "providerStates": null
    });
    let interaction = interaction_from_json("", 0, &interaction_json).unwrap();
    expect!(interaction.provider_states().iter()).to(be_empty());
  }

  #[test]
  fn interaction_from_json_sets_the_id_if_loaded_from_broker() {
    let json = json!({
      "type": "Synchronous/HTTP",
      "_id": "123456789",
      "description": "Test Interaction",
      "request": {
        "method": "GET",
        "path": "/"
      },
      "response": {
        "status": 200
      }
    });
    let interaction = interaction_from_json("", 0, &json).unwrap();
    expect!(interaction.id()).to(be_some().value("123456789".to_string()));
  }

  // TODO: implement these tests
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_different_descriptions() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction 2"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_different_provider_states() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Bad state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_do_not_conflict_if_they_have_the_same_requests_and_responses() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to(be_empty());
  // }
  //
  // #[test]
  // fn interactions_conflict_if_they_have_different_requests() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     request: Request { method: s!("POST"), .. Request::default() },
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  // }
  //
  // #[test]
  // fn interactions_conflict_if_they_have_different_responses() {
  //   let interaction1 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     .. RequestResponseInteraction::default()
  //   };
  //   let interaction2 = RequestResponseInteraction {
  //     description: s!("Test Interaction"),
  //     provider_states: vec![ProviderState { name: s!("Good state to be in"), params: hashmap!{} }],
  //     response: Response { status: 400, .. Response::default() },
  //     .. RequestResponseInteraction::default()
  //   };
  //   expect!(interaction1.conflicts_with(&interaction2).iter()).to_not(be_empty());
  // }
}
