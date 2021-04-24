use std::path::Path;

use clap::ArgMatches;
use log::*;
use serde_json::Value;
use itertools::Itertools;
use anyhow::anyhow;

use pact_matching::models::{ReadWritePact, Pact, RequestResponsePact};

use crate::handle_error;

pub async fn create_mock_server(host: &str, port: u16, matches: &ArgMatches<'_>) -> Result<(), i32> {
  let file = matches.value_of("file").unwrap();
  log::info!("Creating mock server from file {}", file);

  match RequestResponsePact::read_pact(&Path::new(file)) {
    Ok(ref pact) => {
      let mut args = vec![];
      if matches.is_present("cors") {
        info!("Setting mock server to handle CORS pre-flight requests");
        args.push("cors=true");
      }
      if matches.is_present("tls") {
        info!("Setting mock server to use TLS");
        args.push("tls=true");
      }
      let url = if args.is_empty() {
        format!("http://{}:{}/", host, port)
      } else {
        format!("http://{}:{}/?{}", host, port, args.iter().join("&"))
      };
      let client = reqwest::Client::new();
      let json = pact.to_json(pact.specification_version())
        .map_err(|err| {
          crate::display_error(format!("Failed to send pact as JSON '{}': {}", file, err), matches);
          -1
        })?;
      let resp = client.post(url.as_str())
        .json(&json)
        .send().await;
      match resp {
        Ok(response) => {
          if response.status().is_success() {
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
                crate::display_error(format!("Failed to parse JSON: {}", err), matches);
              }
            }
          } else {
            crate::display_error(format!("Master mock server returned an error: {}\n{}",
              response.status(), response.text().await.unwrap_or_default()), matches);
          }
        }
        Err(err) => {
            crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
        }
      }
    },
    Err(err) => {
      crate::display_error(format!("Failed to load pact file '{}': {}", file, err), matches);
    }
  }
}
