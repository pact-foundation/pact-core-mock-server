//! Module for fetching documents via HTTP

use reqwest::{Client, Error};
use serde_json::Value;

/// Type of authentication to use
#[derive(Debug, Clone)]
pub enum UrlAuth {
  /// Username and Password
  User(String, Option<String>),
  /// Bearer token
  Token(String)
}

/// Fetches the JSON from a URL
pub fn fetch_json_from_url(url: &String, auth: &Option<UrlAuth>) -> Result<(String, Value), String> {
  let client = Client::new();
  let request = match auth {
    &Some(ref auth) => {
      match auth {
        &UrlAuth::User(ref username, ref password) => client.get(url).basic_auth(username.clone(), password.clone()),
        &UrlAuth::Token(ref token) => client.get(url).bearer_auth(token.clone())
      }
    },
    &None => client.get(url)
  };

  match request.send() {
    Ok(mut res) => if res.status().is_success() {
      let pact_json: Result<Value, Error> = res.json();
      match pact_json {
        Ok(ref json) => Ok((url.clone(), json.clone())),
        Err(err) => Err(format!("Failed to parse JSON - {}", err))
      }
    } else {
      Err(format!("Request failed with status - {}", res.status()))
    },
    Err(err) => Err(format!("Request failed - {}", err))
  }
}
