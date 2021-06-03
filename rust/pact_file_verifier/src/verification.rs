use log::error;
use serde::Serialize;

use pact_models::verify_json::{PactFileVerificationResult, PactFileVerificationResultLevel};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VerificationResult {
  /// source of the verification
  pub source: String,
  /// results
  pub results: Vec<PactFileVerificationResult>
}

impl VerificationResult {
  pub(crate) fn has_errors(&self) -> bool {
    self.results.iter().any(|result| result.level == PactFileVerificationResultLevel::Error)
  }
}

impl VerificationResult {
  pub(crate) fn new(source: &String, results: Vec<PactFileVerificationResult>) -> Self {
    VerificationResult {
      source: source.clone(),
      results: results.clone()
    }
  }
}

pub(crate) fn display_results(result: &Vec<VerificationResult>, output_type: &str) -> anyhow::Result<()> {
  if output_type == "json" {
    generate_json_output(result)
  } else {
    display_output(result)
  }
}

fn display_output(results: &Vec<VerificationResult>) -> anyhow::Result<()> {
  todo!()
}

fn generate_json_output(results: &Vec<VerificationResult>) -> anyhow::Result<()> {
  match serde_json::to_string_pretty(&results) {
    Ok(json) => {
      println!("{}", json);
      Ok(())
    },
    Err(err) => {
      error!("ERROR: Failed to generate JSON - {}", err);
      Err(err.into())
    }
  }
}
