use log::error;
use serde::Serialize;

use pact_models::verify_json::{PactFileVerificationResult, ResultLevel};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VerificationResult {
  /// source of the verification
  pub source: String,
  /// results
  pub results: Vec<PactFileVerificationResult>
}

impl VerificationResult {
  pub(crate) fn has_errors(&self) -> bool {
    self.results.iter().any(|result| result.level == ResultLevel::ERROR)
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
  let overall_result = results.iter().fold(ResultLevel::NOTICE, |acc, result| {
    result.results.iter().fold(acc, |acc, result| {
      match (acc, &result.level) {
        (ResultLevel::NOTICE, ResultLevel::NOTICE) => ResultLevel::NOTICE,
        (ResultLevel::NOTICE, ResultLevel::WARNING) => ResultLevel::WARNING,
        (ResultLevel::NOTICE, ResultLevel::ERROR) => ResultLevel::ERROR,
        (ResultLevel::WARNING, ResultLevel::NOTICE) => ResultLevel::WARNING,
        (ResultLevel::WARNING, ResultLevel::WARNING) => ResultLevel::WARNING,
        (ResultLevel::WARNING, ResultLevel::ERROR) => ResultLevel::ERROR,
        (ResultLevel::ERROR, _) => ResultLevel::ERROR,
      }
    })
  });

  println!("Verification result is {}\n", match overall_result {
    ResultLevel::ERROR => "ERROR",
    ResultLevel::WARNING => "WARNING",
    ResultLevel::NOTICE => "OK"
  });

  let mut errors = 0_usize;
  let mut info = 0_usize;
  for (index, result) in results.iter().enumerate() {
    if result.results.is_empty() {
      println!("  {}) {}: OK", index + 1, result.source);
    } else {
      println!("  {}) {}:\n", index + 1, result.source);
      for (j, r) in result.results.iter().enumerate() {
        println!("    {}.{}) {}: \"{}\" - {}", index + 1, j + 1, r.level, r.path, r.message);

        match r.level {
          ResultLevel::ERROR => errors += 1,
          ResultLevel::WARNING => info += 1,
          _ => {}
        }
      }
    }
    println!()
  }

  println!("\nThere were {} error(s) and {} warning(s) in {} file(s)", errors, info, results.len());

  Ok(())
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
