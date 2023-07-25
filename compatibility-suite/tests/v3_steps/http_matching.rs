use std::collections::hash_map::Entry;
use std::fs::File;
use std::io::BufReader;

use anyhow::anyhow;
use cucumber::{given, then, when};
use cucumber::gherkin::Step;
use maplit::hashmap;
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::interaction::Interaction;
use pact_models::matchingrules::matchers_from_json;
use pact_models::pact::Pact;
use pact_models::prelude::RequestResponseInteraction;
use pact_models::request::Request;
use pact_models::sync_pact::RequestResponsePact;
use serde_json::Value;

use pact_matching::{match_request, Mismatch};

use crate::shared_steps::setup_body;
use crate::v3_steps::V3World;

#[given(expr = "an expected request with a(n) {string} header of {string}")]
fn an_expected_request_with_a_header_of(world: &mut V3World, header: String, value: String) {
  let headers = world.expected_request.headers_mut();
  match headers.entry(header.clone()) {
    Entry::Occupied(mut entry) => {
      entry.insert(parse_header(header.as_str(), value.as_str()));
    }
    Entry::Vacant(entry) => {
      entry.insert(parse_header(header.as_str(), value.as_str()));
    }
  }
}

#[given(expr = "a request is received with a(n) {string} header of {string}")]
fn a_request_is_received_with_a_header_of(world: &mut V3World, header: String, value: String) {
  world.received_requests.push(Request {
    headers: Some(hashmap!{ header.clone() => parse_header(header.as_str(), value.as_str()) }),
    .. Request::default()
  })
}

#[given(expr = "an expected request configured with the following:")]
fn an_expected_request_configured_with_the_following(world: &mut V3World, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => setup_body(value, &mut world.expected_request),
          "matching rules" => {
            let json: Value = if value.starts_with("JSON:") {
              serde_json::from_str(value.strip_prefix("JSON:").unwrap_or(value).trim()).unwrap()
            } else {
              let f = File::open(format!("pact-compatibility-suite/fixtures/{}", value))
                .expect(format!("could not load fixture '{}'", value).as_str());
              let reader = BufReader::new(f);
              serde_json::from_reader(reader).unwrap()
            };
            world.expected_request.matching_rules = matchers_from_json(&json, &None)
              .expect("Matching rules fixture is not valid JSON");
          }
          _ => {}
        }
      }
    }
  }
}

#[given(expr = "a request is received with the following:")]
fn a_request_is_received_with_the_following(world: &mut V3World, step: &Step) {
  let mut request = Request::default();
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for (index, value) in table.rows.get(1).unwrap().iter().enumerate() {
      if let Some(field) = headers.get(index) {
        match field.as_str() {
          "body" => setup_body(value, &mut request),
          _ => {}
        }
      }
    }
  }
  world.received_requests.push(request);
}

#[given(expr = "the following requests are received:")]
fn the_following_requests_are_received(world: &mut V3World, step: &Step) {
  if let Some(table) = step.table.as_ref() {
    let headers = table.rows.first().unwrap();
    for row in table.rows.iter().skip(1) {
      let mut request = Request::default();
      for (index, value) in row.iter().enumerate() {
        if let Some(field) = headers.get(index) {
          match field.as_str() {
            "body" => setup_body(value, &mut request),
            _ => {}
          }
        }
      }
      world.received_requests.push(request);
    }
  }
}

#[when("the request is compared to the expected one")]
async fn the_request_is_compared_to_the_expected_one(world: &mut V3World) {
  world.match_result.push(
    match_request(
      world.expected_request.as_v4_request(),
    world.received_requests.first().unwrap().as_v4_request(),
      &RequestResponsePact::default().boxed(),
      &RequestResponseInteraction::default().boxed()
    ).await
  );
}

#[when("the requests are compared to the expected one")]
async fn the_requests_are_compared_to_the_expected_one(world: &mut V3World) {
  for request in &world.received_requests {
    world.match_result.push(
      match_request(
        world.expected_request.as_v4_request(),
        request.as_v4_request(),
        &RequestResponsePact::default().boxed(),
        &RequestResponseInteraction::default().boxed()
      ).await
    );
  }
}

#[then("the comparison should be OK")]
fn the_comparison_should_be_ok(world: &mut V3World) -> anyhow::Result<()> {
  if world.match_result.iter().all(|result| result.all_matched()) {
    Ok(())
  } else {
    let count = world.match_result.iter()
      .filter_map(|res| {
        let mismatches = res.mismatches();
        if mismatches.is_empty() {
          None
        } else {
          Some(mismatches)
        }
      })
      .flatten()
      .collect::<Vec<_>>();
    Err(anyhow!("There were match results with mismatches ({:?})", count))
  }
}

#[then("the comparison should NOT be OK")]
fn the_comparison_should_not_be_ok(world: &mut V3World) -> anyhow::Result<()> {
  if world.match_result.iter().all(|result| result.all_matched()) {
    Err(anyhow!("All requests matched"))
  } else {
    Ok(())
  }
}

#[then(expr = "the mismatches will contain a mismatch with error {string} -> {string}")]
fn the_mismatches_will_contain_a_mismatch_with_error(
  world: &mut V3World,
  error_path: String,
  error: String
) -> anyhow::Result<()> {
  if world.match_result.iter().flat_map(|result| result.mismatches())
    .any(|mismatch| {
      let path_matches = match &mismatch {
        Mismatch::QueryMismatch { parameter, .. } => parameter.as_str() == error_path,
        Mismatch::HeaderMismatch { key, .. } => key.as_str() == error_path,
        Mismatch::BodyMismatch { path, .. } => path.as_str() == error_path,
        Mismatch::MetadataMismatch { key, .. } => key.as_str() == error_path,
        _ => false
      };
      path_matches && mismatch.description().contains(error.as_str())
    }) {
    Ok(())
  } else {
    Err(anyhow!("Did not find a mismatch with the required error message"))
  }
}
