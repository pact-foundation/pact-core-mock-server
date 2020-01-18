use pact_matching::models::Pact;
use pact_matching::s;
use crate::MismatchResult;
use serde_json::{json};
use itertools::Itertools;
use std::collections::HashMap;
use super::provider_client::join_paths;
use regex::{Regex, Captures};
use futures::stream::*;
use pact_matching::models::http_utils::HttpAuth;
use pact_matching::Mismatch;
use std::fmt::{Display, Formatter};
use maplit::*;

fn is_true(object: &serde_json::Map<String, serde_json::Value>, field: &String) -> bool {
    match object.get(field) {
        Some(json) => match json {
            &serde_json::Value::Bool(b) => b,
            _ => false
        },
        None => false
    }
}

fn as_string(json: &serde_json::Value) -> String {
    match json {
        &serde_json::Value::String(ref s) => s.clone(),
        _ => format!("{}", json)
    }
}

fn content_type(response: &reqwest::Response) -> String {
    match response.headers().get("content-type") {
        Some(value) => value.to_str().unwrap_or("text/plain").into(),
        None => s!("text/plain")
    }
}

fn json_content_type(response: &reqwest::Response) -> bool {
    match content_type(response).parse::<mime::Mime>() {
        Ok(mime) => {
            match (mime.type_().as_str(), mime.subtype().as_str(), mime.suffix()) {
                ("application", "json", None) => true,
                ("application", "hal", Some(mime::JSON)) => true,
                _ => false
            }
        }
        Err(_) => false
    }
}

fn find_entry(map: &serde_json::Map<String, serde_json::Value>, key: &String) -> Option<(String, serde_json::Value)> {
    match map.keys().find(|k| k.to_lowercase() == key.to_lowercase() ) {
        Some(k) => map.get(k).map(|v| (key.clone(), v.clone()) ),
        None => None
    }
}

#[derive(Debug, Clone)]
pub enum PactBrokerError {
    LinkError(String),
    ContentError(String),
    IoError(String),
    NotFound(String),
    UrlError(String)
}

impl PartialEq<String> for PactBrokerError {
    fn eq(&self, other: &String) -> bool {
        let message = match self {
            &PactBrokerError::LinkError(ref s) => s.clone(),
            &PactBrokerError::ContentError(ref s) => s.clone(),
            &PactBrokerError::IoError(ref s) => s.clone(),
            &PactBrokerError::NotFound(ref s) => s.clone(),
            &PactBrokerError::UrlError(ref s) => s.clone()
        };
        message == *other
    }
}

impl <'a> PartialEq<&'a str> for PactBrokerError {
    fn eq(&self, other: &&str) -> bool {
        let message = match self {
            &PactBrokerError::LinkError(ref s) => s.clone(),
            &PactBrokerError::ContentError(ref s) => s.clone(),
            &PactBrokerError::IoError(ref s) => s.clone(),
            &PactBrokerError::NotFound(ref s) => s.clone(),
            &PactBrokerError::UrlError(ref s) => s.clone()
        };
        message.as_str() == *other
    }
}

impl Display for PactBrokerError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      &PactBrokerError::LinkError(ref s) => write!(f, "LinkError({})", s),
      &PactBrokerError::ContentError(ref s) => write!(f, "ContentError({})", s),
      &PactBrokerError::IoError(ref s) => write!(f, "IoError({})", s),
      &PactBrokerError::NotFound(ref s) => write!(f, "NotFound({})", s),
      &PactBrokerError::UrlError(ref s) => write!(f, "UrlError({})", s)
    }
  }
}

#[derive(Debug, Clone)]
pub struct Link {
  pub name: String,
  pub href: Option<String>,
  pub templated: bool
}

impl Link {

    pub fn from_json(link: &String, link_data: &serde_json::Map<String, serde_json::Value>) -> Link {
        Link {
            name: link.clone(),
            href: find_entry(link_data, &s!("href")).map(|(_, href)| as_string(&href)),
            templated: is_true(link_data, &s!("templated"))
        }
    }
}

impl Default for Link {
  fn default() -> Self {
    Link {
      name: "link".to_string(),
      href: None,
      templated: false
    }
  }
}

#[derive(Clone)]
pub struct HALClient {
    client: reqwest::Client,
    url: String,
    path_info: Option<serde_json::Value>,
    auth: Option<HttpAuth>
}

impl HALClient {

    fn default() -> HALClient {
      HALClient {
        client: reqwest::ClientBuilder::new()
          .build()
          .unwrap(),
        url: s!(""),
        path_info: None,
        auth: None
      }
    }

    fn with_url(url: String, auth: Option<HttpAuth>) -> HALClient {
        HALClient { url, auth, ..HALClient::default() }
    }

    fn update_path_info(self, path_info: serde_json::Value) -> HALClient {
        HALClient {
            client: self.client.clone(),
            url: self.url,
            path_info: Some(path_info),
            auth: self.auth.clone()
        }
    }

    async fn navigate(
        self,
        link: &'static str,
        template_values: HashMap<String, String>
    ) -> Result<HALClient, PactBrokerError> {
        let path_info = self.clone().fetch("/".into()).await?;
        let client = self.update_path_info(path_info);

        let path_info = client.clone().fetch_link(link, template_values).await?;
        let client = client.update_path_info(path_info);

        Ok(client)
    }

    fn find_link(&self, link: &'static str) -> Result<Link, PactBrokerError> {
        match self.path_info {
            None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
                self.url, link))),
            Some(ref json) => match json.get("_links") {
                Some(json) => match json.get(link) {
                    Some(link_data) => link_data.as_object()
                        .map(|link_data| Link::from_json(&s!(link), &link_data))
                        .ok_or(PactBrokerError::LinkError(format!("Link is malformed, expected an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }

    async fn fetch_link(
        self,
        link: &'static str,
        template_values: HashMap<String, String>
    ) -> Result<serde_json::Value, PactBrokerError> {
        let link_data = self.find_link(link)?;

        self.fetch_url(link_data, template_values).await
    }

    async fn fetch_url(
        self,
        link: Link,
        template_values: HashMap<String, String>
    ) -> Result<serde_json::Value, PactBrokerError> {
        let link_url = if link.templated {
            log::debug!("Link URL is templated");
            self.parse_link_url(&link, &template_values)
        } else {
            link.href.clone()
                .ok_or_else(|| PactBrokerError::LinkError(
                    format!("Link is malformed, there is no href. URL: '{}', LINK: '{}'",
                        self.url, link.name
                    )
                ))
        }?;

        let base_url = self.url.parse::<reqwest::Url>()
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

        let joined_url = base_url.join(&link_url)
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

        self.fetch(joined_url.path().into()).await
    }

    async fn fetch(self, path: String) -> Result<serde_json::Value, PactBrokerError> {
        log::debug!("Fetching path '{}' from pact broker", path);

        let url = join_paths(&self.url, path.clone()).parse::<reqwest::Url>()
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

        let request_builder = match self.auth {
            Some(ref auth) => match auth {
                HttpAuth::User(username, password) => self.client.get(url).basic_auth(username, password.clone()),
                HttpAuth::Token(token) => self.client.get(url).bearer_auth(token)
            },
            None => self.client.get(url)
        }.header("accept", "application/hal+json, application/json");

        let response = request_builder
            .send()
            .await
            .map_err(|err| {
                PactBrokerError::IoError(format!("Failed to access pact broker path '{}' - {}. URL: '{}'",
                    &path,
                    err,
                    &self.url,
                ))
            })?;

        self.parse_broker_response(path, response)
            .await
    }

    async fn parse_broker_response(
        self,
        path: String,
        response: reqwest::Response,
    ) -> Result<serde_json::Value, PactBrokerError> {
        let is_json_content_type = json_content_type(&response);
        let content_type = content_type(&response);

        if response.status().is_success() {
            let body = response.bytes()
                .await
                .map_err(|_| PactBrokerError::IoError(
                    format!("Failed to download response body for path '{}'. URL: '{}'", &path, &self.url)
                ))?;

            if is_json_content_type {
                serde_json::from_slice(&body)
                    .map_err(|err| PactBrokerError::ContentError(
                        format!("Did not get a valid HAL response body from pact broker path '{}' - {}. URL: '{}'",
                            path, err, &self.url)
                    ))
            } else {
                Err(PactBrokerError::ContentError(
                    format!("Did not get a HAL response from pact broker path '{}', content type is '{}'. URL: '{}'",
                        path, content_type, &self.url
                    )
                ))
            }
        } else if response.status() == reqwest::StatusCode::NOT_FOUND {
            Err(PactBrokerError::NotFound(
                format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                    response.status(), self.url
                )
            ))
        } else {
            Err(PactBrokerError::IoError(
                format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                    response.status(), self.url
                )
            ))
        }
    }

    fn parse_link_url(&self, link: &Link, values: &HashMap<String, String>) -> Result<String, PactBrokerError> {
        match link.href {
            Some(ref href) => {
                log::debug!("templated URL = {}", href);
                let re = Regex::new(r"\{(\w+)\}").unwrap();
                let final_url = re.replace_all(href, |caps: &Captures| {
                    let lookup = caps.get(1).unwrap().as_str();
                    log::debug!("Looking up value for key '{}'", lookup);
                    match values.get(lookup) {
                        Some(val) => val.clone(),
                        None => {
                            log::warn!("No value was found for key '{}', mapped values are {:?}",
                                lookup, values);
                            format!("{{{}}}", lookup)
                        }
                    }
                });
                log::debug!("final URL = {}", final_url);
                Ok(final_url.to_string())
            },
            None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{}'",
                self.url, link.name)))
        }
    }

    fn iter_links(&self, link: String) -> Result<Vec<Link>, PactBrokerError> {
        match self.path_info {
            None => Err(PactBrokerError::LinkError(format!("No previous resource has been fetched from the pact broker. URL: '{}', LINK: '{}'",
                self.url, link))),
            Some(ref json) => match json.get("_links") {
                Some(json) => match json.get(&link) {
                    Some(link_data) => link_data.as_array()
                        .map(|link_data| link_data.iter().map(|link_json| match link_json {
                            &serde_json::Value::Object(ref data) => Link::from_json(&link, data),
                            &serde_json::Value::String(ref s) => Link { name: link.clone(), href: Some(s.clone()), templated: false },
                            _ => Link { name: link.clone(), href: Some(link_json.to_string()), templated: false }
                        }).collect())
                        .ok_or(PactBrokerError::LinkError(format!("Link is malformed, expected an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }

    async fn post_json(self, url: String, body: String) -> Result<(), PactBrokerError> {
        log::debug!("Posting JSON to {}: {}", url, body);

        let url = url.parse::<reqwest::Url>()
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))?;

        let request_builder = match self.auth {
            Some(ref auth) => match auth {
                HttpAuth::User(username, password) => self.client
                    .post(url.clone())
                    .basic_auth(username, password.clone()),
                HttpAuth::Token(token) => self.client
                    .post(url.clone())
                    .bearer_auth(token)
            },
            None => self.client.post(url.clone())
        }
            .header("Content-Type", "application/json")
            .body(body);

        request_builder.send()
            .await
            .map_err(|err| PactBrokerError::IoError(
                format!("Failed to post JSON to the pact broker URL '{}' - {}", url, err)
            ))?
            .error_for_status()
            .map_err(|err| PactBrokerError::ContentError(
                format!("Post request to pact broker URL '{}' failed - {}",  url, err)
            ))
            .map(|_| ())
    }
}

fn links_from_json(json: &serde_json::Value) -> Vec<Link> {
   match json.get("_links") {
    Some(json) => match json {
      &serde_json::Value::Object(ref v) => {
        v.iter().map(|(link, json)| match json {
          &serde_json::Value::Object(ref attr) => Link::from_json(link, attr),
          _ => Link { name: link.clone(), .. Link::default() }
        }).collect()
      },
      _ => vec![]
    },
    None => vec![]
  }
}

pub async fn fetch_pacts_from_broker(
    broker_url: String,
    provider_name: String,
    auth: Option<HttpAuth>
) -> Result<Vec<Result<(Pact, Vec<Link>), PactBrokerError>>, PactBrokerError> {
    let mut hal_client = HALClient::with_url(broker_url.clone(), auth);
    let template_values = hashmap!{ s!("provider") => provider_name.clone() };

    hal_client = hal_client.navigate("pb:latest-provider-pacts", template_values.clone())
        .await
        .map_err(move |err| {
            match err {
                PactBrokerError::NotFound(_) =>
                    PactBrokerError::NotFound(
                        format!("No pacts for provider '{}' where found in the pact broker. URL: '{}'",
                            provider_name, broker_url)),
                _ => err
            }
        })?;

    let pact_links = hal_client.iter_links(s!("pacts"))?;

    let results: Vec<_> = futures::stream::iter(pact_links)
        .map(|pact_link| {
            match pact_link.clone().href {
                Some(_) => Ok((hal_client.clone(), pact_link)),
                None => Err(
                    PactBrokerError::LinkError(
                        format!(
                            "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{:?}'",
                            &hal_client.url,
                            pact_link
                        )
                    )
                )
            }
        })
        .and_then(|(hal_client, pact_link)| async {
            let pact_json = hal_client.fetch_url(
                pact_link.clone(),
                template_values.clone()
            ).await?;
            Ok((pact_link, pact_json))
        })
        .map_ok(|(pact_link, pact_json)| {
            let href = pact_link.href.unwrap();
            let pact = Pact::from_json(&href, &pact_json);
            let links = links_from_json(&pact_json);

            (pact, links)
        })
        .into_stream()
        .collect()
        .await;

    Ok(results)
}

pub enum TestResult {
  Ok,
  Failed(Vec<(String, MismatchResult)>)
}

impl TestResult {
  pub fn to_bool(&self) -> bool {
    match self {
      TestResult::Ok => true,
      _ => false
    }
  }
}

/// Publishes the result to the "pb:publish-verification-results" link in the links associated with the pact
pub async fn publish_verification_results(
    links: Vec<Link>,
    broker_url: String,
    auth: Option<HttpAuth>,
    result: TestResult, version:
    String, build_url: Option<String>
) -> Result<(), PactBrokerError> {
    let publish_link = links
        .iter()
        .cloned()
        .find(|item| item.name.to_ascii_lowercase() == "pb:publish-verification-results")
        .ok_or(PactBrokerError::LinkError(
            "Response from the pact broker has no 'pb:publish-verification-results' link".into()
        ))?;

    let json = build_payload(result, version, build_url);
    let hal_client = HALClient::with_url(broker_url.clone(), auth.clone());
    hal_client.post_json(publish_link.href.clone().unwrap(), json.to_string()).await
}

fn build_payload(result: TestResult, version: String, build_url: Option<String>) -> serde_json::Value {
  let mut json = json!({
    "success": result.to_bool(),
    "providerApplicationVersion": version
  });
  let json_obj = json.as_object_mut().unwrap();

  if build_url.is_some() {
    json_obj.insert("buildUrl".into(), json!(build_url.unwrap()));
  }

  match result {
    TestResult::Failed(mismatches) => {
      let values = mismatches.iter()
        .group_by(|mismatch| mismatch.1.interaction_id().unwrap_or(String::new()))
        .into_iter()
        .map(|(key, mismatches)| {
          let acc: (Vec<serde_json::Value>, Vec<serde_json::Value>) = (vec![], vec![]);
          let values = mismatches.into_iter().fold(acc, |mut acc, mismatch| {
            match mismatch.1 {
              MismatchResult::Mismatches { ref mismatches, .. } => {
                for mismatch in mismatches {
                  match mismatch {
                    &Mismatch::MethodMismatch { ref expected, ref actual } => acc.0.push(json!({
                      "attribute": "method",
                      "description": format!("Expected method of {} but received {}", expected, actual)
                    })),
                    &Mismatch::PathMismatch { ref mismatch, .. } => acc.0.push(json!({
                      "attribute": "path",
                      "description": mismatch
                    })),
                    &Mismatch::StatusMismatch { ref expected, ref actual } => acc.0.push(json!({
                      "attribute": "status",
                      "description": format!("Expected status of {} but received {}", expected, actual)
                    })),
                    &Mismatch::QueryMismatch { ref parameter, ref mismatch, .. } => acc.0.push(json!({
                      "attribute": "query",
                      "identifier": parameter,
                      "description": mismatch
                    })),
                    &Mismatch::HeaderMismatch { ref key, ref mismatch, .. } => acc.0.push(json!({
                      "attribute": "header",
                      "identifier": key,
                      "description": mismatch
                    })),
                    &Mismatch::BodyTypeMismatch { ref expected, ref actual} => acc.0.push(json!({
                      "attribute": "body",
                      "identifier": "$",
                      "description": format!("Expected body type of '{}' but received '{}'", expected, actual)
                    })),
                    &Mismatch::BodyMismatch { ref path, ref mismatch, .. } => acc.0.push(json!({
                      "attribute": "body",
                      "identifier": path,
                      "description": mismatch
                    }))
                  }
                }
              },
              MismatchResult::Error(ref err, _) => acc.1.push(json!({ "message": err }))
            };
            acc
          });

          let mut json = json!({
            "interactionId": key,
            "success": false,
            "mismatches": values.0
          });

          if !values.1.is_empty() {
            json.as_object_mut().unwrap().insert("exceptions".into(), json!(values.1));
          }

          json
        }).collect::<Vec<serde_json::Value>>();

      json_obj.insert("testResults".into(), serde_json::Value::Array(values));
    },
    _ => ()
  }
  json
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use expectest::expect;
    use super::*;
    use super::{content_type, json_content_type};
    use pact_consumer::prelude::*;
    use pact_consumer::*;
    use env_logger::*;
    use pact_matching::models::{Pact, Consumer, Provider, Interaction, PactSpecification};
    use pact_matching::Mismatch::MethodMismatch;

    #[tokio::test]
    async fn fetch_returns_an_error_if_there_is_no_pact_broker() {
        let client = HALClient::with_url(s!("http://idont.exist:6666"), None);
        expect!(client.fetch(s!("/")).await).to(be_err());
    }

    #[tokio::test]
    async fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to a non-existant path", |i| {
                i.given("the pact broker has a valid pact");
                i.request.path("/hello");
                i.response.status(404);
            })
            .start_mock_server();

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.fetch(s!("/hello")).await;
        expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-json resource", |i| {
                i.request.path("/nonjson");
                i.response
                    .header("Content-Type", "text/html")
                    .body("<html></html>");
            })
            .start_mock_server();

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.fetch(s!("/nonjson")).await;
        expect!(result).to(be_err().value(format!("Did not get a HAL response from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn content_type_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json; charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(content_type(&response)).to(be_equal_to(s!("application/hal+json; charset=utf-8")));
    }

    #[test]
    fn json_content_type_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/json")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test]
    fn json_content_type_utf8_test() {
        let response = reqwest::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json;charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[tokio::test]
    async fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal resource", |i| {
                i.request.path("/nonhal");
                i.response.header("Content-Type", "application/hal+json");
            })
            .interaction("a request to a non-hal resource 2", |i| {
                i.request.path("/nonhal2");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("<html>This is not JSON</html>");
            })
            .start_mock_server();

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/nonhal")).await;
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal\' - EOF while parsing a value at line 1 column 0. URL: '{}'",
            pact_broker.url())));

        let result = client.clone().fetch(s!("/nonhal2")).await;
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal2\' - expected value at line 1 column 1. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn parse_link_url_returns_error_if_there_is_no_href() {
        let client = HALClient::default();
        let link = Link { name: s!("link"), href: None, templated: false };
        expect!(client.parse_link_url(&link, &hashmap!{})).to(be_err().value(
            "Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '', LINK: 'link'"));
    }

    #[test]
    fn parse_link_url_replaces_all_tokens_in_href() {
        let client = HALClient::default();
        let values = hashmap!{ s!("valA") => s!("A"), s!("valB") => s!("B") };

        let link = Link { name: s!("link"), href: Some(s!("http://localhost")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://localhost"));

        let link = Link { name: s!("link"), href: Some(s!("http://{valA}/{valB}")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/B"));

        let link = Link { name: s!("link"), href: Some(s!("http://{valA}/{valC}")), templated: false };
        expect!(client.parse_link_url(&link, &values)).to(be_ok().value("http://A/{valC}"));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_a_previous_resource_has_not_been_fetched() {
        let client = HALClient::with_url(s!("http://localhost"), None);
        let result = client.fetch_link("anything_will_do", hashmap!{}).await;
        expect!(result).to(be_err().value(s!("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'")));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("hal2", hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("any", hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("any", hashmap!{}).await;
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_link_returns_the_resource_for_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("next", hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[tokio::test]
    async fn fetch_link_returns_handles_absolute_resource_links() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with absolute paths", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"http://localhost/abc\"},\"prev\":{\"href\":\"http://localhost/def\"}}}");
            })
            .interaction("a request to next", |i| {
                i.request.path("/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("next", hashmap!{}).await;
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[tokio::test]
    async fn fetch_link_returns_the_resource_for_the_templated_link() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a templated hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"document\":{\"href\":\"/doc/{id}\",\"templated\":true}}}");

            })
            .interaction("a request for a document", |i| {
                i.request.path("/doc/abc");
                i.response
                    .header("Content-Type", "application/json")
                    .json_body(json_pattern!("Yay! You found your way here"));
            })
            .start_mock_server();

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = client.clone().fetch(s!("/")).await;
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = client.clone().fetch_link("document", hashmap!{ s!("id") => s!("abc") }).await;
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[tokio::test]
    async fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
            })
            .interaction("a request for a providers pacts", |i| {
                i.given("There are no pacts in the pact broker");
                i.request
                    .path("/pacts/provider/sad_provider/latest")
                    .header("Accept", "application/hal+json, application/json");
                i.response.status(404);
            })
            .start_mock_server();

        let result = fetch_pacts_from_broker(pact_broker.url().to_string(), s!("sad_provider"), None).await;
        expect!(result).to(be_err().value(format!("No pacts for provider 'sad_provider' where found in the pact broker. URL: '{}'",
            pact_broker.url())));
    }

    #[tokio::test]
    async fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
        init().unwrap_or(());
        let pact = Pact { consumer: Consumer { name: s!("Consumer") },
            provider: Provider { name: s!("happy_provider") },
            .. Pact::default() }
            .to_json(PactSpecification::V3).to_string();
        let pact2 = Pact { consumer: Consumer { name: s!("Consumer2") },
            provider: Provider { name: s!("happy_provider") },
            interactions: vec![ Interaction { description: s!("a request friends"), .. Interaction::default() } ],
            .. Pact::default() }
            .to_json(PactSpecification::V3).to_string();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to the pact broker root", |i| {
                i.request
                    .path("/")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links": {
                            "pb:latest-provider-pacts": {
                                "href": "http://localhost/pacts/provider/{provider}/latest",
                                "templated": true,
                            }
                        }
                    }));
            })
            .interaction("a request for a providers pacts", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/latest")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .json_body(json_pattern!({
                        "_links":{
                            "pacts":[
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0"},
                                {"href":"http://localhost/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0"}
                            ]
                        }
                    }));
            })
            .interaction("a request for the first provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer/version/1.0.0")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact.clone());
            })
            .interaction("a request for the second provider pact", |i| {
                i.given("There are two pacts in the pact broker");
                i.request
                    .path("/pacts/provider/happy_provider/consumer/Consumer2/version/1.0.0")
                    .header("Accept", "application/hal+json, application/json");
                i.response
                    .header("Content-Type", "application/json")
                    .body(pact2.clone());
            })
            .start_mock_server();

        let result = fetch_pacts_from_broker(pact_broker.url().to_string(), s!("happy_provider"), None).await;
        expect!(result.clone()).to(be_ok());
        let pacts = result.unwrap();
        expect!(pacts.len()).to(be_equal_to(2));
        for pact in pacts {
            expect!(pact).to(be_ok());
        }
    }

  #[test]
  fn test_build_payload_with_success() {
    let result = TestResult::Ok;
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1", "success": true
    })));
  }

  #[test]
  fn test_build_payload_adds_the_build_url_if_provided() {
    let result = TestResult::Ok;
    let payload = super::build_payload(result, "1".to_string(), Some("http://build-url".to_string()));
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": true,
      "buildUrl": "http://build-url"
    })));
  }

  #[test]
  fn test_build_payload_with_failure() {
    let result = TestResult::Failed(vec![]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1", "success": false, "testResults": []
    })));
  }

  #[test]
  fn test_build_payload_with_failure_with_mismatches() {
    let result = TestResult::Failed(vec![
      ("Description".to_string(), MismatchResult::Mismatches {
        mismatches: vec![
          MethodMismatch { expected: "PUT".to_string(), actual: "POST".to_string() }
        ],
        expected: Default::default(),
        actual: Default::default(),
        interaction_id: Some("1234abc".to_string())
      })
    ]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "interactionId": "1234abc",
          "mismatches": [
            {
              "attribute": "method", "description": "Expected method of PUT but received POST"
            }
          ],
          "success": false
        }
      ]
    })));
  }

  #[test]
  fn test_build_payload_with_failure_with_exception() {
    let result = TestResult::Failed(vec![
      ("Description".to_string(), MismatchResult::Error("Bang".to_string(), Some("1234abc".to_string())))
    ]);
    let payload = super::build_payload(result, "1".to_string(), None);
    expect!(payload).to(be_equal_to(json!({
      "providerApplicationVersion": "1",
      "success": false,
      "testResults": [
        {
          "exceptions": [
            {
              "message": "Bang"
            }
          ],
          "interactionId": "1234abc",
          "mismatches": [],
          "success": false
        }
      ]
    })));
  }
}
