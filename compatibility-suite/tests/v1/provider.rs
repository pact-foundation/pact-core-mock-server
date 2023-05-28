use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::anyhow;
use async_trait::async_trait;
use cucumber::{given, then, when, World};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_models::{Consumer, PactSpecification, Provider};
use pact_models::pact::Pact;
use pact_models::provider_states::ProviderState;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::sync_pact::RequestResponsePact;
use reqwest::Client;
use serde_json::Value;
use uuid::Uuid;
use pact_matching::Mismatch;

use pact_mock_server::mock_server::{MockServer, MockServerConfig};
use pact_verifier::{FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, ProviderTransport, PublishOptions, VerificationOptions, verify_provider_async};
use pact_verifier::callback_executors::ProviderStateExecutor;
use pact_verifier::verification_result::{VerificationExecutionResult, VerificationMismatchResult};

use crate::v1::common::setup_common_interactions;

#[derive(Debug, World)]
pub struct ProviderWorld {
  pub interactions: Vec<RequestResponseInteraction>,
  pub provider_key: String,
  pub provider_server: Arc<Mutex<MockServer>>,
  pub provider_info: ProviderInfo,
  pub sources: Vec<PactSource>,
  pub publish_options: Option<PublishOptions>,
  pub verification_results: VerificationExecutionResult
}

impl Default for ProviderWorld {
  fn default() -> Self {
    ProviderWorld {
      interactions: vec![],
      provider_key: "".to_string(),
      provider_server: Arc::new(Mutex::new(Default::default())),
      provider_info: ProviderInfo::default(),
      sources: vec![],
      publish_options: None,
      verification_results: VerificationExecutionResult::new(),
    }
  }
}

#[derive(Debug)]
struct DummyProviderStateExecutor;

#[async_trait]
impl ProviderStateExecutor for DummyProviderStateExecutor {
  async fn call(
    self: Arc<Self>,
    _interaction_id: Option<String>,
    _provider_state: &ProviderState,
    _setup: bool,
    _client: Option<&Client>
  ) -> anyhow::Result<HashMap<String, Value>> {
    Ok(hashmap!{})
  }

  fn teardown(self: &Self) -> bool {
    return false
  }
}

#[given("the following HTTP interactions have been defined:")]
fn the_following_http_interactions_have_been_setup(world: &mut ProviderWorld, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let interactions = setup_common_interactions(table);
    world.interactions.extend(interactions);
  }
}

#[given(expr = "a provider is started that returns the response from interaction \\{{int}}")]
#[allow(deprecated)]
async fn a_provider_is_started_that_returns_the_response_from_interaction(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "v1-compatibility-suite-c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.provider_key = Uuid::new_v4().to_string();
  let config = MockServerConfig {
    pact_specification: PactSpecification::V1,
    .. MockServerConfig::default()
  };
  let (mock_server, future) = MockServer::new(
    world.provider_key.clone(), pact.boxed(), "[::1]:0".parse()?, config
  ).await.map_err(|err| anyhow!(err))?;
  tokio::spawn(future);
  world.provider_server = mock_server;

  let ms = world.provider_server.lock().unwrap();
  world.provider_info = ProviderInfo {
    name: "p".to_string(),
    host: "[::1]".to_string(),
    port: ms.port,
    transports: vec![ProviderTransport {
      port: ms.port,
      .. ProviderTransport::default()
    }],
    .. ProviderInfo::default()
  };

  Ok(())
}

#[given(expr = "a Pact file for interaction \\{{int}} is to be verified")]
fn a_pact_file_for_interaction_is_to_be_verified(world: &mut ProviderWorld, num: usize) -> anyhow::Result<()> {
  let pact = RequestResponsePact {
    consumer: Consumer { name: "c".to_string() },
    provider: Provider { name: "p".to_string() },
    interactions: vec![ world.interactions.get(num - 1).unwrap().clone() ],
    specification_version: PactSpecification::V1,
    .. RequestResponsePact::default()
  };
  world.sources.push(PactSource::String(pact.to_json(PactSpecification::V1)?.to_string()));
  Ok(())
}

#[when("the verification is run")]
async fn the_verification_is_run(world: &mut ProviderWorld) -> anyhow::Result<()> {
  let options: VerificationOptions<NullRequestFilterExecutor> = VerificationOptions::default();
  world.verification_results = verify_provider_async(
    world.provider_info.clone(),
    world.sources.clone(),
    FilterInfo::None,
    vec![],
    &options,
    world.publish_options.as_ref(),
    &Arc::new(DummyProviderStateExecutor),
    None
  ).await?;
  Ok(())
}

#[then("the verification will be successful")]
fn the_verification_will_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Ok(())
  } else {
    Err(anyhow!("Verification failed"))
  }
}

//   @Given('a provider is started that returns the responses from interactions {string}')
//   void a_provider_is_started_that_returns_the_responses_from_interactions(String ids) {
//     def interactions = ids.split(',\\s*').collect {
//       def index = it.toInteger()
//       world.interactions[index - 1]
//     }
//     Pact pact = new RequestResponsePact(new Provider('p'), new Consumer('v1-compatibility-suite-c'),
//       interactions)
//     mockProvider = new KTorMockServer(pact, new MockProviderConfig())
//     mockProvider.start()
//     providerInfo = new ProviderInfo('p')
//     providerInfo.port = mockProvider.port
//   }

#[then("the verification will NOT be successful")]
fn the_verification_will_not_be_successful(world: &mut ProviderWorld) -> anyhow::Result<()> {
  if world.verification_results.result {
    Err(anyhow!("Was expecting the verification to fail"))
  } else {
    Ok(())
  }
}

#[then(expr = "the verification results will contain a {string} error")]
fn the_verification_results_will_contain_a_error(world: &mut ProviderWorld, err: String) -> anyhow::Result<()> {
  if world.verification_results.errors.iter().any(|(_, r)| {
    match r {
      VerificationMismatchResult::Mismatches { mismatches, .. } => {
        mismatches.iter().any(|mismatch| {
          match mismatch {
            Mismatch::MethodMismatch { .. } => false,
            Mismatch::PathMismatch { .. } => false,
            Mismatch::StatusMismatch { .. } => err == "Response status did not match",
            Mismatch::QueryMismatch { .. } => false,
            Mismatch::HeaderMismatch { .. } => false,
            Mismatch::BodyTypeMismatch { .. } => false,
            Mismatch::BodyMismatch { .. } => false,
            Mismatch::MetadataMismatch { .. } => false
          }
        })
      }
      VerificationMismatchResult::Error { error, .. } => error.as_str() == err
    }
  }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find error message in verification results"))
  }
}

//   @Given('a Pact file for interaction \\{{int}} is to be verified from a Pact broker')
//   void a_pact_file_for_interaction_is_to_be_verified_from_a_pact_broker(Integer num) {
//     Pact pact = new RequestResponsePact(new Provider('p'),
//       new Consumer("c_$num"), [ world.interactions[num - 1] ])
//     def pactJson = pact.toMap(PactSpecVersion.V1)
//     pactJson['_links'] = [
//       "pb:publish-verification-results": [
//         "title": "Publish verification results",
//         "href": "http://localhost:1234/pacts/provider/p/consumer/c_$num/verification-results"
//       ]
//     ]
//     pactJson['interactions'][0]['_id'] = world.interactions[num - 1].interactionId
//
//     File contents = new File("pact-compatibility-suite/fixtures/pact-broker_c${num}.json")
//     Pact brokerPact = DefaultPactReader.INSTANCE.loadPact(contents) as BasePact
//     /// AAARGH! My head. Adding a Pact Interaction to a Pact file for fetching a Pact file for verification
//     def matchingRules = new MatchingRulesImpl()
//     matchingRules
//       .addCategory('body')
//       .addRule('$._links.pb:publish-verification-results.href',
//         new RegexMatcher(".*\\/(pacts\\/provider\\/p\\/consumer\\/c_$num\\/verification-results)"))
//     Generators generators = new Generators([
//       (Category.BODY): [
//         '$._links.pb:publish-verification-results.href': new MockServerURLGenerator(
//           "http://localhost:1234/pacts/provider/p/consumer/c_$num/verification-results",
//           ".*\\/(pacts\\/provider\\/p\\/consumer\\/c_$num\\/verification-results)"
//         )
//       ]
//     ])
//     Interaction interaction = new RequestResponseInteraction("Interaction $num", [],
//       new Request('GET', "/pacts/provider/p/consumer/c_$num"),
//       new Response(200,
//         ['content-type': ['application/json']],
//         OptionalBody.body(Json.INSTANCE.prettyPrint(pactJson).bytes, ContentType.JSON),
//         matchingRules, generators
//       )
//     )
//     brokerPact.interactions << interaction
//
//     def mockBroker = new KTorMockServer(brokerPact, new MockProviderConfig())
//     mockBroker.start()
//     mockBrokers << mockBroker
//
//     providerInfo.hasPactsFromPactBrokerWithSelectorsV2("http://127.0.0.1:${mockBroker.port}", [])
//   }
//
//   @Then('a verification result will NOT be published back')
//   void a_verification_result_will_not_be_published_back() {
//     assert mockBrokers.every { mock ->
//       mock.matchedRequests.find { it.path.endsWith('/verification-results') } == null
//     }
//   }
//
//   @Given('publishing of verification results is enabled')
//   void publishing_of_verification_results_is_enabled() {
//     verificationProperties['pact.verifier.publishResults'] = 'true'
//   }
//
//   @Then('a successful verification result will be published back for interaction \\{{int}}')
//   void a_successful_verification_result_will_be_published_back_for_interaction(Integer num) {
//     def request = mockBrokers.collect {
//       it.matchedRequests.find { it.path == "/pacts/provider/p/consumer/c_$num/verification-results".toString() }
//     }.find()
//     assert request != null
//     def json = new JsonSlurper().parseText( request.body.valueAsString())
//     assert json.success == true
//   }
//
//   @Then("a failed verification result will be published back for the interaction \\{{int}}")
//   void a_failed_verification_result_will_be_published_back_for_the_interaction(Integer num) {
//     def request = mockBrokers.collect {
//       it.matchedRequests.find { it.path == "/pacts/provider/p/consumer/c_$num/verification-results".toString() }
//     }.find()
//     assert request != null
//     def json = new JsonSlurper().parseText( request.body.valueAsString())
//     assert json.success == false
//   }
