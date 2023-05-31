use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::str::FromStr;

use anyhow::anyhow;
use bytes::Bytes;
use cucumber::gherkin::Table;
use cucumber::Parameter;
use pact_models::bodies::OptionalBody;
use pact_models::content_types::{ContentType, JSON, XML};
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::query_strings::parse_query_string;
use pact_models::sync_interaction::RequestResponseInteraction;

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
        if !body.is_empty() {
          if body.starts_with("JSON:") {
            interaction.request.add_header("content-type", vec!["application/json"]);
            interaction.request.body = OptionalBody::Present(Bytes::from(body.strip_prefix("JSON:").unwrap_or(body).to_string()),
                                                             Some(JSON.clone()), None);
          } else if body.starts_with("XML:") {
            interaction.request.add_header("content-type", vec!["application/xml"]);
            interaction.request.body = OptionalBody::Present(Bytes::from(body.strip_prefix("XML:").unwrap_or(body).to_string()),
                                                             Some(XML.clone()), None);
          } else {
            let ct = if body.ends_with(".json") {
              "application/json"
            } else if body.ends_with(".xml") {
              "application/xml"
            } else {
              "text/plain"
            };
            interaction.request.headers_mut().insert("content-type".to_string(), vec![ct.to_string()]);

            let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", body))
              .expect(format!("could not load fixture '{}'", body).as_str());
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)
              .expect(format!("could not read fixture '{}'", body).as_str());
            interaction.request.body = OptionalBody::Present(Bytes::from(buffer),
                                                             ContentType::parse(ct).ok(), None);
          }
        }
      }
    }

    if let Some(index) = headers.get("response") {
      if let Some(response) = values.get(*index) {
        interaction.response.status = response.parse().unwrap();
      }
    }

    if let Some(index) = headers.get("response body") {
      if let Some(response) = values.get(*index) {
        if !response.is_empty() {
          let ct = headers.get("response content")
            .map(|i| values.get(*i))
            .flatten()
            .cloned()
            .unwrap_or("text/plain".to_string());
          interaction.response.headers_mut().insert("content-type".to_string(), vec![ct.clone()]);

          let mut f = File::open(format!("pact-compatibility-suite/fixtures/{}", response))
            .expect(format!("could not load fixture '{}'", response).as_str());
          let mut buffer = Vec::new();
          f.read_to_end(&mut buffer)
            .expect(format!("could not read fixture '{}'", response).as_str());
          interaction.response.body = OptionalBody::Present(Bytes::from(buffer),
                                                            ContentType::parse(ct.as_str()).ok(), None);
        }
      }
    }

    interactions.push(interaction);
  }
  interactions
}
