use clap::ArgMatches;
use std::path::Path;
use pact_matching::models::RequestResponsePact;
use serde_json::Value;

pub fn create_mock_server(host: &str, port: u16, matches: &ArgMatches) -> Result<(), i32> {
  let file = matches.value_of("file").unwrap();
  log::info!("Creating mock server from file {}", file);

  match RequestResponsePact::read_pact(&Path::new(file)) {
    Ok(ref pact) => {
      let url = if matches.is_present("cors") {
        log::info!("Setting mock server to handle CORS pre-flight requests");
        format!("http://{}:{}/?cors=true", host, port)
      } else {
        format!("http://{}:{}/", host, port)
      };
      let client = reqwest::blocking::Client::new();
      let resp = client.post(url.as_str())
        .json(&pact.to_json(pact.spec_version()))
        .send();
      match resp {
        Ok(result) => {
          if result.status().is_success() {
            match result.json::<Value>() {
              Ok(json) => {
                let mock_server = json.get("mockServer").unwrap();
                let id = mock_server.get("id").unwrap().as_str().unwrap();
                let port = mock_server.get("port").unwrap().as_u64().unwrap();
                println!("Mock server {} started on port {}", id, port);
                Ok(())
              },
              Err(err) => {
                log::error!("Failed to parse JSON: {}", err);
                crate::display_error(format!("Failed to parse JSON: {}", err), matches);
              }
            }
          } else {
            crate::display_error(format!("Master mock server returned an error: {}\n{}",
                                         result.status(), result.text().unwrap_or_default()), matches);
          }
        },
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
