//! V4 specification models - HTTP parts for SynchronousHttp

use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use base64::decode;
use bytes::BytesMut;
use log::*;
use serde_json::{json, Value};

use pact_models::bodies::OptionalBody;
use pact_models::content_types::ContentType;
use pact_models::generators::{Generators, generators_from_json, generators_to_json};
use pact_models::json_utils::{headers_from_json, json_to_string};
use pact_models::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use pact_models::PactSpecification;
use pact_models::query_strings::{query_to_json, v3_query_from_json};

use crate::models::{detect_content_type_from_bytes, Request, Response};
use crate::models::v4::calc_content_type;

/// Struct that defines the HTTP request.
#[derive(Debug, Clone, Eq)]
pub struct HttpRequest {
  /// Request method
  pub method: String,
  /// Request path
  pub path: String,
  /// Request query string
  pub query: Option<HashMap<String, Vec<String>>>,
  /// Request headers
  pub headers: Option<HashMap<String, Vec<String>>>,
  /// Request body
  pub body: OptionalBody,
  /// Request matching rules
  pub matching_rules: MatchingRules,
  /// Request generators
  pub generators: Generators
}

impl HttpRequest {
  /// Builds a `HttpRequest` from a JSON `Value` struct.
  pub fn from_json(request_json: &Value) -> Self {
    let method_val = match request_json.get("method") {
      Some(v) => match *v {
        Value::String(ref s) => s.to_uppercase(),
        _ => v.to_string().to_uppercase()
      },
      None => "GET".to_string()
    };
    let path_val = match request_json.get("path") {
      Some(v) => match *v {
        Value::String(ref s) => s.clone(),
        _ => v.to_string()
      },
      None => "/".to_string()
    };
    let query_val = match request_json.get("query") {
      Some(v) => v3_query_from_json(v, &PactSpecification::V4),
      None => None
    };
    let headers = headers_from_json(request_json);
    HttpRequest {
      method: method_val,
      path: path_val,
      query: query_val,
      headers: headers.clone(),
      body: body_from_json(request_json, "body", &headers),
      matching_rules: matchers_from_json(request_json, &None),
      generators: generators_from_json(request_json)
    }
  }

  /// Converts this `HttpRequest` to a `Value` struct.
  pub fn to_json(&self) -> Value {
    let mut json = json!({
      "method": Value::String(self.method.to_uppercase()),
      "path": Value::String(self.path.clone())
    });
    {
      let map = json.as_object_mut().unwrap();

      if let Some(ref query) = self.query {
        map.insert("query".to_string(), query_to_json(query.clone(), &PactSpecification::V4));
      }

      if let Some(ref headers) = self.headers {
        map.insert("headers".to_string(), Value::Object(
          headers.iter().map(|(k, v)| (k.clone(), json!(v))).collect()
        ));
      }

      if let Value::Object(body) = self.body.to_v4_json() {
        map.insert("body".to_string(), Value::Object(body));
      }

      if self.matching_rules.is_not_empty() {
        map.insert("matchingRules".to_string(), matchers_to_json(
          &self.matching_rules.clone(), &PactSpecification::V4));
      }

      if self.generators.is_not_empty() {
        map.insert("generators".to_string(), generators_to_json(
          &self.generators.clone(), &PactSpecification::V4));
      }
    }
    json
  }

  /// Convert this request to a V3 request struct
  pub fn as_v3_request(&self) -> Request {
    Request {
      method: self.method.clone(),
      path: self.path.clone(),
      query: self.query.clone(),
      headers: self.headers.clone(),
      body: self.body.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }

  /// Determine the content type of the request. Returns the content type of the body, otherwise
  /// if a `Content-Type` header is present, the value of that header will be returned.
  /// Otherwise, the body will be inspected.
  pub fn content_type(&self) -> Option<ContentType> {
    calc_content_type(&self.body, &self.headers)
  }
}

impl PartialEq for HttpRequest {
  fn eq(&self, other: &Self) -> bool {
    self.method == other.method && self.path == other.path && self.query == other.query &&
      self.headers == other.headers && self.body == other.body &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }
}

impl Hash for HttpRequest {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.method.hash(state);
    self.path.hash(state);

    if let Some(ref query) = self.query {
      for (k, v) in query {
        k.hash(state);
        v.hash(state);
      }
    }

    if let Some(ref headers) = self.headers {
      for (k, v) in headers {
        k.hash(state);
        v.hash(state);
      }
    }

    self.body.hash(state);
    self.matching_rules.hash(state);
    self.generators.hash(state);
  }
}

pub(crate) fn body_from_json(json: &Value, attr_name: &str, headers: &Option<HashMap<String, Vec<String>>>) -> OptionalBody {
  match json.get(attr_name) {
    Some(body) => match *body {
      Value::Object(ref body_attrs) => {
        match body_attrs.get("content") {
          Some(body_contents) => {
            let content_type = match body_attrs.get("contentType") {
              Some(v) => {
                let content_type_str = json_to_string(v);
                match ContentType::parse(&*content_type_str) {
                  Ok(ct) => Some(ct),
                  Err(err) => {
                    warn!("Failed to parse body content type '{}' - {}", content_type_str, err);
                    None
                  }
                }
              },
              None => {
                warn!("Body has no content type set, will default to any headers or metadata");
                match headers {
                  Some(ref h) => match h.iter().find(|kv| kv.0.to_lowercase() == "content-type") {
                    Some((_, v)) => {
                      match ContentType::parse(v[0].as_str()) {
                        Ok(v) => Some(v),
                        Err(err) => {
                          warn!("Failed to parse body content type '{}' - {}", v[0], err);
                          None
                        }
                      }
                    },
                    None => None
                  },
                  None => None
                }
              }
            };

            let (encoded, encoding) = match body_attrs.get("encoded") {
              Some(v) => match *v {
                Value::String(ref s) => (true, s.clone()),
                Value::Bool(b) => (b, Default::default()),
                _ => (true, v.to_string())
              },
              None => (false, Default::default())
            };

            let body_bytes = if encoded {
              match encoding.as_str() {
                "base64" => {
                  match decode(json_to_string(body_contents)) {
                    Ok(bytes) => bytes,
                    Err(err) => {
                      warn!("Failed to decode base64 encoded body - {}", err);
                      json_to_string(body_contents).into()
                    }
                  }
                },
                "json" => body_contents.to_string().into(),
                _ => {
                  warn!("Unrecognised body encoding scheme '{}', will use the raw body", encoding);
                  json_to_string(body_contents).into()
                }
              }
            } else {
              json_to_string(body_contents).into()
            };

            if body_bytes.is_empty() {
              OptionalBody::Empty
            } else {
              let content_type = content_type.unwrap_or_else(|| {
                detect_content_type_from_bytes(&body_bytes).unwrap_or_default()
              });
              let mut buf = BytesMut::new();
              buf.extend_from_slice(&*body_bytes);
              OptionalBody::Present(buf.freeze(), Some(content_type))
            }
          },
          None => OptionalBody::Missing
        }
      },
      Value::Null => OptionalBody::Null,
      _ => {
        warn!("Body in attribute '{}' from JSON file is not formatted correctly, will load it as plain text", attr_name);
        OptionalBody::Present(body.to_string().into(), None)
      }
    },
    None => OptionalBody::Missing
  }
}

impl Display for HttpRequest {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "HTTP Request ( method: {}, path: {}, query: {:?}, headers: {:?}, body: {} )",
           self.method, self.path, self.query, self.headers, self.body)
  }
}

impl Default for HttpRequest {
  fn default() -> Self {
    HttpRequest {
      method: "GET".into(),
      path: "/".into(),
      query: None,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: MatchingRules::default(),
      generators: Generators::default()
    }
  }
}

/// Struct that defines the HTTP response.
#[derive(Debug, Clone, Eq)]
pub struct HttpResponse {
  /// Response status
  pub status: u16,
  /// Response headers
  pub headers: Option<HashMap<String, Vec<String>>>,
  /// Response body
  pub body: OptionalBody,
  /// Response matching rules
  pub matching_rules: MatchingRules,
  /// Response generators
  pub generators: Generators
}

impl Display for HttpResponse {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "HTTP Response ( status: {}, headers: {:?}, body: {} )", self.status, self.headers,
           self.body)
  }
}

impl Default for HttpResponse {
  fn default() -> Self {
    HttpResponse {
      status: 200,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: MatchingRules::default(),
      generators: Generators::default()
    }
  }
}

impl PartialEq for HttpResponse {
  fn eq(&self, other: &Self) -> bool {
    self.status == other.status && self.headers == other.headers && self.body == other.body &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }
}

impl Hash for HttpResponse {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.status.hash(state);

    if let Some(ref headers) = self.headers {
      for (k, v) in headers {
        k.hash(state);
        v.hash(state);
      }
    }

    self.body.hash(state);
    self.matching_rules.hash(state);
    self.generators.hash(state);
  }
}

impl HttpResponse {
  /// Build an `HttpResponse` from a JSON `Value` struct.
  pub fn from_json(response: &Value) -> Self {
    let status_val = match response.get("status") {
      Some(v) => v.as_u64().unwrap() as u16,
      None => 200
    };
    let headers = headers_from_json(response);
    HttpResponse {
      status: status_val,
      headers: headers.clone(),
      body: body_from_json(response, "body", &headers),
      matching_rules:  matchers_from_json(response, &None),
      generators:  generators_from_json(response)
    }
  }

  /// Converts this response to a `Value` struct.
  pub fn to_json(&self) -> Value {
    let mut json = json!({
      "status" : self.status
    });
    {
      let map = json.as_object_mut().unwrap();

      if let Some(ref headers) = self.headers {
        map.insert("headers".to_string(), Value::Object(
          headers.iter().map(|(k, v)| (k.clone(), json!(v))).collect()
        ));
      }

      if let Value::Object(body) = self.body.to_v4_json() {
        map.insert("body".to_string(), Value::Object(body));
      }

      if self.matching_rules.is_not_empty() {
        map.insert("matchingRules".to_string(), matchers_to_json(
          &self.matching_rules.clone(), &PactSpecification::V4));
      }

      if self.generators.is_not_empty() {
        map.insert("generators".to_string(), generators_to_json(
          &self.generators.clone(), &PactSpecification::V4));
      }
    }
    json
  }

  /// Converts this response to a v3 response struct
  pub fn as_v3_response(&self) -> Response {
    Response {
      status: self.status,
      headers: self.headers.clone(),
      body: self.body.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }

  /// Determine the content type of the response. Returns the content type of the body, otherwise
  /// if a `Content-Type` header is present, the value of that header will be returned.
  /// Otherwise, the body will be inspected.
  pub fn content_type(&self) -> Option<ContentType> {
    calc_content_type(&self.body, &self.headers)
  }
}
