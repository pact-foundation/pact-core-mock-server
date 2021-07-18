use ansi_term::Colour::*;

use pact_matching::Mismatch;
use pact_models::sync_interaction::RequestResponseInteraction;

use crate::{display_result, MismatchResult};

pub fn display_request_response_result(
  interaction: &RequestResponseInteraction,
  match_result: &Result<Option<String>, MismatchResult>) {
  match match_result {
    Ok(_) => {
      display_result(
        interaction.response.status,
        Green.paint("OK"),
        interaction.response.headers.clone().map(|h| h.iter().map(|(k, v)| {
          (k.clone(), v.join(", "), Green.paint("OK"))
        }).collect()), Green.paint("OK")
      );
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
      },
      MismatchResult::Mismatches { ref mismatches, .. } => {
        let status_result = if mismatches.iter().any(|m| m.mismatch_type() == "StatusMismatch") {
          Red.paint("FAILED")
        } else {
          Green.paint("OK")
        };
        let header_results = match interaction.response.headers {
          Some(ref h) => Some(h.iter().map(|(k, v)| {
            (k.clone(), v.join(", "), if mismatches.iter().any(|m| {
              match *m {
                Mismatch::HeaderMismatch { ref key, .. } => k == key,
                _ => false
              }
            }) {
              Red.paint("FAILED")
            } else {
              Green.paint("OK")
            })
          }).collect()),
          None => None
        };
        let body_result = if mismatches.iter().any(|m| m.mismatch_type() == "BodyMismatch" ||
          m.mismatch_type() == "BodyTypeMismatch") {
          Red.paint("FAILED")
        } else {
          Green.paint("OK")
        };

        display_result(interaction.response.status, status_result, header_results, body_result);
      }
    }
  }
}
