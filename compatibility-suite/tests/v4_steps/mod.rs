use std::collections::HashMap;
use std::panic::RefUnwindSafe;
use std::sync::{Arc, Mutex};

use cucumber::World;
use pact_models::bodies::OptionalBody;
use pact_models::generators::GeneratorTestMode;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::interaction::V4Interaction;
use pact_models::v4::pact::V4Pact;
use serde_json::Value;

use pact_consumer::builders::{InteractionBuilder, MessageInteractionBuilder, PactBuilder};
use pact_matching::{Mismatch, RequestMatchResult};
use pact_mock_server::mock_server::MockServer;
use pact_verifier::{PactSource, ProviderInfo};
use pact_verifier::verification_result::VerificationExecutionResult;

mod http_consumer;
mod http_provider;
mod generators;
mod http_matching;
mod message_consumer;

#[derive(Debug, World)]
pub struct V4World {
  pub request: HttpRequest,
  pub generated_request: HttpRequest,
  pub original_body: OptionalBody,
  pub generated_body: OptionalBody,
  pub generator_test_mode: GeneratorTestMode,
  pub generator_context: HashMap<String, Value>,
  pub builder: PactBuilder,
  pub integration_builder: InteractionBuilder,
  pub message_builder: Option<MessageInteractionBuilder>,
  pub pact: V4Pact,
  pub pact_json: Value,
  pub interactions: Vec<Box<dyn V4Interaction + Send + Sync + RefUnwindSafe>>,
  pub provider_key: String,
  pub provider_server: Arc<Mutex<MockServer>>,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub verification_results: VerificationExecutionResult,
  pub expected_response: HttpResponse,
  pub received_responses: Vec<HttpResponse>,
  pub response_results: Vec<Mismatch>,
  pub expected_request: HttpRequest,
  pub received_requests: Vec<HttpRequest>,
  pub request_results: Vec<RequestMatchResult>
}

impl Default for V4World {
  fn default() -> Self {
    V4World {
      request: Default::default(),
      generated_request: Default::default(),
      original_body: Default::default(),
      generated_body: Default::default(),
      generator_test_mode: GeneratorTestMode::Consumer,
      generator_context: Default::default(),
      builder: PactBuilder::new("C", "P"),
      integration_builder: InteractionBuilder::new("interaction", ""),
      message_builder: None,
      pact: Default::default(),
      pact_json: Default::default(),
      interactions: vec![],
      provider_key: "".to_string(),
      provider_server: Arc::new(Mutex::new(Default::default())),
      provider_info: Default::default(),
      sources: vec![],
      verification_results: VerificationExecutionResult::new(),
      expected_response: Default::default(),
      received_responses: vec![],
      response_results: vec![],
      expected_request: Default::default(),
      received_requests: vec![],
      request_results: vec![]
    }
  }
}
