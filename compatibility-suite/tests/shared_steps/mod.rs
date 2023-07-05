use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use anyhow::anyhow;
use bytes::Bytes;
use cucumber::gherkin::Table;
use cucumber::Parameter;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, JSON, XML, TEXT};
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::matchingrules::matchers_from_json;
use pact_models::query_strings::parse_query_string;
use pact_models::sync_interaction::RequestResponseInteraction;
use pact_models::xml_utils::parse_bytes;
use serde_json::{json, Value};
use sxd_document::dom::Element;

pub mod consumer;
pub mod provider;

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
        setup_body(body, &mut interaction.request);
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
          setup_body(body, &mut interaction.response);
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

pub(crate) fn setup_body(body: &String, httppart: &mut dyn HttpPart) {
  if !body.is_empty() {
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
        let content_type = determine_content_type(body, httppart);
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
      let content_type = determine_content_type(body, httppart);
      httppart.add_header("content-type", vec![content_type.to_string().as_str()]);
      let body = Bytes::from(body.clone());
      *httppart.body_mut() = OptionalBody::Present(body, Some(content_type), None);
    }
  }
}

fn element_text(root: Element, name: &str) -> Option<String> {
  root.children().iter()
    .filter_map(|n| n.element())
    .find_map(|n| if n.name().local_part().to_string() == name {
      Some(n.children().iter()
        .filter_map(|child| child.text().map(|t| t.text().trim()))
        .collect::<String>())
    } else {
      None
    })
}

fn determine_content_type(body: &String, httppart: &mut dyn HttpPart) -> ContentType {
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
