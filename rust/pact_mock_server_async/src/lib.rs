#[allow(unused_imports)] extern crate p_macro;
extern crate pact_matching;
extern crate serde_json;
extern crate hyper;
extern crate futures;
extern crate tokio;
#[macro_use] extern crate log;
extern crate itertools;
#[macro_use] extern crate lazy_static;

mod server;
mod server_manager;

use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};
use pact_matching::Mismatch;
use pact_matching::s;
use std::ffi::CString;
use std::sync::Mutex;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::io::{self, Read, Write};
use serde_json::json;
use futures::future::Future;

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

lazy_static! {
    static ref MANAGER: Mutex<server_manager::ServerManager> = Mutex::new(
        server_manager::ServerManager::new()
    );
}

/// Starts a mock server with the given ID, pact and port number. The ID needs to be unique. A port
/// number of 0 will result in an auto-allocated port by the operating system. Returns the port
/// that the mock server is running on wrapped in a `Result`.
///
/// # Errors
///
/// An error with a message will be returned in the following conditions:
///
/// - If a mock server is not able to be started
pub fn start_mock_server(id: String, pact: Pact, port: i32) -> Result<i32, String> {
    MANAGER.lock().unwrap()
        .start_mock_server(id, pact, port as u16)
        .map(|port| port as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
}