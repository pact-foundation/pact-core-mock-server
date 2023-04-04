use clap::{App, ArgMatches};
use http::StatusCode;
use serde_json::json;

pub async fn shutdown_mock_server(host: &str, port: u16, matches: &ArgMatches, app: &mut App<'_>) -> Result<(), i32> {
  let mock_server_id = matches.value_of("mock-server-id");
  let mock_server_port = matches.value_of("mock-server-port");
  let id = if let Some(id) = mock_server_id {
    (id, "id")
  } else {
    (mock_server_port.unwrap(), "port")
  };

  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/mockserver/{}", host, port, id.0);
  let resp = client.delete(&url).send().await;
  match resp {
    Ok(result) => {
      if !result.status().is_success() {
        match result.status() {
          StatusCode::NOT_FOUND => {
            println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.", id.1, id.0);
            Err(3)
          },
          _ => crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status()), app)
        }
      } else {
        println!("Mock server with {} '{}' shutdown ok", id.1, id.0);
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), app);
    }
  }
}

pub async fn shutdown_master_server(host: &str, port: u16, matches: &ArgMatches, app: &mut App<'_>) -> Result<(), i32> {
  let client = reqwest::Client::new();
  let server_key = matches.value_of("server-key").unwrap().to_owned();
  let shutdown_period = matches.value_of("period").map(|val| val.parse::<u16>().unwrap_or(100)).unwrap_or(100);
  let url = format!("http://{}:{}/shutdown", host, port);
  let res = client.post(&url)
    .bearer_auth(server_key)
    .json(&json!({ "period": shutdown_period }))
    .send().await;
  match res {
    Ok(result) => {
      if !result.status().is_success() {
        if result.status() == StatusCode::FORBIDDEN {
          crate::display_error(format!("Invalid server key: got response {}", result.status()), app)
        } else {
          crate::display_error(format!("Unexpected response from master mock server '{}': {}",
                                       url, result.status()), app)
        }
      } else {
        println!("Master server shutting down ok");
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), app);
    }
  }
}
