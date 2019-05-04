#[allow(unused_imports)] extern crate p_macro;
extern crate pact_matching;
extern crate serde_json;
extern crate hyper;
extern crate futures;
extern crate tokio;
extern crate log;
extern crate itertools;

mod server;

use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};
use pact_matching::Mismatch;
use pact_matching::s;
use serde_json::json;

/// Enum to define a match result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    /// Match result where the request was sucessfully matched
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

fn mismatches_to_json(request: &Request, mismatches: &Vec<Mismatch>) -> serde_json::Value {
    json!({
        s!("type") : json!("request-mismatch"),
        s!("method") : json!(request.method),
        s!("path") : json!(request.path),
        s!("mismatches") : mismatches.iter().map(|m| m.to_json()).collect::<serde_json::Value>()
    })
}


pub fn run_server_test() {
    let pact = pact_matching::models::Pact::default();

    let f = server::start("yo".into(), pact, 0, futures::future::done(Ok(())));
}