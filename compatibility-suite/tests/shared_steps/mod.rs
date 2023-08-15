use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use bytes::Bytes;
use cucumber::gherkin::Table;
use cucumber::Parameter;
use lazy_static::lazy_static;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, JSON, TEXT, XML};
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::json_utils::json_to_string;
use pact_models::matchingrules::matchers_from_json;
use pact_models::query_strings::parse_query_string;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::xml_utils::parse_bytes;
use regex::Regex;
use serde_json::{json, Value};
use sxd_document::dom::Element;
use uuid::Uuid;

pub mod consumer;
pub mod provider;

lazy_static! {
  static ref INT_REGEX: Regex = Regex::new(r"\d+").unwrap();
  static ref DEC_REGEX: Regex = Regex::new(r"\d+\.\d+").unwrap();
  static ref HEX_REGEX: Regex = Regex::new(r"[a-fA-F0-9]+").unwrap();
  static ref STR_REGEX: Regex = Regex::new(r"\d{1,8}").unwrap();
  static ref DATE_REGEX: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}").unwrap();
  static ref TIME_REGEX: Regex = Regex::new(r"\d{2}:\d{2}:\d{2}").unwrap();
  static ref DATETIME_REGEX: Regex = Regex::new(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{1,9}").unwrap();
}

#[derive(Debug, Default, Parameter)]
#[param(name = "numType", regex = "first|second|third")]
pub struct IndexType(usize);

impl IndexType {
  pub fn val(&self) -> usize {
    self.0
  }
}

impl FromStr for IndexType {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "first" => Ok(IndexType(0)),
      "second" => Ok(IndexType(1)),
      "third" => Ok(IndexType(2)),
      _ => Err(anyhow!("{} is not a valid index type", s))
    }
  }
}

pub fn setup_common_interactions(table: &Table) -> Vec<RequestResponseInteraction> {
  let headers = table.rows.first().unwrap().iter()
    .enumerate()
    .map(|(index, h)| (h.clone(), index))
    .collect::<HashMap<String, usize>>();
  let mut interactions = vec![];
  for (row, values) in table.rows.iter().skip(1).enumerate() {
    let mut interaction = RequestResponseInteraction {
      description: format!("Interaction {}", row),
      id: Some(format!("ID{}", row + 1)),
      ..RequestResponseInteraction::default()
    };

    if let Some(index) = headers.get("method") {
      if let Some(method) = values.get(*index) {
        interaction.request.method = method.clone();
      }
    }

    if let Some(index) = headers.get("path") {
      if let Some(path) = values.get(*index) {
        interaction.request.path = path.clone();
      }
    }

    if let Some(index) = headers.get("query") {
      if let Some(query) = values.get(*index) {
        interaction.request.query = parse_query_string(query);
      }
    }

    if let Some(index) = headers.get("headers") {
      if let Some(headers) = values.get(*index) {
        if !headers.is_empty() {
          let headers = headers.split(",")
            .map(|header| {
              let key_value = header.strip_prefix("'").unwrap_or(header)
                .strip_suffix("'").unwrap_or(header)
                .splitn(2, ":")
                .map(|v| v.trim())
                .collect::<Vec<_>>();
              (key_value[0].to_string(), parse_header(key_value[0], key_value[1]))
            }).collect();
          interaction.request.headers = Some(headers);
        }
      }
    }

    if let Some(index) = headers.get("body") {
      if let Some(body) = values.get(*index) {
        setup_body(body, &mut interaction.request, None);
      }
    }

    if let Some(index) = headers.get("matching rules") {
      if let Some(rules) = values.get(*index) {
        let json: Value = if rules.starts_with("JSON:") {
          serde_json::from_str(rules.strip_prefix("JSON:").unwrap_or(rules)).unwrap()
        } else {
          let file = File::open(format!("pact-compatibility-suite/fixtures/{}", rules)).unwrap();
          serde_json::from_reader(file).unwrap()
        };
        interaction.request.matching_rules = matchers_from_json(&json!({"matchingRules": json}), &None).unwrap();
      }
    }

    if let Some(index) = headers.get("response") {
      if let Some(response) = values.get(*index) {
        interaction.response.status = response.parse().unwrap();
      }
    }

    if let Some(index) = headers.get("response headers") {
      if let Some(headers) = values.get(*index) {
        if !headers.is_empty() {
          let headers = headers.split(",")
            .map(|header| {
              let key_value = header.strip_prefix("'").unwrap_or(header)
                .strip_suffix("'").unwrap_or(header)
                .splitn(2, ":")
                .map(|v| v.trim())
                .collect::<Vec<_>>();
              (key_value[0].to_string(), parse_header(key_value[0], key_value[1]))
            }).collect();
          interaction.response.headers = Some(headers);
        }
      }
    }

    if let Some(index) = headers.get("response body") {
      if let Some(body) = values.get(*index) {
        if !body.is_empty() {
          setup_body(body, &mut interaction.response, None);
        }
      }
    }

    if let Some(index) = headers.get("response matching rules") {
      if let Some(rules) = values.get(*index) {
        let json: Value = if rules.starts_with("JSON:") {
          serde_json::from_str(rules.strip_prefix("JSON:").unwrap_or(rules)).unwrap()
        } else {
          let file = File::open(format!("pact-compatibility-suite/fixtures/{}", rules)).unwrap();
          serde_json::from_reader(file).unwrap()
        };
        interaction.response.matching_rules = matchers_from_json(&json!({"matchingRules": json}), &None).unwrap();
      }
    }

    interactions.push(interaction);
  }
  interactions
}

pub fn setup_body(body: &String, httppart: &mut dyn HttpPart, content_type: Option<&str>) {
  if body.starts_with("JSON:") {
    httppart.add_header("content-type", vec!["application/json"]);
    *httppart.body_mut() = OptionalBody::Present(Bytes::from(body.strip_prefix("JSON:").unwrap_or(body).trim().to_string()),
      Some(JSON.clone()), None);
  } else if body.starts_with("XML:") {
    httppart.add_header("content-type", vec!["application/xml"]);
    *httppart.body_mut() = OptionalBody::Present(Bytes::from(body.strip_prefix("XML:").unwrap_or(body).trim().to_string()),
    Some(XML.clone()), None);
  } else if body.starts_with("file:") {
    if body.ends_with("-body.xml") {
      let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
      let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
        .expect(format!("could not load fixture '{}'", body).as_str());
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer)
        .expect(format!("could not read fixture '{}'", body).as_str());
      let fixture = parse_bytes(buffer.as_slice())
        .expect(format!("could not parse fixture as XML: '{}'", body).as_str());
      let root = fixture.as_document().root();
      let body_node = root.children().iter().find_map(|n| n.element()).unwrap();
      let content_type = element_text(body_node, "contentType").unwrap_or("text/plain".to_string());
      httppart.add_header("content-type", vec![content_type.as_str()]);
      *httppart.body_mut() = OptionalBody::Present(Bytes::from(element_text(body_node, "contents").unwrap_or_default()),
        ContentType::parse(content_type.as_str()).ok(), None);
    } else {
      let content_type = content_type.map(|ct| ContentType::from(ct))
        .unwrap_or_else(|| determine_content_type(body, httppart));
      httppart.add_header("content-type", vec![content_type.to_string().as_str()]);

      let file_name = body.strip_prefix("file:").unwrap_or(body).trim();
      let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", file_name))
        .expect(format!("could not load fixture '{}'", body).as_str());
      let mut buffer = Vec::new();
      f.read_to_end(&mut buffer)
        .expect(format!("could not read fixture '{}'", body).as_str());
      *httppart.body_mut() = OptionalBody::Present(Bytes::from(buffer),
        Some(content_type), None);
    }
  } else {
    let content_type = content_type.map(|ct| ContentType::from(ct))
      .unwrap_or_else(|| determine_content_type(body, httppart));
    httppart.add_header("content-type", vec![content_type.to_string().as_str()]);
    let body = Bytes::from(body.clone());
    *httppart.body_mut() = OptionalBody::Present(body, Some(content_type), None);
  }
}

pub fn element_text(root: Element, name: &str) -> Option<String> {
  root.children().iter()
    .filter_map(|n| n.element())
    .find_map(|n| {
      if n.name().local_part().to_string() == name {
        let string = n.children().iter()
          .filter_map(|child| child.text().map(|t| t.text().trim()))
          .collect::<String>();
        if let Some(line_endings) = n.attribute_value("eol") {
          if line_endings == "CRLF" && !cfg!(windows) {
            Some(string.replace('\n', "\r\n"))
          } else {
            Some(string)
          }
        } else {
          Some(string)
        }
      } else {
        None
      }
    })
}

pub fn determine_content_type(body: &String, httppart: &mut dyn HttpPart) -> ContentType {
  if body.ends_with(".json") {
    JSON.clone()
  } else if body.ends_with(".xml") {
    XML.clone()
  } else if body.ends_with(".jpg") {
    ContentType::from("image/jpeg")
  } else if body.ends_with(".pdf") {
    ContentType::from("application/pdf")
  } else {
    httppart.content_type().unwrap_or(TEXT.clone())
  }
}

pub fn assert_value_type(value_type: String, element: &Value) -> Result<(), Error> {
  match value_type.as_str() {
    "integer" => {
      if !INT_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting an integer, but got {}", element))
      } else {
        Ok(())
      }
    }
    "decimal number" => {
      if !DEC_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting a decimal number, but got {}", element))
      } else {
        Ok(())
      }
    }
    "hexadecimal number" => {
      if !HEX_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting a hexadecimal number, but got {}", element))
      } else {
        Ok(())
      }
    }
    "random string" => {
      if !element.is_string() {
        Err(anyhow!("Was expecting a string, but got {}", element))
      } else {
        Ok(())
      }
    }
    "string from the regex" => {
      if !element.is_string() {
        Err(anyhow!("Was expecting a string, but got {}", element))
      } else if !STR_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting {} to match \\d{{1,8}}", element))
      } else {
        Ok(())
      }
    }
    "date" => {
      if !DATE_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting a date, but got {}", element))
      } else {
        Ok(())
      }
    }
    "time" => {
      if !TIME_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting a time, but got {}", element))
      } else {
        Ok(())
      }
    }
    "date-time" => {
      if !DATETIME_REGEX.is_match(json_to_string(element).as_str()) {
        Err(anyhow!("Was expecting a date-time, but got {}", element))
      } else {
        Ok(())
      }
    }
    "UUID" | "simple UUID" | "lower-case-hyphenated UUID" | "upper-case-hyphenated UUID" | "URN UUID" => {
      if Uuid::parse_str(json_to_string(element).as_str()).is_err() {
        Err(anyhow!("Was expecting an UUID, but got {}", element))
      } else {
        Ok(())
      }
    }
    "boolean" => {
      let string = json_to_string(element);
      if string == "true" || string == "false" {
        Ok(())
      } else {
        Err(anyhow!("Was expecting a boolean, but got {}", element))
      }
    }
    _ => Err(anyhow!("Invalid type: {}", value_type))
  }
}
