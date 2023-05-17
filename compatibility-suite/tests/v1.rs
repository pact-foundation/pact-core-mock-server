use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use bytes::Bytes;
use cucumber::{given, then, when, World, Parameter};
use cucumber::gherkin::Step;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::http_parts::HttpPart;
use pact_models::pact::{Pact, read_pact};
use pact_models::query_strings::parse_query_string;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;
use pact_models::v4::http_parts::HttpResponse;
use serde_json::Value;
use uuid::Uuid;
use pact_mock_server::matching::MatchResult;

use pact_mock_server::mock_server::{MockServer, MockServerConfig};
use pact_verifier::{NullRequestFilterExecutor, ProviderInfo, ProviderTransport, VerificationOptions};
use pact_verifier::provider_client::make_provider_request;

#[derive(Debug, World)]
pub struct CompatibilitySuiteWorld {
  pub interactions: Vec<RequestResponseInteraction>,
  pub mock_server_key: String,
  pub mock_server: Arc<Mutex<MockServer>>,
  pub response: HttpResponse,
  pub scenario_id: String,
  pub pact: Box<dyn Pact>
}

impl Default for CompatibilitySuiteWorld {
  fn default() -> Self {
    CompatibilitySuiteWorld {
      interactions: vec![],
      mock_server_key: "".to_string(),
      mock_server: Arc::new(Mutex::new(Default::default())),
      response: Default::default(),
      scenario_id: "".to_string(),
      pact: RequestResponsePact::default().boxed()
    }
  }
}

#[derive(Debug, Default, Parameter)]
#[param(name = "numType", regex = "first|second|third")]
struct IndexType(usize);

impl FromStr for IndexType {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "first" => Ok(IndexType(0)),
      "second" => Ok(IndexType(1)),
      "third" => Ok(IndexType(2)),
      _ => Err(anyhow!("{} is not a valid index type", s))
    }
  }
}

#[given("the following HTTP interactions have been defined:")]
fn the_following_http_interactions_have_been_setup(world: &mut CompatibilitySuiteWorld, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (h.clone(), index))
      .collect::<HashMap<String, usize>>();
    for (row, values) in table.rows.iter().skip(1).enumerate() {
      let mut interaction = RequestResponseInteraction {
        description: format!("Interaction {}", row),
        ..RequestResponseInteraction::default()
      };

      if let Some(index) = headers.get("method") {
        if let Some(method) = values.get(*index) {
          interaction.request.method = method.clone();
        }
      }

      if let Some(index) = headers.get("path") {
        if let Some(path) = values.get(*index) {
          interaction.request.path = path.clone();
        }
      }

      if let Some(index) = headers.get("query") {
        if let Some(query) = values.get(*index) {
          interaction.request.query = parse_query_string(query);
        }
      }

      if let Some(index) = headers.get("response") {
        if let Some(response) = values.get(*index) {
          interaction.response.status = response.parse().unwrap();
        }
      }

      if let Some(index) = headers.get("response body") {
        if let Some(response) = values.get(*index) {
          if !response.is_empty() {
            let ct = headers.get("response content")
              .map(|i| values.get(*i))
              .flatten()
              .cloned()
              .unwrap_or("text/plain".to_string());
            interaction.response.headers_mut().insert("content-type".to_string(), vec![ct.clone()]);

            let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", response))
              .expect(format!("could not load fixture '{}'", response).as_str());
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)
              .expect(format!("could not read fixture '{}'", response).as_str());
            interaction.response.body = OptionalBody::Present(Bytes::from(buffer),
              ContentType::parse(ct.as_str()).ok(), None);
          }
        }
      }

      world.interactions.push(interaction);
    }
  }
}

#[when(expr = "the mock server is started with interaction \\{{int}}")]
async fn the_mock_server_is_started_with_interaction(world: &mut CompatibilitySuiteWorld, interaction: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(interaction - 1).unwrap().clone() ],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.mock_server_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V1,
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    world.mock_server_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  world.mock_server = mock_server;
  Ok(())
}

#[when(expr = "the mock server is started with interactions {string}")]
async fn the_mock_server_is_started_with_interactions(world: &mut CompatibilitySuiteWorld, ids: String) -> anyhow::Result<()> {
  let interactions = ids.split(",")
    .map(|id| id.trim().parse::<usize>().unwrap())
    .map(|index| world.interactions.get(index - 1).unwrap().clone())
    .collect();
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions,
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.mock_server_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V1,
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    world.mock_server_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  world.mock_server = mock_server;
  Ok(())
}

#[when(expr = "request \\{{int}} is made to the mock server")]
async fn request_is_made_to_the_mock_server(world: &mut CompatibilitySuiteWorld, num: usize) -> anyhow::Result<()> {
  let request = world.interactions.get(num - 1).unwrap()
    .request.as_v4_request();
  let port = {
    let guard = world.mock_server.lock().unwrap();
    guard.port
  };
  let transport = ProviderTransport {
    port,
    ..ProviderTransport::default()
  };
  let provider_info = ProviderInfo {
    host: "[::1]".to_string(),
    transports: vec![transport.clone()],
    .. ProviderInfo::default()
  };
  let verification_options = VerificationOptions {
    request_filter: None::<Arc<NullRequestFilterExecutor>>,
    .. VerificationOptions::default()
  };
  let client = reqwest::Client::builder().build()?;
  world.response = make_provider_request(
    &provider_info, &request, &verification_options, &client, Some(transport)
  ).await?;
  Ok(())
}

#[when(expr = "request \\{{int}} is made to the mock server with the following changes:")]
async fn request_is_made_to_the_mock_server_with_the_following_changes(
  world: &mut CompatibilitySuiteWorld,
  step: &Step,
  num: usize
) -> anyhow::Result<()> {
  let mut request = world.interactions.get(num - 1).unwrap()
    .request.as_v4_request();

  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "method" => request.method = value.clone(),
          "path" => request.path = value.clone(),
          "query" => request.query = parse_query_string(value),
          _ => {}
        }
      }
    }
  }

  let port = {
    let guard = world.mock_server.lock().unwrap();
    guard.port
  };
  let transport = ProviderTransport {
    port,
    ..ProviderTransport::default()
  };
  let provider_info = ProviderInfo {
    host: "[::1]".to_string(),
    transports: vec![transport.clone()],
    .. ProviderInfo::default()
  };
  let verification_options = VerificationOptions {
    request_filter: None::<Arc<NullRequestFilterExecutor>>,
    .. VerificationOptions::default()
  };
  let client = reqwest::Client::builder().build()?;
  world.response = make_provider_request(
    &provider_info, &request, &verification_options, &client, Some(transport)
  ).await?;

  Ok(())
}

#[then(expr = "a {int} success response is returned")]
fn a_success_response_is_returned(world: &mut CompatibilitySuiteWorld, status: u16) -> anyhow::Result<()> {
  if world.response.status == status {
    Ok(())
  } else {
    Err(anyhow!("Expected a success response of {} but got {}", status, world.response.status))
  }
}

#[then(expr = "a {int} error response is returned")]
fn a_error_response_is_returned(world: &mut CompatibilitySuiteWorld, status: u16) -> anyhow::Result<()> {
  if world.response.status == status {
    Ok(())
  } else {
    Err(anyhow!("Expected an error response of {} but got {}", status, world.response.status))
  }
}

#[then(expr = "the payload will contain the {string} JSON document")]
fn the_payload_will_contain_the_json_document(world: &mut CompatibilitySuiteWorld, name: String) -> anyhow::Result<()> {
  let mut fixture = File::open(format!("pact-compatibility-suite/fixtures/{}.json", name))?;
  let mut buffer = Vec::new();
  fixture.read_to_end(&mut buffer)?;
  let actual_body = world.response.body.value().unwrap_or_default();
  if &actual_body == buffer.as_slice() {
    Ok(())
  } else {
    let body = OptionalBody::Present(Bytes::from(buffer), None, None);
    Err(anyhow!("Expected payload with {} but got {}", world.response.body.display_string(),
      body.display_string()))
  }
}

#[then(expr = "the content type will be set as {string}")]
fn the_content_type_will_be_set_as(world: &mut CompatibilitySuiteWorld, string: String) -> anyhow::Result<()> {
  if let Some(header) = world.response.lookup_header_value("content-type") {
    if header == string {
      Ok(())
    } else {
      Err(anyhow!("Expected response content-type of '{}' but was '{}'", string, header))
    }
  } else {
    Err(anyhow!("Response does not contain a content-type header"))
  }
}

#[when("the pact test is done")]
fn the_pact_test_is_done(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let mut mockserver = world.mock_server.lock().unwrap();
  mockserver.shutdown().map_err(|err| anyhow!(err))?;

  let mismatches = mockserver.mismatches();
  if mismatches.is_empty() {
    let dir = PathBuf::from("target/compatibility-suite/v1").join(&world.scenario_id);
    fs::create_dir_all(&dir)?;
    mockserver.write_pact(&Some(dir.to_string_lossy().to_string()), true)?;
  }

  Ok(())
}

#[then(expr = "the mock server will write out a Pact file for the interaction(s) when done")]
fn the_mock_server_will_write_out_a_pact_file_for_the_interaction_when_done(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let dir = PathBuf::from("target/compatibility-suite/v1").join(&world.scenario_id);
  let pact_file = dir.join("v1-compatibility-suite-c-p.json");
  if pact_file.exists() {
    let pact = read_pact(&pact_file)?;
    if pact.specification_version() == PactSpecification::V1 {
      world.pact = pact;
      Ok(())
    } else {
      Err(anyhow!("Expected Pact file to be V1 Pact, but was {}", pact.specification_version()))
    }
  } else {
    Err(anyhow!("No pact file found: {}", pact_file.to_string_lossy()))
  }
}

#[then(expr = "the mock server will NOT write out a Pact file for the interaction(s) when done")]
fn the_mock_server_will_not_write_out_a_pact_file_for_the_interaction_when_done(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let dir = PathBuf::from("target/compatibility-suite/v1").join(&world.scenario_id);
  let pact_file = dir.join("v1-compatibility-suite-c-p.json");
  if pact_file.exists() {
    Err(anyhow!("Expected no pact file, but found: {}", pact_file.to_string_lossy()))
  } else {
    Ok(())
  }
}

#[then("the mock server status will be OK")]
fn the_mock_server_status_will_be_ok(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let mock_server = world.mock_server.lock().unwrap();
  if mock_server.mismatches().is_empty() {
    Ok(())
  } else {
    Err(anyhow!("Mock server has {} mismatches", mock_server.mismatches().len()))
  }
}

#[then("the mock server status will NOT be OK")]
fn the_mock_server_status_will_be_error(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let mock_server = world.mock_server.lock().unwrap();
  if mock_server.mismatches().is_empty() {
    Err(anyhow!("Mock server has no mismatches"))
  } else {
    Ok(())
  }
}

#[then(expr = "the pact file will contain \\{{int}} interaction(s)")]
fn the_pact_file_will_contain_interaction(world: &mut CompatibilitySuiteWorld, num: usize) -> anyhow::Result<()> {
  let i = world.pact.interactions().len();
  if i == num {
    Ok(())
  } else {
    Err(anyhow!("Expected the pact file to contain {} interaction(s), but had {}", num, i))
  }
}

#[then(expr = "the \\{{numType}} interaction request will be for a {string}")]
fn the_interaction_request_will_be_for_a(world: &mut CompatibilitySuiteWorld, num: IndexType, method: String) -> anyhow::Result<()> {
  if let Some(interaction) = world.pact.interactions().get(num.0) {
    if let Some(reqres) = interaction.as_request_response() {
      if reqres.request.method == method {
        Ok(())
      } else {
        Err(anyhow!("Expected interaction {} request to be for a {} but was a {}", num.0 + 1, method, reqres.request.method))
      }
    } else {
      Err(anyhow!("Interaction {} is not a RequestResponseInteraction", num.0 + 1))
    }
  } else {
    Err(anyhow!("Did not find interaction {} in the Pact", num.0 + 1))
  }
}

#[then(expr = "the \\{{numType}} interaction response will contain the {string} document")]
fn the_interaction_response_will_contain_the_document(world: &mut CompatibilitySuiteWorld, num: IndexType, fixture: String) -> anyhow::Result<()> {
  if let Some(interaction) = world.pact.interactions().get(num.0) {
    if let Some(reqres) = interaction.as_request_response() {
      let mut fixture = File::open(format!("pact-compatibility-suite/fixtures/{}", fixture))?;
      let mut buffer = Vec::new();
      fixture.read_to_end(&mut buffer)?;
      let json: Value = serde_json::from_slice(&buffer)?;
      let json_str = json.to_string();
      let actual_body = reqres.response.body.value().unwrap_or_default();
      if &actual_body == json_str.as_bytes() {
        Ok(())
      } else {
        let body = OptionalBody::Present(Bytes::from(buffer), None, None);
        Err(anyhow!("Expected Interaction {} response payload with {} but got {}", num.0 + 1,
          reqres.response.body.display_string(), body.display_string()))
      }
    } else {
      Err(anyhow!("Interaction {} is not a RequestResponseInteraction", num.0 + 1))
    }
  } else {
    Err(anyhow!("Did not find interaction {} in the Pact", num.0 + 1))
  }
}

#[then(expr = "the mock server status will be an expected but not received error for interaction \\{{int}}")]
fn the_mock_server_status_will_be_an_expected_but_not_received_error_for_interaction(
  world: &mut CompatibilitySuiteWorld,
  num: usize
) -> anyhow::Result<()> {
  let mock_server = { world.mock_server.lock().unwrap().clone() };
  if let Some(interaction) = world.interactions.get(num - 1) {
    if let Some(_) = mock_server.mismatches().iter().find(|mismatch| {
      match mismatch {
        MatchResult::MissingRequest(request) => request == &interaction.request.as_v4_request(),
        _ => false
      }
    }) {
      Ok(())
    } else {
      Err(anyhow!("Did not find a MissingRequest mismatch for Interaction {}", num))
    }
  } else {
    Err(anyhow!("Did not find interaction {} in the Pact", num))
  }
}

#[then(expr = "the \\{{numType}} interaction request query parameters will be {string}")]
fn the_interaction_request_query_parameters_will_be(
  world: &mut CompatibilitySuiteWorld,
  num: IndexType,
  query_str: String
) -> anyhow::Result<()> {
  if let Some(interaction) = world.pact.interactions().get(num.0) {
    if let Some(reqres) = interaction.as_request_response() {
      if reqres.request.query == parse_query_string(query_str.as_str()) {
        Ok(())
      } else {
        Err(anyhow!("Expected interaction {} request to have query {} but was {:?}", num.0 + 1, query_str, reqres.request.query))
      }
    } else {
      Err(anyhow!("Interaction {} is not a RequestResponseInteraction", num.0 + 1))
    }
  } else {
    Err(anyhow!("Did not find interaction {} in the Pact", num.0 + 1))
  }
}

#[then("the mock server status will be mismatches")]
fn the_mock_server_status_will_be_mismatches(world: &mut CompatibilitySuiteWorld) -> anyhow::Result<()> {
  let mock_server = world.mock_server.lock().unwrap();
  if mock_server.mismatches().is_empty() {
    Err(anyhow!("Mock server has no mismatches"))
  } else {
    Ok(())
  }
}

#[then(expr = "the mismatches will contain a {string} mismatch with error {string}")]
fn the_mismatches_will_contain_a_mismatch_with_error(
  world: &mut CompatibilitySuiteWorld,
  mismatch_type: String,
  error: String
) -> anyhow::Result<()> {
  let mock_server = world.mock_server.lock().unwrap();
  let mismatches: Vec<_> = mock_server.mismatches().iter()
    .flat_map(|m| match m {
      MatchResult::RequestMismatch(_, mismatches) => mismatches.clone(),
      _ => vec![]
    })
    .collect();
  if mismatches.iter().find(|ms| ms.mismatch_type().to_lowercase().starts_with(mismatch_type.as_str()) && ms.description() == error).is_some() {
    Ok(())
  } else {
    Err(anyhow!("Did not find a {} mismatch with error {}", mismatch_type, error))
  }
}

#[then(expr = "the mock server status will be an unexpected request received error for interaction \\{{int}}")]
fn the_mock_server_status_will_be_an_unexpected_request_received_error_for_interaction(
  world: &mut CompatibilitySuiteWorld,
  num: usize
) -> anyhow::Result<()> {
  let mock_server = { world.mock_server.lock().unwrap().clone() };
  if let Some(interaction) = world.interactions.get(num - 1) {
    if let Some(_) = mock_server.mismatches().iter().find(|mismatch| {
      match mismatch {
        MatchResult::RequestNotFound(request) => request.method == interaction.request.method &&
          request.path == interaction.request.path && request.query == interaction.request.query,
        _ => false
      }
    }) {
      Ok(())
    } else {
      Err(anyhow!("Did not find a RequestNotFound mismatch for Interaction {}", num))
    }
  } else {
    Err(anyhow!("Did not find interaction {} in the Pact", num))
  }
}

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  CompatibilitySuiteWorld::cucumber()
    .fail_on_skipped()
    .before(|_feature, _, scenario, world| Box::pin(async move {
      world.scenario_id = scenario.name.clone();
    }))
    .run_and_exit("pact-compatibility-suite/features/V1")
    .await;
}
