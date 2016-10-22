//! `provider_states` module contains all the logic for dealing with provider states.
//! See http://docs.pact.io/documentation/provider_states.html for more info on provider states.

use std::collections::HashMap;
use rustc_serialize::json::Json;
use std::hash::{Hash, Hasher};
use std::cmp::Eq;

/// Struct that encapsulates all the info about a provider state
#[derive(Debug, Clone, PartialEq)]
pub struct ProviderState {
    /// Description of this provider state
    pub name: String,
    /// Provider state parameters as key value pairs
    pub params: HashMap<String, Json>
}

impl ProviderState {

    /// Constructs a provider state from the `Json` struct
    pub fn from_json_v3(pact_json: &Json) -> ProviderState {
        let state = match pact_json.find("name") {
            Some(v) => match *v {
                Json::String(ref s) => s.clone(),
                _ => v.to_string()
            },
            None => {
                warn!("Provider state does not have a 'name' field");
                s!("unknown provider states")
            }
        };
        let params = match pact_json.find("params") {
            Some(v) => match *v {
                Json::Object(ref map) => map.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
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
    pub fn from_json(pact_json: &Json) -> Vec<ProviderState> {
        match pact_json.find("providerStates") {
            Some(v) => match *v {
                Json::Array(ref a) => a.iter().map(|i| ProviderState::from_json_v3(i)).collect(),
                _ => vec![]
            },
            None => match pact_json.find("providerState").or(pact_json.find("provider_state")) {
                Some(v) => match *v {
                    Json::String(ref s) => if s.is_empty() {
                        vec![]
                    } else {
                        vec![ProviderState{ name: s.clone(), params: hashmap!{} }]
                    },
                    Json::Null => vec![],
                    _ => vec![ProviderState{ name: v.to_string(), params: hashmap!{} }]
                },
                None => vec![]
            }
        }
    }

    /// Converts this provider state into a JSON structure
    pub fn to_json(&self) -> Json {
        let mut map = btreemap!{
            s!("name") => Json::String(self.name.clone())
        };
        if !self.params.is_empty() {
            map.insert(s!("params"), Json::Object(
                self.params.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
        }
        Json::Object(map)
    }

}

impl Hash for ProviderState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (k, v) in self.params.clone() {
            k.hash(state);
            match v {
                Json::I64(i) => i.hash(state),
                Json::U64(u) => u.hash(state),
                Json::F64(f) => f.to_string().hash(state),
                Json::String(s) => s.hash(state),
                Json::Boolean(b) => b.hash(state),
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
    use rustc_serialize::json::Json;

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
        let provider_states = ProviderState::from_json(&Json::from_str(json).unwrap());
        expect!(provider_states.iter()).to(have_count(2));
        expect!(&provider_states[0]).to(be_equal_to(&ProviderState {
            name: s!("test state"),
            params: hashmap!{ s!("name") => Json::String(s!("Testy")) }
        }));
        expect!(&provider_states[1]).to(be_equal_to(&ProviderState {
            name: s!("test state 2"),
            params: hashmap!{ s!("name") => Json::String(s!("Testy2")) }
        }));
    }

    #[test]
    fn falls_back_to_v2_pact_provider_state() {
        let json = r#"{
            "providerState": "test state",
            "description" : "test interaction"
        }"#;
        let provider_states = ProviderState::from_json(&Json::from_str(json).unwrap());
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
        let provider_states = ProviderState::from_json(&Json::from_str(json).unwrap());
        expect!(provider_states).to(be_empty());
    }

}
