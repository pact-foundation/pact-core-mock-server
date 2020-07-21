//!
//! The matching module defines how a request is matched
//! against a list of potential interactions.
//!

use pact_matching::models::{Interaction, Request, PactSpecification};
use pact_matching::Mismatch;
use pact_matching::s;
use serde_json::json;
use itertools::Itertools;
use std::fmt::{Display, Formatter};

/// Enum to define a match result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    /// Match result where the request was successfully matched
    RequestMatch(Interaction),
    /// Match result where there were a number of mismatches
    RequestMismatch(Interaction, Vec<Mismatch>),
    /// Match result where the request was not expected
    RequestNotFound(Request),
    /// Match result where an expected request was not received
    MissingRequest(Interaction)
}

impl MatchResult {
    /// Returns the match key for this mismatch
    pub fn match_key(&self) -> String {
        match self {
            &MatchResult::RequestMatch(_) => s!("Request-Matched"),
            &MatchResult::RequestMismatch(_, _) => s!("Request-Mismatch"),
            &MatchResult::RequestNotFound(_) => s!("Unexpected-Request"),
            &MatchResult::MissingRequest(_) => s!("Missing-Request")
        }
    }

    /// Returns true if this match result is a `RequestMatch`
    pub fn matched(&self) -> bool {
        match self {
            &MatchResult::RequestMatch(_) => true,
            _ => false
        }
    }

    /// Converts this match result to a `Value` struct
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            &MatchResult::RequestMatch(_) => json!({ s!("type") : s!("request-match")}),
            &MatchResult::RequestMismatch(ref interaction, ref mismatches) => mismatches_to_json(&interaction.request, mismatches),
            &MatchResult::RequestNotFound(ref req) => json!({
                "type": json!("request-not-found"),
                "method": json!(req.method),
                "path": json!(req.path),
                "request": req.to_json(&PactSpecification::V3)
            }),
            &MatchResult::MissingRequest(ref interaction) => json!({
                "type": json!("missing-request"),
                "method": json!(interaction.request.method),
                "path": json!(interaction.request.path),
                "request": interaction.request.to_json(&PactSpecification::V3)
            })
        }
    }
}

impl Display for MatchResult {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MatchResult::RequestMatch(interaction) => {
        write!(f, "Request matched OK - {}", interaction.request)
      },
      MatchResult::RequestMismatch(interaction, mismatches) => {
        write!(f, "Request did not match - {}", interaction.request)?;
        for (i, mismatch) in mismatches.iter().enumerate() {
          write!(f, "    {}) {}", i, mismatch)?;
        }
        Ok(())
      },
      MatchResult::RequestNotFound(request) => {
        write!(f, "Request was not expected - {}", request)
      },
      MatchResult::MissingRequest(interaction) => {
        write!(f, "Request was not received - {}", interaction.request)
      }
    }
  }
}

fn mismatches_to_json(request: &Request, mismatches: &Vec<Mismatch>) -> serde_json::Value {
    json!({
        s!("type") : json!("request-mismatch"),
        s!("method") : json!(request.method),
        s!("path") : json!(request.path),
        s!("mismatches") : mismatches.iter().map(|m| m.to_json()).collect::<serde_json::Value>()
    })
}

///
/// Matches a request against a list of interactions
///
pub fn match_request(req: &Request, interactions: &Vec<Interaction>) -> MatchResult {
  let mut match_results = interactions
    .into_iter()
    .map(|i| (i.clone(), pact_matching::match_request_result(i.request.clone(), req.clone())))
    .sorted_by(|(_, i1), (_, i2)| {
      Ord::cmp(&i2.score(), &i1.score())
    });
  match match_results.next() {
    Some(res) => {
      if res.1.all_matched() {
        MatchResult::RequestMatch(res.0.clone())
      } else if res.1.method_or_path_mismatch() {
        MatchResult::RequestNotFound(req.clone())
      } else {
        MatchResult::RequestMismatch(res.0.clone(), res.1.mismatches())
      }
    },
    None => MatchResult::RequestNotFound(req.clone())
  }
}
