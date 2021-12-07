//! Executor abstraction for executing callbacks to user code (request filters, provider state change callbacks)

use std::collections::HashMap;
use std::sync::Arc;

use ansi_term::Colour::Yellow;
use async_trait::async_trait;
use maplit::*;
use serde_json::{json, Value};

use pact_models::bodies::OptionalBody;
use pact_models::content_types::JSON;
use pact_models::provider_states::ProviderState;
use pact_models::v4::http_parts::HttpRequest;

use crate::provider_client::make_state_change_request;
use std::fmt::{Display, Formatter};

/// Trait for executors that call request filters
pub trait RequestFilterExecutor {
  /// Mutates requests based on some criteria.
  fn call(self: Arc<Self>, request: &HttpRequest) -> HttpRequest;
}

/// A "null" request filter executor, which does nothing, but permits
/// bypassing of typechecking issues where no filter should be applied.
#[derive(Debug, Clone)]
pub struct NullRequestFilterExecutor {
  // This field is added (and is private) to guarantee that this struct
  // is never instantiated accidentally, and is instead only able to be
  // used for type-level programming.
  _private_field: (),
}

impl RequestFilterExecutor for NullRequestFilterExecutor {
  fn call(self: Arc<Self>, _request: &HttpRequest) -> HttpRequest {
    unimplemented!("NullRequestFilterExecutor should never be called")
  }
}

/// Struct for returning errors from executing a provider state
#[derive(Debug, Clone)]
pub struct ProviderStateError {
  /// Description of the error
  pub description: String,
  /// Interaction ID of the interaction that the error occurred
  pub interaction_id: Option<String>
}

impl Display for ProviderStateError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Provider state failed: {}{}", self.interaction_id.as_ref()
      .map(|id| format!("(interaction_id: {}) ", id)).unwrap_or_default(), self.description)
  }
}

impl std::error::Error for ProviderStateError {}

/// Trait for executors that call provider state callbacks
#[async_trait]
pub trait ProviderStateExecutor {
  /// Invoke the callback for the given provider state, returning an optional Map of values
  async fn call(
    self: Arc<Self>,
    interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    client: Option<&reqwest::Client>
  ) -> anyhow::Result<HashMap<String, Value>>;

  /// If a teardown call for the Executor should be performed
  fn teardown(self: &Self)-> bool;
}

/// Default provider state callback executor, which executes an HTTP request
#[derive(Debug, Clone)]
pub struct HttpRequestProviderStateExecutor {
  /// URL to post state change requests to
  pub state_change_url: Option<String>,
  /// If teardown state change requests should be made (default is false)
  pub state_change_teardown: bool,
  /// If state change request data should be sent in the body (true) or as query parameters (false)
  pub state_change_body: bool
}

impl Default for HttpRequestProviderStateExecutor {
  /// Create a default executor
  fn default() -> HttpRequestProviderStateExecutor {
    HttpRequestProviderStateExecutor {
      state_change_url: None,
      state_change_teardown: false,
      state_change_body: true
    }
  }
}

#[async_trait]
impl ProviderStateExecutor for HttpRequestProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    interaction_id: Option<String>,
    provider_state: &ProviderState,
    setup: bool,
    client: Option<&reqwest::Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    match &self.state_change_url {
      Some(state_change_url) => {
        let mut state_change_request = HttpRequest { method: "POST".to_string(), .. HttpRequest::default() };
        if self.state_change_body {
          let json_body = json!({
                    "state".to_string() : provider_state.name.clone(),
                    "params".to_string() : provider_state.params.clone(),
                    "action".to_string() : if setup {
                        "setup".to_string()
                    } else {
                        "teardown".to_string()
                    }
                });
          state_change_request.body = OptionalBody::Present(json_body.to_string().into(), Some(JSON.clone()), None);
          state_change_request.headers = Some(hashmap!{ "Content-Type".to_string() => vec!["application/json".to_string()] });
        } else {
          let mut query = hashmap!{ "state".to_string() => vec![provider_state.name.clone()] };
          if setup {
            query.insert("action".to_string(), vec!["setup".to_string()]);
          } else {
            query.insert("action".to_string(), vec!["teardown".to_string()]);
          }
          for (k, v) in provider_state.params.clone() {
            query.insert(k, vec![match v {
              Value::String(ref s) => s.clone(),
              _ => v.to_string()
            }]);
          }
          state_change_request.query = Some(query);
        }
        make_state_change_request(client.unwrap_or(&reqwest::Client::default()), &state_change_url, &state_change_request).await
          .map_err(|err| ProviderStateError { description: err.to_string(), interaction_id }.into())
      },
      None => {
        if setup {
          println!("    {}", Yellow.paint("WARNING: State Change ignored as there is no state change URL provided"));
        }
        Ok(hashmap!{})
      }
    }
  }

  fn teardown(
    self: &Self,
  ) -> bool {
    return self.state_change_teardown;
  }
}
