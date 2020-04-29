use clap::ArgMatches;
use hyper::Client;
use hyper::Url;
use std::io::prelude::*;
use serde_json::{self, Value};

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
pub fn list_mock_servers(host: &str, port: u16, matches: &ArgMatches) -> Result<(), i32> {
    let client = Client::new();
    let url = Url::parse(format!("http://{}:{}/", host, port).as_str()).unwrap();
    let res = client.get(url.clone()).send();

    match res {
        Ok(mut result) => {
            let mut body = String::new();
            result.read_to_string(&mut body).unwrap();
            if result.status.is_success() {
                let json_result: Result<serde_json::Value, _> = serde_json::from_str(body.as_str());
                match json_result {
                    Ok(json) => {
                        let mock_servers_json = json.get("mockServers").unwrap();
                        let mock_servers = mock_servers_json.as_array().unwrap();
                        let provider_len = mock_servers.iter().fold(0, |acc, ref ms| {
                            let provider = ms.get("provider").unwrap().to_string();
                            if provider.len() > acc {
                                provider.len()
                            } else {
                                acc
                            }
                        });

                        println!("{0:32}  {1:5}  {2:3$}  {4}", "Mock Server Id", "Port",
                            "Provider", provider_len, "Verification State");
                        for ms in mock_servers {
                            let id = json2string(ms.get("id"));
                            let port = json2string(ms.get("port"));
                            let provider = json2string(ms.get("provider"));
                            let status = json2string(ms.get("status"));
                            println!("{0}  {1}   {2:3$}  {4}", id, port, provider, provider_len, status);
                        };
                        Ok(())
                    },
                    Err(err) => {
                        log::error!("Failed to parse JSON: {}\n{}", err, body);
                        crate::display_error(format!("Failed to parse JSON: {}\n{}", err, body), matches);
                    }
                }
            } else {
                crate::display_error(format!("Master mock server returned an error: {}\n{}", result.status, body), matches);
            }
        },
        Err(err) => {
            crate::display_error(format!("Failed to connect to the master mock server '{}': {}", url, err), matches);
        }
    }
}
