use crate::callback_executors::RequestFilterExecutor;
use crate::{ProviderInfo, VerificationOptions, MismatchResult};
use pact_matching::models::message::Message;
use std::collections::HashMap;
use serde_json::{json, Value};
use pact_matching::models::{Request, OptionalBody};
use crate::provider_client::{make_provider_request, provider_client_error_to_string};
use ansi_term::Colour::*;
use pact_matching::match_message;
use ansi_term::ANSIGenericString;

pub async fn verify_message_from_provider<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  interaction: &Message,
  options: &VerificationOptions<F>,
  client: &reqwest::Client,
  verification_context: HashMap<String, Value>
) -> Result<(), MismatchResult> {
  let mut request_body = json!({
    "description": interaction.description
  });
  if !interaction.provider_states.is_empty() {
    if let Some(map) = request_body.as_object_mut() {
      map.insert("providerStates".into(), Value::Array(interaction.provider_states.iter()
        .map(|ps| ps.to_json()).collect()));
    }
  }
  let message_request = Request {
    method: "POST".into(),
    body: OptionalBody::Present(request_body.to_string().as_bytes().to_vec(), Some("application/json".into())),
    .. Request::default()
  };
  match make_provider_request(provider, &message_request, options, client).await {
    Ok(ref actual_response) => {
      let actual = Message {
        contents: actual_response.body.clone(),
        .. Message::default()
      };
      let mismatches = match_message(interaction, &actual);
      if mismatches.is_empty() {
        Ok(())
      } else {
        Err(MismatchResult::Mismatches {
          mismatches,
          expected: Box::new(interaction.clone()),
          actual: Box::new(actual),
          interaction_id: interaction.id.clone()
        })
      }
    },
    Err(err) => {
      Err(MismatchResult::Error(provider_client_error_to_string(err), interaction.id.clone()))
    }
  }
}

pub fn display_message_result(
  errors: &mut Vec<(String, MismatchResult)>,
  interaction: &Message,
  match_result: &Result<(), MismatchResult>,
  description: &String
) {
  match match_result {
    Ok(()) => {
      display_result(Green.paint("OK") //,
        // interaction.response.headers.clone().map(|h| h.iter().map(|(k, v)| {
        //   (k.clone(), v.join(", "), Green.paint("OK"))
        // }).collect()), Green.paint("OK")
      )
    },
    Err(ref err) => match *err {
      MismatchResult::Error(ref err_des, _) => {
        println!("      {}", Red.paint(format!("Request Failed - {}", err_des)));
        errors.push((description.clone(), err.clone()));
      },
      MismatchResult::Mismatches { ref mismatches, .. } => {
        let description = description.to_owned() + " generates a message which ";
        // let header_results = match interaction.response.headers {
        //   Some(ref h) => Some(h.iter().map(|(k, v)| {
        //     (k.clone(), v.join(", "), if mismatches.iter().any(|m| {
        //       match *m {
        //         Mismatch::HeaderMismatch { ref key, .. } => k == key,
        //         _ => false
        //       }
        //     }) {
        //       Red.paint("FAILED")
        //     } else {
        //       Green.paint("OK")
        //     })
        //   }).collect()),
        //   None => None
        // };
        let body_result = if mismatches.iter().any(|m| m.mismatch_type() == "BodyMismatch" ||
          m.mismatch_type() == "BodyTypeMismatch") {
          Red.paint("FAILED")
        } else {
          Green.paint("OK")
        };

        display_result(body_result);
        errors.push((description.clone(), err.clone()));
      }
    }
  }
}

fn display_result(body_result: ANSIGenericString<str>) {
  println!("    generates a message which");
  // if let Some(header_results) = header_results {
  //   println!("      includes headers");
  //   for (key, value, result) in header_results {
  //     println!("        \"{}\" with value \"{}\" ({})", Style::new().bold().paint(key),
  //              Style::new().bold().paint(value), result);
  //   }
  // }
  println!("      has a matching body ({})", body_result);
}
