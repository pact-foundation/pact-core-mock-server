//! Module for fetching documents via HTTP

use reqwest::Error;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fmt::{Display, Formatter};

/// Type of authentication to use
#[derive(Debug, Clone)]
pub enum HttpAuth {
  /// Username and Password
  User(String, Option<String>),
  /// Bearer token
  Token(String)
}

/// Fetches the JSON from a URL
pub fn fetch_json_from_url(url: &String, auth: &Option<HttpAuth>) -> Result<(String, Value), String> {
  let client = Client::new();
  let request = match auth {
    &Some(ref auth) => {
      match auth {
        &HttpAuth::User(ref username, ref password) => client.get(url).basic_auth(username.clone(), password.clone()),
        &HttpAuth::Token(ref token) => client.get(url).bearer_auth(token.clone())
      }
    },
    &None => client.get(url)
  };

  match request.send() {
    Ok(res) => if res.status().is_success() {
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

impl Display for HttpAuth {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match *self {
      HttpAuth::Token(ref t) => write!(f, "Token({}****)", t.get(0..5).unwrap_or("****")),
      HttpAuth::User(ref u, ref p) => {
        if let Some(pass) = p {
          write!(f, "User({}, {}****)", u, pass.get(0..5).unwrap_or("****"))
        } else {
          write!(f, "User({}, [no password])", u)
        }
      },
    }
  }
}