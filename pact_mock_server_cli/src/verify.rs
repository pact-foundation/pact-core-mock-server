use std::sync::Mutex;

use clap::ArgMatches;
use http::StatusCode;
use pact_models::json_utils::json_to_string;
use serde_json::Value;
use tracing::error;

use pact_mock_server::{
  mock_server::MockServer,
  server_manager::ServerManager
};

use crate::handle_error;

pub async fn verify_mock_server(host: &str, port: u16, matches: &ArgMatches, usage: &str) -> Result<(), i32> {
  let mock_server_id = matches.get_one::<String>("mock-server-id");
  let mock_server_port = matches.get_one::<u16>("mock-server-port");
  let (id, id_type) = match (mock_server_id, mock_server_port) {
    (Some(id), _) => (id.clone(), "id"),
    (_, Some(port)) => (port.to_string(), "port"),
    _ => crate::display_error("Either an ID or port must be provided".to_string(), usage)
  };

  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/mockserver/{}/verify", host, port, id);
  let resp = client.post(&url)
    .send().await;
  match resp {
    Ok(result) => {
      let status = result.status();
      if !status.is_success() {
        match status {
          StatusCode::NOT_FOUND => {
            println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.", id_type, id);
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
                    error!("Failed to parse JSON: {}\n{}", err, body);
                    crate::display_error(format!("Failed to parse JSON: {}\n{}", err, body), usage);
                  }
                }
              },
              Err(err) => {
                error!("Failed to parse JSON: {}", err);
                crate::display_error(format!("Failed to parse JSON: {}", err), usage);
              }
            }
          },
          _ => crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status()), usage)
        }
      } else {
        println!("Mock server with {} '{}' verified ok", id, id_type);
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), usage);
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
        .find_mock_server_by_id(&id.to_string(), &|_, ms| {
            ms.unwrap_left().clone()
        })
        .ok_or(format!("No mock server running with id '{}'", id))
}

pub fn validate_id(id: &str, server_manager: &Mutex<ServerManager>) -> Result<MockServer, String> {
    if id.chars().all(|ch| ch.is_digit(10)) {
        validate_port(id.parse::<u16>().unwrap(), server_manager)
    } else {
        validate_uuid(id, server_manager)
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
