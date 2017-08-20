//! `provider_states` module contains all the logic for dealing with provider states.
//! See http://docs.pact.io/documentation/provider_states.html for more info on provider states.

use std::collections::HashMap;
use serde_json::Value;
use std::hash::{Hash, Hasher};
use std::cmp::Eq;

/// Struct that encapsulates all the info about a provider state
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProviderState {
    /// Description of this provider state
    pub name: String,
    /// Provider state parameters as key value pairs
    pub params: HashMap<String, Value>
}

impl ProviderState {

    /// Creates a default state with the given name
    pub fn default(name: &String) -> ProviderState {
        ProviderState {
            name: name.clone(),
            params: hashmap!{}
        }
    }

    /// Constructs a provider state from the `Json` struct
    pub fn from_json_v3(pact_json: &Value) -> ProviderState {
        let state = match pact_json.get("name") {
            Some(v) => match *v {
                Value::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => {
                warn!("Provider state does not have a 'name' field");
                s!("unknown provider states")
            }
        };
        let params = match pact_json.get("params") {
            Some(v) => match *v {
                Value::Object(ref map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                _ => {
                    warn!("Provider state parameters must be a map");
                    hashmap!{}
                }
            },
            None => hashmap!{}
        };
        ProviderState{
            name: state,
            params: params
        }
    }

    /// Constructs a list of provider states from the `Json` struct
    pub fn from_json(pact_json: &Value) -> Vec<ProviderState> {
        match pact_json.get("providerStates") {
            Some(v) => match *v {
                Value::Array(ref a) => a.iter().map(|i| ProviderState::from_json_v3(i)).collect(),
                _ => vec![]
            },
            None => match pact_json.get("providerState").or(pact_json.get("provider_state")) {
                Some(v) => match *v {
                    Value::String(ref s) => if s.is_empty() {
                        vec![]
                    } else {
                        vec![ProviderState{ name: s.clone(), params: hashmap!{} }]
                    },
                    Value::Null => vec![],
                    _ => vec![ProviderState{ name: v.to_string(), params: hashmap!{} }]
                },
                None => vec![]
            }
        }
    }

    /// Converts this provider state into a JSON structure
    pub fn to_json(&self) -> Value {
        let mut value = json!({
            "name": Value::String(self.name.clone())
        });
        if !self.params.is_empty() {
            let mut map = value.as_object_mut().unwrap();
            map.insert(s!("params"), Value::Object(
                self.params.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
        }
        value
    }

}

impl Hash for ProviderState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in self.params.clone() {
            k.hash(state);
            match v {
              Value::Number(n) => if n.is_u64() {
                n.as_u64().unwrap().hash(state)
              } else if n.is_i64() {
                n.as_i64().unwrap().hash(state)
              } else if n.is_f64() {
                n.as_f64().unwrap().to_string().hash(state)
              },
              Value::String(s) => s.hash(state),
              Value::Bool(b) => b.hash(state),
              _ => ()
            }
        }
    }
}

impl Eq for ProviderState {

}

#[cfg(test)]
mod tests {
    use super::*;
    use expectest::prelude::*;
    use serde_json;
    use serde_json::Value;

    #[test]
    fn defaults_to_v3_pact_provider_states() {
        let json = r#"{
            "providerStates": [
              {
                "name": "test state",
                "params": { "name": "Testy" }
              },
              {
                "name": "test state 2",
                "params": { "name": "Testy2" }
              }
            ],
            "description" : "test interaction"
        }"#;
        let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
        expect!(provider_states.iter()).to(have_count(2));
        expect!(&provider_states[0]).to(be_equal_to(&ProviderState {
            name: s!("test state"),
            params: hashmap!{ s!("name") => Value::String(s!("Testy")) }
        }));
        expect!(&provider_states[1]).to(be_equal_to(&ProviderState {
            name: s!("test state 2"),
            params: hashmap!{ s!("name") => Value::String(s!("Testy2")) }
        }));
    }

    #[test]
    fn falls_back_to_v2_pact_provider_state() {
        let json = r#"{
            "providerState": "test state",
            "description" : "test interaction"
        }"#;
        let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
        expect!(provider_states.iter()).to(have_count(1));
        expect!(&provider_states[0]).to(be_equal_to(&ProviderState {
            name: s!("test state"),
            params: hashmap!{}
        }));
    }

    #[test]
    fn pact_with_no_provider_states() {
        let json = r#"{
            "description" : "test interaction"
        }"#;
        let provider_states = ProviderState::from_json(&serde_json::from_str(json).unwrap());
        expect!(provider_states.iter()).to(be_empty());
    }

}
