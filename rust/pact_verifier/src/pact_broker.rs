use pact_matching::models::Pact;
use ::{serde_json, MismatchResult};
use itertools::Itertools;
use std::collections::HashMap;
use super::provider_client::join_paths;
use regex::{Regex, Captures};
use futures::future;
use futures::future::Future;
use futures::stream::Stream;
use pact_matching::models::http_utils::HttpAuth;
use pact_matching::Mismatch;
use std::fmt::{Display, Formatter};

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

fn content_type(response: &reqwest::async::Response) -> String {
    match response.headers().get("content-type") {
        Some(value) => value.to_str().unwrap_or("text/plain").into(),
        None => s!("text/plain")
    }
}

fn json_content_type(response: &reqwest::async::Response) -> bool {
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
  client: reqwest::async::Client,
  url: String,
  path_info: Option<serde_json::Value>,
  auth: Option<HttpAuth>
}

impl HALClient {

    fn default() -> HALClient {
      HALClient {
        client: reqwest::async::ClientBuilder::new()
          .use_default_tls()
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

    fn navigate(self, link: &'static str, template_values: HashMap<String, String>) -> impl Future<Item = HALClient, Error = PactBrokerError> {
        future::ok(self)
            .and_then(|client| {
                client.clone().fetch("/".into())
                    .map(|path_info| client.update_path_info(path_info))
            })
            .and_then(move |client| {
                client.clone().fetch_link(link, template_values)
                    .map(|path_info| client.update_path_info(path_info))
            })
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

    fn fetch_link(self, link: &'static str, template_values: HashMap<String, String>) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        future::done(self.find_link(link))
            .and_then(|link_data| self.fetch_url(&link_data, template_values))
    }

    fn fetch_url(self, link: &Link, template_values: HashMap<String, String>) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        future::done(if link.templated {
            debug!("Link URL is templated");
            self.parse_link_url(&link, &template_values)
        } else {
            link.href.clone().ok_or(
                PactBrokerError::LinkError(format!("Link is malformed, there is no href. URL: '{}', LINK: '{}'",
                                                   self.url, link.name)))
        })
            .and_then(move |link_url| {
                self.url.parse::<reqwest::Url>()
                    .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))
                    .and_then(|base_url| base_url.join(&link_url)
                        .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))
                    )
                    .map(|uri| (self, uri))
            })
            .and_then(|(hal_client, uri)| {
                hal_client.fetch(uri.path().into())
            })
    }

    fn fetch(self, path: String) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        debug!("Fetching path '{}' from pact broker", path);

        future::done(join_paths(&self.url, path.clone()).parse::<reqwest::Url>())
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))
            .and_then(move |url| {
              let client_url_cloned = self.url.clone();
              let path_cloned = path.clone();
              let http_client = match self.auth {
                Some(ref auth) => match auth {
                   HttpAuth::User(username, password) => self.client.get(url).basic_auth(username, password.clone()),
                   HttpAuth::Token(token) => self.client.get(url).bearer_auth(token)
                },
                None => self.client.get(url)
              }.header("accept", "application/hal+json, application/json");

              http_client.send()
                  .map_err(move |err| {
                      PactBrokerError::IoError(format!("Failed to access pact broker path '{}' - {}. URL: '{}'",
                          path_cloned,
                          err,
                          client_url_cloned
                      ))
                  })
                  .map(|response| (self, path, response))
            })
            .and_then(|(hal_client, path, response)| hal_client.parse_broker_response(path, response))
    }

    fn parse_broker_response(
        self,
        path: String,
        response: reqwest::async::Response
    ) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        let is_json_content_type = json_content_type(&response);
        let content_type = content_type(&response);

        future::done(Ok(response))
            .and_then(move |response| {
                if response.status().is_success() {
                    Ok((self, path, response))
                } else {
                    if response.status() == reqwest::StatusCode::NOT_FOUND {
                        Err(PactBrokerError::NotFound(format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                            response.status(), self.url)))
                    } else {
                        Err(PactBrokerError::IoError(format!("Request to pact broker path '{}' failed: {}. URL: '{}'", path,
                            response.status(), self.url)))
                    }
                }
            })
            .and_then(|(hal_client, path, response)| {
                let client_url_cloned = hal_client.url.clone();
                let path_cloned = path.clone();

                response.into_body().concat2()
                    .map(|body| (hal_client, path, body))
                    .map_err(move |_| {
                        PactBrokerError::IoError(format!("Failed to download response body for path '{}'. URL: '{}'",
                            path_cloned, client_url_cloned
                        ))
                    })
            })
            .and_then(move |(hal_client, path, body)| {
                if is_json_content_type {
                    serde_json::from_slice(&body)
                        .map_err(|err| {
                            PactBrokerError::ContentError(format!("Did not get a valid HAL response body from pact broker path '{}' - {}. URL: '{}'",
                                path, err, hal_client.url))
                        })
                } else {
                    Err(PactBrokerError::ContentError(format!("Did not get a HAL response from pact broker path '{}', content type is '{}'. URL: '{}'",
                        path, content_type, hal_client.url)))
                }
            })
    }

    fn parse_link_url(&self, link: &Link, values: &HashMap<String, String>) -> Result<String, PactBrokerError> {
        match link.href {
            Some(ref href) => {
                debug!("templated URL = {}", href);
                let re = Regex::new(r"\{(\w+)\}").unwrap();
                let final_url = re.replace_all(href, |caps: &Captures| {
                    let lookup = caps.get(1).unwrap().as_str();
                    debug!("Looking up value for key '{}'", lookup);
                    match values.get(lookup) {
                        Some(val) => val.clone(),
                        None => {
                            warn!("No value was found for key '{}', mapped values are {:?}",
                                lookup, values);
                            format!("{{{}}}", lookup)
                        }
                    }
                });
                debug!("final URL = {}", final_url);
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

  fn post_json(self, url: String, body: String) -> impl Future<Item = (), Error = PactBrokerError> {
    debug!("Posting JSON to {}: {}", url, body);
    future::done(url.parse::<reqwest::Url>())
      .map_err(|err| PactBrokerError::UrlError(format!("{}", err)))
      .and_then( move |url| {
        let http_client = match self.auth {
          Some(ref auth) => match auth {
            HttpAuth::User(username, password) => self.client.post(url.clone()).basic_auth(username, password.clone()),
            HttpAuth::Token(token) => self.client.post(url.clone()).bearer_auth(token)
          },
          None => self.client.post(url.clone())
        }.header("Content-Type", "application/json")
        .body(body);

        http_client.send()
          .map_err(move |err| {
            PactBrokerError::IoError(format!("Failed to post JSON to the pact broker URL '{}' - {}",
              url, err
            ))
          })
          .map(|_| ())
      })
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

pub fn fetch_pacts_from_broker(
    broker_url: String,
    provider_name: String,
    auth: Option<HttpAuth>
) -> impl Future<Item = Vec<Result<(Pact, Vec<Link>), PactBrokerError>>, Error = PactBrokerError> {
    let hal_client = HALClient::with_url(broker_url.clone(), auth);
    let template_values = hashmap!{ s!("provider") => provider_name.clone() };

    hal_client.navigate("pb:latest-provider-pacts", template_values.clone())
        .map_err(move |err| {
            match err {
                PactBrokerError::NotFound(_) =>
                    PactBrokerError::NotFound(
                        format!("No pacts for provider '{}' where found in the pact broker. URL: '{}'",
                            provider_name, broker_url)),
                _ => err
            }
        })
        .and_then(|hal_client| {
            hal_client.iter_links(s!("pacts"))
                .map(|pact_links| (hal_client, pact_links))
        })
        .and_then(move |(hal_client, pact_links)| {
            let client_url_cloned = hal_client.url.clone();

            futures::stream::iter_ok::<_, PactBrokerError>(pact_links)
                .and_then(move |pact_link| {
                    match pact_link.clone().href {
                        Some(_) => Ok(pact_link),
                        None => Err(
                            PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a link with no HREF. URL: '{}', LINK: '{:?}'",
                                client_url_cloned, pact_link))
                        )
                    }
                })
                .and_then(move |pact_link| {
                  hal_client.clone().fetch_url(&pact_link, template_values.clone())
                    .map(move |pact_json| {
                      let pact = Pact::from_json(&pact_link.href.clone().unwrap(), &pact_json);
                      let links = links_from_json(&pact_json);
                      (pact, links)
                    })
                })
                .then(|result| {
                    Ok(result)
                })
                .collect()
        })
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
pub fn publish_verification_results(links: Vec<Link>, broker_url: String, auth: Option<HttpAuth>, result: TestResult, version: String, build_url: Option<String>)
  -> impl Future<Item = (), Error = PactBrokerError> {
  let publish_link = links.iter().cloned().find(|item| item.name.to_ascii_lowercase() == "pb:publish-verification-results")
    .ok_or(PactBrokerError::LinkError("Response from the pact broker has no 'pb:publish-verification-results' link".into()));
  future::done(publish_link)
  .and_then(move |publish_link| {
    let json = build_payload(result, version, build_url);
    let hal_client = HALClient::with_url(broker_url.clone(), auth.clone());
    hal_client.post_json(publish_link.href.clone().unwrap(), json.to_string())
  })
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
    use super::*;
    use super::{content_type, json_content_type};
    use pact_consumer::prelude::*;
    use env_logger::*;
    use pact_matching::models::{Pact, Consumer, Provider, Interaction, PactSpecification};
    use tokio::runtime::current_thread::Runtime;
  use pact_matching::Mismatch::MethodMismatch;

  #[test]
    fn fetch_returns_an_error_if_there_is_no_pact_broker() {
        let mut runtime = Runtime::new().unwrap();
        let client = HALClient::with_url(s!("http://idont.exist:6666"), None);
        expect!(runtime.block_on(client.fetch(s!("/")))).to(be_err());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
        let mut runtime = Runtime::new().unwrap();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to a non-existant path", |i| {
                i.given("the pact broker has a valid pact");
                i.request.path("/hello");
                i.response.status(404);
            })
            .create_mock_server(|future| { runtime.spawn(future); });

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.fetch(s!("/hello")));
        expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
        let mut runtime = Runtime::new().unwrap();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-json resource", |i| {
                i.request.path("/nonjson");
                i.response
                    .header("Content-Type", "text/html")
                    .body("<html></html>");
            })
            .create_mock_server(|future| { runtime.spawn(future); });

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.fetch(s!("/nonjson")));
        expect!(result).to(be_err().value(format!("Did not get a HAL response from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn content_type_test() {
        let response = reqwest::async::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json; charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(content_type(&response)).to(be_equal_to(s!("application/hal+json; charset=utf-8")));
    }

    #[test]
    fn json_content_type_test() {
        let response = reqwest::async::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/json")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test]
    fn json_content_type_utf8_test() {
        let response = reqwest::async::Response::from(
            http::response::Builder::new()
                .header("content-type", "application/hal+json;charset=utf-8")
                .body("null")
                .unwrap()
        );

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/nonhal")));
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal\' - EOF while parsing a value at line 1 column 0. URL: '{}'",
            pact_broker.url())));

        let result = runtime.block_on(client.clone().fetch(s!("/nonhal2")));
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

    #[test]
    fn fetch_link_returns_an_error_if_a_previous_resource_has_not_been_fetched() {
        let mut runtime = Runtime::new().unwrap();
        let client = HALClient::with_url(s!("http://localhost"), None);
        let result = runtime.block_on(client.fetch_link("anything_will_do", hashmap!{}));
        expect!(result).to(be_err().value(s!("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'")));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
            })
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("hal2", hashmap!{}));
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
            })
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("any", hashmap!{}));
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
        let mut runtime = Runtime::new().unwrap();
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("any", hashmap!{}));
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_link() {
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("next", hashmap!{}));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_handles_absolute_resource_links() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("next", hashmap!{}));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_templated_link() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let mut client = HALClient::with_url(pact_broker.url().to_string(), None);
        let result = runtime.block_on(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = runtime.block_on(client.clone().fetch_link("document", hashmap!{ s!("id") => s!("abc") }));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let result = runtime.block_on(fetch_pacts_from_broker(pact_broker.url().to_string(), s!("sad_provider"), None));
        expect!(result).to(be_err().value(format!("No pacts for provider 'sad_provider' where found in the pact broker. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
        init().unwrap_or(());
        let mut runtime = Runtime::new().unwrap();
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
            .create_mock_server(|future| { runtime.spawn(future); });

        let result = runtime.block_on(fetch_pacts_from_broker(pact_broker.url().to_string(), s!("happy_provider"), None));
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
