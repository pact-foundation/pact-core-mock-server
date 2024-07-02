use comfy_table::presets::UTF8_FULL;
use comfy_table::Table;
use serde_json::{self, Value};
use tracing::error;

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
pub async fn list_mock_servers(host: &str, port: u16, usage: &str) -> Result<(), i32> {
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

            let mut table = Table::new();
            table
              .load_preset(UTF8_FULL)
              .set_header(vec!["Mock Server Id", "Port", "Provider", "Verification State"]);
            for ms in mock_servers {
              let id = json2string(ms.get("id"));
              let port = json2string(ms.get("port"));
              let provider = json2string(ms.get("provider"));
              let status = json2string(ms.get("status"));
              table.add_row(vec![id.as_str(), port.as_str(), provider.as_str(), status.as_str()]);
            };
            println!("{table}");
            Ok(())
          },
          Err(err) => {
            error!("Failed to parse JSON: {}\n", err);
            display_error(format!("Failed to parse JSON: {}", err), usage, 10);
          }
        }
      } else {
        let body = result.text().await.unwrap_or_default();
        display_error(format!("Master mock server returned an error: {}\n{}", status, body), usage, 10);
      }
    },
    Err(err) => {
      display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), usage, 10);
    }
  }
}
