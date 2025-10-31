use std::path::Path;

use clap::ArgMatches;
use itertools::Itertools;
use pact_models::pact::{Pact, ReadWritePact};
use pact_models::sync_pact::RequestResponsePact;
use serde_json::Value;
use tracing::{debug, error, info};

use crate::handle_error;

pub async fn create_mock_server(host: &str, port: u16, matches: &ArgMatches, usage: &str) -> Result<(), i32> {
  let file = matches.get_one::<String>("file").unwrap();
  info!("Creating mock server from file {}", file);

  match RequestResponsePact::read_pact(Path::new(file)) {
    Ok(ref pact) => {
      let mut args = Vec::<String>::new();
      if matches.get_flag("cors") {
        info!("Setting mock server to handle CORS pre-flight requests");
        args.push("cors=true".to_string());
      }
      if let Some(specification) = matches.get_one::<String>("specification") {
        info!("Setting mock server to use pact specification {}", specification);
        let spec_arg = format!("specification={}", specification);
        args.push(spec_arg);
      }
      if matches.get_flag("tls") {
        info!("Setting mock server to use TLS");
        args.push("tls=true".to_string());
      }
      let url = if args.is_empty() {
        format!("http://{}:{}/", host, port)
      } else {
        format!("http://{}:{}/?{}", host, port, args.iter().join("&"))
      };
      let client = reqwest::Client::new();
      let json = match pact.to_json(pact.specification_version()) {
        Ok(json) => json,
        Err(err) => {
          crate::display_error(format!("Failed to send pact as JSON '{}': {}", file, err), usage, 21);
        }
      };
      let resp = client.post(url.as_str())
        .json(&json)
        .send().await;
      match resp {
        Ok(response) => {
          let status_code = response.status();
          let content_length = response.content_length();
          if status_code.is_success() {
            match response.json::<Value>().await {
              Ok(json) => {
                debug!("Got response from master server: {:?}", json);
                let mock_server = json.get("mockServer")
                  .ok_or_else(|| handle_error("Invalid JSON received from master server - no mockServer attribute"))?;
                let id = mock_server.get("id")
                  .ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer has no id attribute"))?
                  .as_str().ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer id attribute is not a string"))?;
                let port = mock_server.get("port")
                  .ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer has no port attribute"))?
                  .as_u64().ok_or_else(|| handle_error("Invalid JSON received from master server - mockServer port attribute is not a number"))?;
                println!("Mock server {} started on port {}", id, port);
                Ok(())
              },
              Err(err) => {
                error!("Failed to parse JSON: {}", err);
                error!("Response:    {}", status_code);
                error!("Body length: {:?}", content_length);
                crate::display_error(format!("Failed to parse JSON: {}", err), usage, 20);
              }
            }
          } else {
            crate::display_error(format!("Master mock server returned an error: {}\n{}",
                                         status_code, response.text().await.unwrap_or_default()), usage, 20);
          }
        }
        Err(err) => {
            crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), usage, 20);
        }
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to load pact file '{}': {}", file, err), usage, 20);
    }
  }
}
