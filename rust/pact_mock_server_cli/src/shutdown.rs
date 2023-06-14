use clap::ArgMatches;
use http::StatusCode;
use serde_json::json;

pub async fn shutdown_mock_server(host: &str, port: u16, matches: &ArgMatches, usage: &str) -> Result<(), i32> {
  let mock_server_id = matches.get_one::<String>("mock-server-id");
  let mock_server_port = matches.get_one::<u16>("mock-server-port");
  let (id, id_type) = match (mock_server_id, mock_server_port) {
    (Some(id), _) => (id.clone(), "id"),
    (_, Some(port)) => (port.to_string(), "port"),
    _ => crate::display_error("Either an ID or port must be provided".to_string(), usage)
  };

  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/mockserver/{}", host, port, id);
  let resp = client.delete(&url).send().await;
  match resp {
    Ok(result) => {
      if !result.status().is_success() {
        match result.status() {
          StatusCode::NOT_FOUND => {
            println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.", id_type, id);
            Err(3)
          },
          _ => crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status()), usage)
        }
      } else {
        println!("Mock server with {} '{}' shutdown ok", id_type, id);
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), usage);
    }
  }
}

pub async fn shutdown_master_server(host: &str, port: u16, matches: &ArgMatches, usage: &str) -> Result<(), i32> {
  let client = reqwest::Client::new();
  let server_key = matches.get_one::<String>("server-key").unwrap().to_owned();
  let shutdown_period = matches.get_one::<String>("period").map(|val| val.parse::<u16>().unwrap_or(100)).unwrap_or(100);
  let url = format!("http://{}:{}/shutdown", host, port);
  let res = client.post(&url)
    .bearer_auth(server_key)
    .json(&json!({ "period": shutdown_period }))
    .send().await;
  match res {
    Ok(result) => {
      if !result.status().is_success() {
        if result.status() == StatusCode::FORBIDDEN {
          crate::display_error(format!("Invalid server key: got response {}", result.status()), usage)
        } else {
          crate::display_error(format!("Unexpected response from master mock server '{}': {}",
            url, result.status()), usage)
        }
      } else {
        println!("Master server shutting down ok");
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), usage);
    }
  }
}
