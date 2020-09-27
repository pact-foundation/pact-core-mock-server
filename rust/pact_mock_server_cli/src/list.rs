use clap::ArgMatches;
use serde_json::{self, Value, json};
use log::*;
use crate::{display_error, handle_error};

fn json2string(json: Option<&Value>) -> String {
  match json {
    Some(v) => match *v {
      Value::String(ref s) => s.clone(),
      _ => v.to_string()
    },
    None => String::new()
  }
}

#[allow(clippy::print_literal)]
pub async fn list_mock_servers(host: &str, port: u16, matches: &ArgMatches<'_>) -> Result<(), i32> {
  let client = reqwest::Client::new();
  let url = format!("http://{}:{}/", host, port);
  let res = client.get(&url).send().await;

  match res {
    Ok(result) => {
      let status = result.status();
      if status.is_success() {
        match result.json::<Value>().await {
          Ok(json) => {
            let mock_servers_json = json.get("mockServers")
              .ok_or_else(|| handle_error("Invalid JSON received from master server - no mockServers attribute"))?;
            let mock_servers = mock_servers_json.as_array()
              .ok_or_else(|| handle_error("Invalid JSON received from master server - mockServers is not an array"))?;
            let provider_len = mock_servers.iter().fold(0, |acc, ms| {
              let unknown = &json!("<unknown>");
              let provider = ms.get("provider").unwrap_or(unknown)
                .as_str().unwrap_or("<unknown>");
              if provider.len() > acc {
                provider.len()
              } else {
                acc
              }
            });

            println!("{0:36}  {1:5}  {2:3$}  {4}", "Mock Server Id", "Port",
                     "Provider", provider_len, "Verification State");
            for ms in mock_servers {
              let id = json2string(ms.get("id"));
              let port = json2string(ms.get("port"));
              let provider = json2string(ms.get("provider"));
              let status = json2string(ms.get("status"));
              println!("{0}  {1}  {2:3$}  {4}", id, port, provider, provider_len, status);
            };
            Ok(())
          },
          Err(err) => {
            error!("Failed to parse JSON: {}\n", err);
            display_error(format!("Failed to parse JSON: {}", err), matches);
          }
        }
      } else {
        let body = result.text().await.unwrap_or_default();
        display_error(format!("Master mock server returned an error: {}\n{}", status, body), matches);
      }
    },
    Err(err) => {
      display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
    }
  }
}
