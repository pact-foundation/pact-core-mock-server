use std::collections::HashMap;

use anyhow::anyhow;
use cucumber::{given, then, when};
use cucumber::gherkin::Step;
use pact_models::PactSpecification;
use serde_json::{json, Map, Value};

use pact_consumer::builders::{InteractionBuilder, PactBuilder};

use crate::v3_steps::V3World;

#[given("an integration is being defined for a consumer test")]
fn an_integration_is_being_defined_for_a_consumer_test(world: &mut V3World) {
  world.builder = PactBuilder::new("V3 consumer", "V3 provider");
  world.integration_builder = InteractionBuilder::new("interaction for a consumer test", "");
}

#[given(expr = "a provider state {string} is specified")]
fn a_provider_state_is_specified(world: &mut V3World, state: String) {
  world.integration_builder.given(state);
}

#[given(expr = "a provider state {string} is specified with the following data:")]
fn a_provider_state_is_specified_with_the_following_data(
  world: &mut V3World,
  step: &Step,
  state: String
) -> anyhow::Result<()> {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap().iter()
      .enumerate()
      .map(|(index, h)| (index, h.clone()))
      .collect::<HashMap<usize, String>>();
    if let Some(value) = table.rows.get(1) {
      let data = value.iter().enumerate()
        .map(|(index, v)| {
          (headers.get(&index).unwrap().clone(), serde_json::from_str(v).unwrap())
        })
        .collect::<Map<_, _>>();
      world.integration_builder.given_with_params(state, &Value::Object(data));
    }
    Ok(())
  } else {
    Err(anyhow!("No data table defined"))
  }
}

#[when("the Pact file for the test is generated")]
fn the_pact_file_for_the_test_is_generated(world: &mut V3World) {
  world.builder.push_interaction(&world.integration_builder.build());
  world.pact = world.builder.build();
  world.pact_json = world.pact.to_json(PactSpecification::V3).unwrap();
}

#[then(expr = "the interaction in the Pact file will contain {int} provider state(s)")]
fn the_interaction_in_the_pact_file_will_contain_provider_states(
  world: &mut V3World,
  states: usize
) -> anyhow::Result<()> {
  let interaction = get_interaction(&world.pact_json, 0)?;
  if let Some(provider_states) = interaction.get("providerStates") {
    if let Some(provider_states_array) = provider_states.as_array() {
      if provider_states_array.len() == states {
        Ok(())
      } else {
        Err(anyhow!("Expected {} provider states, but Pact had {}", states, provider_states_array.len()))
      }
    } else {
      Err(anyhow!("providerStates not valid JSON"))
    }
  } else {
    Err(anyhow!("No providerStates in Interaction JSON"))
  }
}

fn get_interaction(pact_json: &Value, num: usize) -> anyhow::Result<Value> {
  if let Some(interactions) = pact_json.get("interactions") {
    if let Some(interaction) = interactions.get(num) {
      Ok(interaction.clone())
    } else {
      Err(anyhow!("No interactions in Pact JSON"))
    }
  } else {
    Err(anyhow!("Generated Pact JSON is invalid"))
  }
}

#[then(expr = "the interaction in the Pact file will contain provider state {string}")]
fn the_interaction_in_the_pact_file_will_contain_provider_state(
  world: &mut V3World,
  state_name: String
) -> anyhow::Result<()> {
  let interaction = get_interaction(&world.pact_json, 0)?;
  if let Some(provider_states) = interaction.get("providerStates") {
    if let Some(provider_states_array) = provider_states.as_array() {
      if provider_states_array.iter()
        .find(|state| state.get("name").cloned().unwrap_or(Value::Null) == json!(state_name))
        .is_some() {
        Ok(())
      } else {
        Err(anyhow!("Did not find a provider state with name {}", state_name))
      }
    } else {
      Err(anyhow!("providerStates not valid JSON"))
    }
  } else {
    Err(anyhow!("No providerStates in Interaction JSON"))
  }
}

#[then(expr = "the provider state {string} in the Pact file will contain the following parameters:")]
fn the_provider_state_in_the_pact_file_will_contain_the_following_parameters(
  world: &mut V3World,
  step: &Step,
  state_name: String
) -> anyhow::Result<()> {
  if let Some(table) = step.table.as_ref() {
    if let Some(value) = table.rows.get(1) {
      let data: Value = serde_json::from_str(value.get(0).unwrap())?;
      let interaction = get_interaction(&world.pact_json, 0)?;
      if let Some(provider_states) = interaction.get("providerStates") {
        if let Some(provider_states_array) = provider_states.as_array() {
          if let Some(state) = provider_states_array.iter()
            .find(|state| state.get("name").cloned().unwrap_or(Value::Null) == json!(state_name)) {
            if let Some(params) = state.get("params") {
              if params == &data {
                Ok(())
              } else {
                Err(anyhow!("Provider state with name {} parameters {} does not equal {}", state_name,
                  params, data))
              }
            } else {
              Err(anyhow!("Provider state with name {} has no parameters", state_name))
            }
          } else {
            Err(anyhow!("Did not find a provider state with name {}", state_name))
          }
        } else {
          Err(anyhow!("providerStates not valid JSON"))
        }
      } else {
        Err(anyhow!("No providerStates in Interaction JSON"))
      }
    } else {
      Err(anyhow!("No data table defined"))
    }
  } else {
    Err(anyhow!("No data table defined"))
  }
}
