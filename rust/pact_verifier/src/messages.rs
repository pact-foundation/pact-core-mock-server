use crate::callback_executors::RequestFilterExecutor;
use crate::{ProviderInfo, VerificationOptions, MismatchResult};
use pact_matching::models::message::Message;
use std::collections::HashMap;
use serde_json::{json, Value};
use pact_matching::models::{Request, OptionalBody};
use crate::provider_client::{make_provider_request, provider_client_error_to_string};

pub async fn verify_message_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &Message,
  options: &VerificationOptions<F>,
  client: &reqwest::Client,
  verification_context: HashMap<String, Value>
) -> Result<(), MismatchResult> {
  let expected_response = &interaction.contents;
  let mut request_body = json!({
    "description": interaction.description
  });
  if !interaction.provider_states.is_empty() {
    if let Some(map) = request_body.as_object_mut() {
      map.insert("providerStates".into(), Value::Array(interaction.provider_states.iter()
        .map(|ps| ps.to_json()).collect()));
    }
  }
  let message_request = Request {
    method: "POST".into(),
    body: OptionalBody::Present(request_body.to_string().as_bytes().to_vec(), Some("application/json".into())),
    .. Request::default()
  };
  match make_provider_request(provider, &message_request, options, client).await {
    Ok(ref actual_response) => {
      // let mismatches = match_response(expected_response.clone(), actual_response.clone());
      // if mismatches.is_empty() {
      //   Ok(())
      // } else {
      //   Err(MismatchResult::Mismatches {
      //     mismatches,
      //     expected: expected_response.clone(),
      //     actual: actual_response.clone(),
      //     interaction_id: interaction.id.clone()
      //   })
      // }
      Ok(())
    },
    Err(err) => {
      Err(MismatchResult::Error(provider_client_error_to_string(err), interaction.id.clone()))
    }
  }
}
