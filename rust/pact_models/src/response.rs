use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use base64::encode;
use log::warn;
use maplit::hashmap;
use serde_json::{Value, json};

use crate::{DifferenceType, PactSpecification};
use crate::bodies::OptionalBody;
use crate::generators::{Generators, generators_from_json, generators_to_json};
use crate::http_parts::HttpPart;
use crate::json_utils::{body_from_json, headers_from_json, headers_to_json};
use crate::matchingrules::{matchers_from_json, matchers_to_json, MatchingRules};
use crate::v4::http_parts::HttpResponse;
use std::str::from_utf8;

/// Struct that defines the response.
#[derive(Debug, Clone, Eq)]
pub struct Response {
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

impl Response {

  /// Build a `Response` from a `Value` struct.
  pub fn from_json(response: &Value, _: &PactSpecification) -> Response {
    let status_val = match response.get("status") {
      Some(v) => v.as_u64().unwrap() as u16,
      None => 200
    };
    let headers = headers_from_json(response);
    Response {
      status: status_val,
      headers: headers.clone(),
      body: body_from_json(response, "body", &headers),
      matching_rules:  matchers_from_json(response, &Some("responseMatchingRules".to_string())),
      generators:  generators_from_json(response)
    }
  }

  /// Returns a default response: Status 200
  #[deprecated(since="0.5.4", note="please use `default()` from the standard Default trait instead")]
  pub fn default_response() -> Response {
    Response::default()
  }

  /// Converts this response to a `Value` struct.
  #[allow(unused_variables)]
  pub fn to_json(&self, spec_version: &PactSpecification) -> Value {
    let mut json = json!({
        "status" : json!(self.status)
      });
    {
      let map = json.as_object_mut().unwrap();
      if self.headers.is_some() {
        map.insert("headers".to_string(), headers_to_json(&self.headers.clone().unwrap()));
      }
      match self.body {
        OptionalBody::Present(ref body, _) => {
          if self.content_type().unwrap_or_default().is_json() {
            match serde_json::from_slice(body) {
              Ok(json_body) => { map.insert("body".to_string(), json_body); },
              Err(err) => {
                warn!("Failed to parse json body: {}", err);
                map.insert("body".to_string(), Value::String(encode(body)));
              }
            }
          } else {
            match from_utf8(body) {
              Ok(s) => map.insert("body".to_string(), Value::String(s.to_string())),
              Err(_) => map.insert("body".to_string(), Value::String(encode(body)))
            };
          }
        },
        OptionalBody::Empty => { map.insert("body".to_string(), Value::String("".to_string())); },
        OptionalBody::Missing => (),
        OptionalBody::Null => { map.insert("body".to_string(), Value::Null); }
      }
      if self.matching_rules.is_not_empty() {
        map.insert("matchingRules".to_string(), matchers_to_json(
          &self.matching_rules.clone(), spec_version));
      }
      if self.generators.is_not_empty() {
        map.insert("generators".to_string(), generators_to_json(
          &self.generators.clone(), spec_version));
      }
    }
    json
  }

  /// Return a description of all the differences from the other response
  pub fn differences_from(&self, other: &Response) -> Vec<(DifferenceType, String)> {
    let mut differences = vec![];
    if self.status != other.status {
      differences.push((DifferenceType::Status, format!("Response status {} != {}", self.status, other.status)));
    }
    if self.headers != other.headers {
      differences.push((DifferenceType::Headers, format!("Response headers {:?} != {:?}", self.headers, other.headers)));
    }
    if self.body != other.body {
      differences.push((DifferenceType::Body, format!("Response body '{:?}' != '{:?}'", self.body, other.body)));
    }
    if self.matching_rules != other.matching_rules {
      differences.push((DifferenceType::MatchingRules, format!("Response matching rules {:?} != {:?}", self.matching_rules, other.matching_rules)));
    }
    differences
  }

  /// Convert this response to V4 format
  pub fn as_v4_response(&self) -> HttpResponse {
    HttpResponse {
      status: self.status,
      headers: self.headers.clone(),
      body: self.body.clone(),
      matching_rules: self.matching_rules.clone(),
      generators: self.generators.clone()
    }
  }
}

impl HttpPart for Response {
  fn headers(&self) -> &Option<HashMap<String, Vec<String>>> {
    &self.headers
  }

  fn headers_mut(&mut self) -> &mut HashMap<String, Vec<String>> {
    if self.headers.is_none() {
      self.headers = Some(hashmap!{});
    }
    self.headers.as_mut().unwrap()
  }

  fn body(&self) -> &OptionalBody {
    &self.body
  }

  fn matching_rules(&self) -> &MatchingRules {
    &self.matching_rules
  }

  fn generators(&self) -> &Generators {
    &self.generators
  }

  fn lookup_content_type(&self) -> Option<String> {
    self.lookup_header_value(&"content-type".to_string())
  }
}

impl Hash for Response {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.status.hash(state);
    if self.headers.is_some() {
      for (k, v) in self.headers.clone().unwrap() {
        k.hash(state);
        v.hash(state);
      }
    }
    self.body.hash(state);
    self.matching_rules.hash(state);
    self.generators.hash(state);
  }
}

impl PartialEq for Response {
  fn eq(&self, other: &Self) -> bool {
    self.status == other.status && self.headers == other.headers && self.body == other.body &&
      self.matching_rules == other.matching_rules && self.generators == other.generators
  }

  fn ne(&self, other: &Self) -> bool {
    self.status != other.status || self.headers != other.headers || self.body != other.body ||
      self.matching_rules != other.matching_rules || self.generators != other.generators
  }
}

impl Display for Response {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Response ( status: {}, headers: {:?}, body: {} )", self.status, self.headers,
           self.body)
  }
}

impl Default for Response {
  fn default() -> Self {
    Response {
      status: 200,
      headers: None,
      body: OptionalBody::Missing,
      matching_rules: MatchingRules::default(),
      generators: Generators::default()
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::PactSpecification;
  use crate::response::Response;

  #[test]
  fn response_from_json_defaults_to_status_200() {
    let response_json : serde_json::Value = serde_json::from_str(r#"
      {
          "headers": {}
      }
     "#).unwrap();
    let response = Response::from_json(&response_json, &PactSpecification::V1_1);
    assert_eq!(response.status, 200);
  }

}
