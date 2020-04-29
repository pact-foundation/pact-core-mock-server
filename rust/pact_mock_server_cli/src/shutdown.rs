use clap::ArgMatches;
use hyper::{
  Client, Url, status::*, header::{Authorization, Bearer}
};
use serde_json::json;

pub fn shutdown_mock_server(host: &str, port: u16, matches: &ArgMatches) -> Result<(), i32> {
    let mock_server_id = matches.value_of("mock-server-id");
    let mock_server_port = matches.value_of("mock-server-port");
    let id = if let Some(id) = mock_server_id {
      (id, "id")
    } else {
      (mock_server_port.unwrap(), "port")
    };

    let client = Client::new();
    let url = Url::parse(format!("http://{}:{}/mockserver/{}", host, port, id.0)
        .as_str()).unwrap();
    let res = client.delete(url.clone()).send();

    match res {
        Ok(result) => {
            if !result.status.is_success() {
                match result.status {
                    StatusCode::NotFound => {
                        println!("No mock server found with {} '{}', use the 'list' command to get a list of available mock servers.",
                            id.1, id.0);
                        Err(3)
                    },
                    _ => crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status), matches)
                }
            } else {
                println!("Mock server with {} '{}' shutdown ok", id.1, id.0);
                Ok(())
            }
        },
        Err(err) => {
            crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
        }
    }
}

pub fn shutdown_master_server(host: &str, port: u16, matches: &ArgMatches) -> Result<(), i32> {
  let client = Client::new();
  let url = Url::parse(format!("http://{}:{}/shutdown", host, port)
    .as_str()).unwrap();
  let server_key = matches.value_of("server-key").unwrap().to_owned();
  let shutdown_period = matches.value_of("period").map(|val| val.parse::<u16>().unwrap_or(100)).unwrap_or(100);
  let res = client.post(url.clone())
    .header(Authorization(Bearer { token: server_key }))
    .body(&json!({ "period": shutdown_period }).to_string())
    .send();

  match res {
    Ok(result) => {
      if !result.status.is_success() {
        crate::display_error(format!("Unexpected response from master mock server '{}': {}", url, result.status), matches)
      } else {
        println!("Master server shutting down ok");
        Ok(())
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
    }
  }
}
