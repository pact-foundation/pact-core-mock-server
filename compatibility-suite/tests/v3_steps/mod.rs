use std::collections::HashMap;
use std::panic::RefUnwindSafe;

use cucumber::World;
use pact_models::bodies::OptionalBody;
use pact_models::generators::GeneratorTestMode;
use pact_models::pact::Pact;
use pact_models::prelude::RequestResponsePact;
use pact_models::request::Request;
use pact_models::response::Response;
use serde_json::Value;

use pact_consumer::builders::{InteractionBuilder, PactBuilder};
use pact_matching::RequestMatchResult;

mod http_consumer;
mod http_matching;
mod generators;

#[derive(Debug, World)]
pub struct V3World {
  pub builder: PactBuilder,
  pub integration_builder: InteractionBuilder,
  pub pact: Box<dyn Pact + Send + Sync + RefUnwindSafe>,
  pub pact_json: Value,
  pub expected_request: Request,
  pub received_requests: Vec<Request>,
  pub match_result: Vec<RequestMatchResult>,
  pub request: Request,
  pub response: Response,
  pub generated_request: Request,
  pub generated_response: Response,
  pub generator_test_mode: GeneratorTestMode,
  pub generator_context: HashMap<String, Value>,
  pub original_body: OptionalBody,
  pub generated_body: OptionalBody
}

impl Default for V3World {
  fn default() -> Self {
    V3World {
      builder: PactBuilder::new("C", "P"),
      integration_builder: InteractionBuilder::new("I", ""),
      pact: Box::new(RequestResponsePact::default()),
      pact_json: Default::default(),
      expected_request: Default::default(),
      received_requests: vec![],
      match_result: vec![],
      request: Default::default(),
      response: Default::default(),
      generated_request: Default::default(),
      generated_response: Default::default(),
      generator_test_mode: GeneratorTestMode::Consumer,
      generator_context: Default::default(),
      original_body: Default::default(),
      generated_body: Default::default()
    }
  }
}
