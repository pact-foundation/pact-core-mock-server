//!
//! The matching module defines how a request is matched
//! against a list of potential interactions.
//!

use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use itertools::Itertools;
use serde_json::json;

use pact_matching::Mismatch;
use pact_matching::models::{Interaction, Request, RequestResponseInteraction, Response};
use pact_matching::s;
use pact_models::PactSpecification;

/// Enum to define a match result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
  /// Match result where the request was successfully matched
  RequestMatch(Request, Response),
  /// Match result where there were a number of mismatches
  RequestMismatch(Request, Vec<Mismatch>),
  /// Match result where the request was not expected
  RequestNotFound(Request),
  /// Match result where an expected request was not received
  MissingRequest(Request)
}

impl MatchResult {
    /// Returns the match key for this mismatch
    pub fn match_key(&self) -> String {
        match self {
            &MatchResult::RequestMatch(_, _) => "Request-Matched",
            &MatchResult::RequestMismatch(_, _) => "Request-Mismatch",
            &MatchResult::RequestNotFound(_) => "Unexpected-Request",
            &MatchResult::MissingRequest(_) => "Missing-Request"
        }.to_string()
    }

    /// Returns true if this match result is a `RequestMatch`
    pub fn matched(&self) -> bool {
        match self {
            &MatchResult::RequestMatch(_, _) => true,
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
            &MatchResult::RequestMatch(_, _) => json!({ "type" : "request-match"}),
            &MatchResult::RequestMismatch(ref request, ref mismatches) => mismatches_to_json(request, mismatches),
            &MatchResult::RequestNotFound(ref req) => json!({
                "type": "request-not-found",
                "method": req.method,
                "path": req.path,
                "request": req.to_json(&PactSpecification::V3)
            }),
            &MatchResult::MissingRequest(ref request) => json!({
                "type": "missing-request",
                "method": request.method,
                "path": request.path,
                "request": request.to_json(&PactSpecification::V3)
            })
        }
    }
}

impl Display for MatchResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MatchResult::RequestMatch(request, _) => {
        write!(f, "Request matched OK - {}", request)
      },
      MatchResult::RequestMismatch(request, mismatches) => {
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

fn mismatches_to_json(request: &Request, mismatches: &Vec<Mismatch>) -> serde_json::Value {
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
pub fn match_request(req: &Request, interactions: Vec<&dyn Interaction>) -> MatchResult {
  let mut match_results = interactions
    .into_iter()
    .filter(|i| i.is_request_response())
    .map(|i| {
      let interaction = i.as_request_response().unwrap();
      (i.clone(), pact_matching::match_request(interaction.request.clone(), req.clone()))
    })
    .sorted_by(|(_, i1), (_, i2)| {
      Ord::cmp(&i2.score(), &i1.score())
    });
  match match_results.next() {
    Some((interaction, result)) => {
      let request_response_interaction = interaction.as_request_response().unwrap();
      if result.all_matched() {
        MatchResult::RequestMatch(request_response_interaction.request, request_response_interaction.response)
      } else if result.method_or_path_mismatch() {
        MatchResult::RequestNotFound(req.clone())
      } else {
        MatchResult::RequestMismatch(request_response_interaction.request, result.mismatches())
      }
    },
    None => MatchResult::RequestNotFound(req.clone())
  }
}
