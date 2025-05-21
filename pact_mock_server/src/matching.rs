//!
//! The matching module defines how a request is matched
//! against a list of potential interactions.
//!

use std::fmt::{Debug, Display, Formatter};

use futures::prelude::*;
use itertools::Itertools;
use serde_json::json;

use pact_matching::{Mismatch, RequestMatchResult};
use pact_models::interaction::Interaction;
use pact_models::PactSpecification;
use pact_models::prelude::Pact;
use pact_models::prelude::v4::SynchronousHttp;
use pact_models::v4::http_parts::{HttpRequest, HttpResponse};
use pact_models::v4::V4InteractionType;
use pact_models::v4::pact::V4Pact;
use tracing::error;

/// Enum to define a match result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
  /// Match result where the request was successfully matched. Stores the expected request,
  /// response returned and the actual request that was received.
  RequestMatch(HttpRequest, HttpResponse, HttpRequest),
  /// Match result where there were a number of mismatches. Stores the expected and actual requests,
  /// and all the mismatches.
  RequestMismatch(HttpRequest, HttpRequest, Vec<Mismatch>),
  /// Match result where the request was not expected
  RequestNotFound(HttpRequest),
  /// Match result where an expected request was not received
  MissingRequest(HttpRequest)
}

impl MatchResult {
    /// Returns the match key for this mismatch
    pub fn match_key(&self) -> String {
        match self {
            &MatchResult::RequestMatch(_, _, _) => "Request-Matched",
            &MatchResult::RequestMismatch(_, _, _) => "Request-Mismatch",
            &MatchResult::RequestNotFound(_) => "Unexpected-Request",
            &MatchResult::MissingRequest(_) => "Missing-Request"
        }.to_string()
    }

    /// Returns true if this match result is a `RequestMatch`
    pub fn matched(&self) -> bool {
        match self {
            &MatchResult::RequestMatch(_, _, _) => true,
            _ => false
        }
    }

    /// Returns true if this is an unexpected OPTIONS request
    pub fn cors_preflight(&self) -> bool {
      match self {
        MatchResult::RequestNotFound(req) => req.method == "OPTIONS",
        _ => false
      }
    }

    /// Converts this match result to a `Value` struct
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            MatchResult::RequestMatch(_, _, _) => json!({ "type" : "request-match"}),
            MatchResult::RequestMismatch(request, _, mismatches) => mismatches_to_json(request, mismatches),
            MatchResult::RequestNotFound(req) => json!({
                "type": "request-not-found",
                "method": req.method,
                "path": req.path,
                "request": req.as_v3_request().to_json(&PactSpecification::V3)
            }),
            MatchResult::MissingRequest(request) => json!({
                "type": "missing-request",
                "method": request.method,
                "path": request.path,
                "request": request.as_v3_request().to_json(&PactSpecification::V3)
            })
        }
    }
}

impl Display for MatchResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MatchResult::RequestMatch(request, _, _) => {
        write!(f, "Request matched OK - {}", request)
      },
      MatchResult::RequestMismatch(request, _, mismatches) => {
        write!(f, "Request did not match - {}", request)?;
        for (i, mismatch) in mismatches.iter().enumerate() {
          write!(f, "    {}) {}", i, mismatch)?;
        }
        Ok(())
      },
      MatchResult::RequestNotFound(request) => {
        write!(f, "Request was not expected - {}", request)
      },
      MatchResult::MissingRequest(request) => {
        write!(f, "Request was not received - {}", request)
      }
    }
  }
}

fn mismatches_to_json(request: &HttpRequest, mismatches: &Vec<Mismatch>) -> serde_json::Value {
    json!({
        "type" : "request-mismatch",
        "method" : request.method,
        "path" : request.path,
        "mismatches" : mismatches.iter().map(|m| m.to_json()).collect::<serde_json::Value>()
    })
}

///
/// Matches a request against a list of interactions
///
pub async fn match_request(
  req: &HttpRequest,
  pact: &V4Pact,
) -> MatchResult {
  let interactions = pact.filter_interactions(V4InteractionType::Synchronous_HTTP);
  let match_results = futures::stream::iter(interactions)
    .filter(|i| future::ready(i.is_request_response()))
    .filter_map(|i| async move {
      let interaction = i.as_v4_http().unwrap();
      let result = pact_matching::match_request(interaction.request.clone(),
        req.clone(), &pact.boxed(), &i).await;
      match result {
        Ok(match_result) => Some((interaction.clone(), match_result)),
        Err(err) => {
          error!("Failed to match request for interaction '{}': {}", interaction.description, err);
          None
        }
      }
    })
    .collect::<Vec<(SynchronousHttp, RequestMatchResult)>>().await;
  let mut sorted = match_results.iter().sorted_by(|(_, i1), (_, i2)| {
    Ord::cmp(&i2.score(), &i1.score())
  });
  match sorted.next() {
    Some((interaction, result)) => {
      let request_response_interaction = interaction.as_v4_http().unwrap();
      if result.all_matched() {
        MatchResult::RequestMatch(request_response_interaction.request, request_response_interaction.response, req.clone())
      } else if result.method_or_path_mismatch() {
        MatchResult::RequestNotFound(req.clone())
      } else {
        MatchResult::RequestMismatch(request_response_interaction.request, req.clone(), result.mismatches())
      }
    },
    None => MatchResult::RequestNotFound(req.clone())
  }
}
