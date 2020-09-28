use clap::ArgMatches;
use std::sync::Mutex;
use pact_mock_server::{
    server_manager::ServerManager,
    mock_server::MockServer
};
use pact_matching::s;
use http::StatusCode;
use serde_json::Value;
use crate::handle_error;
use pact_matching::models::json_utils::json_to_string;

pub async fn verify_mock_server(host: &str, port: u16, matches: &ArgMatches<'_>) -> Result<(), i32> {
  let mock_server_id = matches.value_of("mock-server-id");
  let mock_server_port = matches.value_of("mock-server-port");
  let id = if let Some(id) = mock_server_id {
    (id, "id")
  } else {
    (mock_server_port.unwrap(), "port")
  };

  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/mockserver/{}/verify", host, port, id.0);
  let resp = client.post(&url)
    .send().await;
  match resp {
    Ok(result) => {
      let status = result.status();
      if !status.is_success() {
        match status {
          StatusCode::NOT_FOUND => {
            println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.", id.1, id.0);
            Err(3)
          },
          StatusCode::UNPROCESSABLE_ENTITY => {
            match result.text().await {
              Ok(body) => {
                match serde_json::from_str::<Value>(body.as_str()) {
                  Ok(json) => {
                    let mock_server = json.get("mockServer")
                      .ok_or_else(|| handle_error("Invalid JSON received from master server - no mockServer attribute"))?;
                    let id = mock_server.get("id")
                      .ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer has no id attribute"))?
                      .as_str().ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer id attribute is not a string"))?;
                    let port = mock_server.get("port")
                      .ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer has no port attribute"))?
                      .as_u64().ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer port attribute is not a number"))?;
                    display_verification_errors(id, port, &json);
                    Err(2)
                  },
                  Err(err) => {
                    log::error!("Failed to parse JSON: {}\n{}", err, body);
                    crate::display_error(format!("Failed to parse JSON: {}\n{}", err, body), matches);
                  }
                }
              },
              Err(err) => {
                log::error!("Failed to parse JSON: {}", err);
                crate::display_error(format!("Failed to parse JSON: {}", err), matches);
              }
            }
          },
          _ => crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status()), matches)
        }
      } else {
        println!("Mock server with {} '{}' verified ok", id.1, id.0);
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
    }
  }
}

fn validate_port(port: u16, server_manager: &Mutex<ServerManager>) -> Result<MockServer, String> {
    server_manager.lock().unwrap()
        .find_mock_server_by_port_mut(port, &|ms| {
            ms.clone()
        })
        .ok_or(format!("No mock server running with port '{}'", port))
}

fn validate_uuid(id: &str, server_manager: &Mutex<ServerManager>) -> Result<MockServer, String> {
    server_manager.lock().unwrap()
        .find_mock_server_by_id(&id.to_string(), &|ms| {
            ms.clone()
        })
        .ok_or(format!("No mock server running with id '{}'", id))
}

pub fn validate_id(id: &str, server_manager: &Mutex<ServerManager>) -> Result<MockServer, String> {
    if id.chars().all(|ch| ch.is_digit(10)) {
        validate_port(id.parse::<u16>().unwrap(), server_manager)
    } else {
        validate_uuid(&s!(id), server_manager)
    }
}

fn display_verification_errors(id: &str, port: u64, json: &serde_json::Value) {
  let mismatches = json.get("mismatches").unwrap().as_array().unwrap();
  println!("Mock server {}/{} failed verification with {} errors\n", id, port, mismatches.len());

  for (i, mismatch) in mismatches.iter().enumerate() {
    match json_to_string(mismatch.get("type").unwrap()).as_str() {
      "missing-request" => {
        let request = mismatch.get("request").unwrap();
        println!("{} - Expected request was not received - {}", i, request)
      },
      "request-not-found" => {
        let request = mismatch.get("request").unwrap();
        println!("{} - Received a request that was not expected - {}", i, request)
      },
      "request-mismatch" => {
        let path = mismatch.get("path").unwrap().to_string();
        let method = mismatch.get("method").unwrap().to_string();
        println!("{} - Received a request that did not match with expected - {} {}", i, method, path);
        let request_mismatches = mismatch.get("mismatches").unwrap().as_array().unwrap();
        for request_mismatch in request_mismatches {
          println!("        {}", request_mismatch.get("mismatch").unwrap().to_string())
        }
      },
      _ => println!("{} - Unknown failure - {}", i, mismatch),
    }
  }
}
