//! Functions to verify a Pact file

use ansi_term::Colour::*;
use log::error;
use serde::Serialize;
use serde_json::Value;

use pact_matching::models::{determine_spec_version, MessagePact, parse_meta_data, RequestResponsePact};
use pact_matching::models::v4::V4Pact;
use pact_models::PactSpecification;
use pact_models::verify_json::{json_type_of, PactFileVerificationResult, PactJsonVerifier, ResultLevel};

#[derive(Debug, Clone, Serialize)]
pub struct VerificationResult {
  /// source of the verification
  pub source: String,
  /// results
  pub results: Vec<PactFileVerificationResult>
}

impl VerificationResult {
  pub fn has_errors(&self) -> bool {
    self.results.iter().any(|result| result.level == ResultLevel::ERROR)
  }
}

impl VerificationResult {
  pub fn new(source: &String, results: Vec<PactFileVerificationResult>) -> Self {
    VerificationResult {
      source: source.clone(),
      results: results.clone()
    }
  }
}

pub fn verify_json(pact_json: &Value, spec_version: &PactSpecification, source: &str, strict: bool) -> Vec<PactFileVerificationResult> {
  let spec_version = match spec_version {
    PactSpecification::Unknown => {
      let metadata = parse_meta_data(pact_json);
      determine_spec_version(source, &metadata)
    }
    _ => spec_version.clone()
  };
  match spec_version {
    PactSpecification::V4 => V4Pact::verify_json("/", pact_json, strict),
    _ => match pact_json {
      Value::Object(map) => if map.contains_key("messages") {
        MessagePact::verify_json("/", pact_json, strict)
      } else {
        RequestResponsePact::verify_json("/", pact_json, strict)
      },
      _ => vec![PactFileVerificationResult::new("/", ResultLevel::ERROR,
                                                &format!("Must be an Object, got {}", json_type_of(pact_json)))]
    }
  }
}

pub fn display_results(result: &Vec<VerificationResult>, output_type: &str) -> anyhow::Result<()> {
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
    ResultLevel::ERROR => Red.paint("ERROR"),
    ResultLevel::WARNING => Yellow.paint("WARNING"),
    ResultLevel::NOTICE => Green.paint("OK")
  });

  let mut errors = 0_usize;
  let mut info = 0_usize;
  for (index, result) in results.iter().enumerate() {
    if result.results.is_empty() {
      println!("  {}) {}: {}", index + 1, result.source, Green.paint("OK"));
    } else {
      println!("  {}) {}:\n", index + 1, result.source);
      for (j, r) in result.results.iter().enumerate() {
        match r.level {
          ResultLevel::ERROR => {
            errors += 1;
            println!("    {}.{}) {}: \"{}\" - {}", index + 1, j + 1, Red.paint(r.level.to_string()), r.path, r.message);
          },
          ResultLevel::WARNING => {
            info += 1;
            println!("    {}.{}) {}: \"{}\" - {}", index + 1, j + 1, Yellow.paint(r.level.to_string()), r.path, r.message);
          },
          ResultLevel::NOTICE => {
            println!("    {}.{}) {}: \"{}\" - {}", index + 1, j + 1, r.level, r.path, r.message);
          }
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

#[cfg(test)]
mod tests;
