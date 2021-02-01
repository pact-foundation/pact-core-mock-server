use ansi_term::Colour::*;
use crate::{MismatchResult, display_result};
use pact_matching::models::RequestResponseInteraction;
use pact_matching::Mismatch;

pub fn display_request_response_result(
  errors: &mut Vec<(Option<String>, String, Option<MismatchResult>)>,
  interaction: &RequestResponseInteraction,
  match_result: &Result<Option<String>, MismatchResult>,
  description: &String
) {
  match match_result {
    Ok(id) => {
      display_result(
        interaction.response.status,
        Green.paint("OK"),
        interaction.response.headers.clone().map(|h| h.iter().map(|(k, v)| {
          (k.clone(), v.join(", "), Green.paint("OK"))
        }).collect()), Green.paint("OK")
      );
      errors.push((id.clone(), description.clone(), None));
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
        errors.push((err.interaction_id().clone(), description.clone(), Some(err.clone())));
      },
      MismatchResult::Mismatches { ref mismatches, .. } => {
        let description = description.to_owned() + " returns a response which ";
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
        errors.push((interaction.id.clone(), description.clone(), Some(err.clone())));
      }
    }
  }
}
