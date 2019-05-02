use pact_matching::models::Pact;
use serde_json;
use itertools::Itertools;
use std::collections::HashMap;
use hyper::Client;
use std::error::Error;
use super::provider_client::join_paths;
use regex::{Regex, Captures};
use bytes::Bytes;
use hyper::{Request, Response, Body};
use hyper::Uri;
use hyper::http::uri::{Parts as UriParts};
use hyper::StatusCode;
use futures::future;
use futures::future::Future;
use futures::stream::Stream;

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

fn content_type<T>(response: &Response<T>) -> String {
    match response.headers().get("content-type") {
        Some(value) => value.to_str().unwrap_or("text/plain").into(),
        None => s!("text/plain")
    }
}

fn json_content_type<T>(response: &Response<T>) -> bool {
    match response.headers().get("content-type") {
        Some(value) => {
            match value.to_str() {
                Err(e) => false,
                Ok("application/json") => true,
                Ok("application/hal+json") => true,
                _ => false
            }
        },
        None => false
    }
}

fn join_uris(base: Uri, link: Uri) -> Result<Uri, PactBrokerError> {
    let base_parts = base.into_parts();
    let link_parts = link.into_parts();

    let path_and_query = format!("{}{}",
        base_parts.path_and_query
            .map(|path_and_query| path_and_query.path().to_string())
            .unwrap_or("".into()),
        link_parts.path_and_query
            .map(|path_and_query| path_and_query.as_str().to_string())
            .unwrap_or("".into())
        );

    Uri::builder()
        .scheme(base_parts.scheme.unwrap())
        .authority(base_parts.authority.unwrap())
        .path_and_query(Bytes::from(path_and_query))
        .build()
        .map_err(|err| PactBrokerError::UrlError(format!("{}", err.description())))
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

#[derive(Debug, Clone)]
pub struct Link {
    name: String,
    href: Option<String>,
    templated: bool
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

#[derive(Clone)]
pub struct HALClient {
    url: String,
    path_info: Option<serde_json::Value>
}

impl HALClient {

    fn default() -> HALClient {
        HALClient{ url: s!(""), path_info: None }
    }

    fn update_path_info(self, path_info: serde_json::Value) -> HALClient {
        HALClient{ url: self.url, path_info: Some(path_info) }
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
                link_url.parse::<Uri>()
                    .map_err(|err| PactBrokerError::UrlError(format!("{}", err.description())))
                    .map(|uri| (self, uri))
            })
            .and_then(|(hal_client, link_uri)| {
                hal_client.url.parse::<Uri>()
                    .map_err(|err| PactBrokerError::UrlError(format!("{}", err.description())))
                    .and_then(|base_uri| join_uris(base_uri, link_uri))
                    .map(|uri| (hal_client, uri))
            })
            .and_then(|(hal_client, uri)| {
                hal_client.fetch(uri.path().into())
            })
    }

    fn fetch(self, path: String) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        debug!("Fetching path '{}' from pact broker", path);

        future::done(join_paths(&self.url, path.clone()).parse::<Uri>())
            .map_err(|err| PactBrokerError::UrlError(format!("{}", err.description())))
            .and_then(move |url| {
                let client_url_cloned = self.url.clone();
                let path_cloned = path.clone();

                Client::new().request(
                    Request::get(url)
                        .header("accept", "application/hal+json, application/json")
                        .body(Body::empty())
                        .unwrap()
                )
                    .map_err(move |err| {
                        PactBrokerError::IoError(format!("Failed to access pact broker path '{}' - {:?}. URL: '{}'",
                            path_cloned,
                            err.description(),
                            client_url_cloned
                        ))
                    })
                    .map(|response| (self, path, response))
            })
            .and_then(|(hal_client, path, response)| hal_client.parse_broker_response(path, response))
    }

    fn parse_broker_response(self, path: String, response: Response<Body>) -> impl Future<Item = serde_json::Value, Error = PactBrokerError> {
        let is_json_content_type = json_content_type(&response);
        let content_type = content_type(&response);

        future::done(Ok(response))
            .and_then(move |response| {
                if response.status().is_success() {
                    Ok((self, path, response))
                } else {
                    if response.status() == StatusCode::NOT_FOUND {
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
                            PactBrokerError::ContentError(format!("Did not get a valid HAL response body from pact broker path '{}' - {}: {}. URL: '{}'",
                                path, err.description(), err, hal_client.url))
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
                    let lookup = caps.at(1).unwrap();
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
                Ok(final_url)
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
                        .ok_or(PactBrokerError::LinkError(format!("Link is malformed, expcted an object but got {}. URL: '{}', LINK: '{}'",
                            link_data, self.url, link))),
                    None => Err(PactBrokerError::LinkError(format!("Link '{}' was not found in the response, only the following links where found: {:?}. URL: '{}', LINK: '{}'",
                        link, json.as_object().unwrap_or(&json!({}).as_object().unwrap()).keys().join(", "), self.url, link)))
                },
                None => Err(PactBrokerError::LinkError(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: '{}'",
                    self.url, link)))
            }
        }
    }
}

pub fn fetch_pacts_from_broker(broker_url: String, provider_name: String) -> impl Future<Item = Vec<Result<Pact, PactBrokerError>>, Error = PactBrokerError> {
    let hal_client = HALClient{ url: broker_url.clone(), .. HALClient::default() };
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
                        .map(move |pact_json| Pact::from_json(&pact_link.href.clone().unwrap(), &pact_json))
                })
                .then(|result| {
                    Ok(result)
                })
                .collect()
        })
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

    fn wait<T>(future: impl Future<Item = T, Error = PactBrokerError>) -> Result<T, PactBrokerError> {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(future)
    }

    #[test]
    fn fetch_returns_an_error_if_there_is_no_pact_broker() {
        let client = HALClient{ url: s!("http://idont.exist:6666"), .. HALClient::default() };
        expect!(wait(client.fetch(s!("/")))).to(be_err());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_success_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBroker")
            .interaction("a request to a non-existant path", |i| {
                i.given("the pact broker has a valid pact");
                i.request.path("/hello");
                i.response.status(404);
            })
            .start_mock_server();

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.fetch(s!("/hello")));
        expect!(result).to(be_err().value(format!("Request to pact broker path \'/hello\' failed: 404 Not Found. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_hal_response() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-json resource", |i| {
                i.request.path("/nonjson");
                i.response
                    .header("Content-Type", "text/html")
                    .body("<html></html>");
            })
            .start_mock_server();

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.fetch(s!("/nonjson")));
        expect!(result).to(be_err().value(format!("Did not get a HAL response from pact broker path \'/nonjson\', content type is 'text/html'. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn content_type_test() {
        let response = Response::builder()
            .header("content-type", "application/hal+json; charset=utf-8")
            .body(())
            .unwrap();

        expect!(content_type(&response)).to(be_equal_to(s!("application/hal+json; charset=utf-8")));
    }

    #[test]
    fn json_content_type_test() {
        let response = Response::builder()
            .header("content-type", "application/json")
            .body(())
            .unwrap();

        expect!(json_content_type(&response)).to(be_true());
    }

    #[test]
    fn fetch_returns_an_error_if_it_does_not_get_a_valid_hal_response() {
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

        let client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/nonhal")));
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal\'. URL: '{}'",
            pact_broker.url())));
        let result = wait(client.clone().fetch(s!("/nonhal2")));
        expect!(result).to(be_err().value(format!("Did not get a valid HAL response body from pact broker path \'/nonhal2\' - JSON error: expected value at line 1 column 1. URL: '{}'",
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
        let client = HALClient{ url: s!("http://localhost"), .. HALClient::default() };
        let result = wait(client.fetch_link("anything_will_do", hashmap!{}));
        expect!(result).to(be_err().value(s!("No previous resource has been fetched from the pact broker. URL: 'http://localhost', LINK: 'anything_will_do'")));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_was_not_hal() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a non-hal json resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("hal2", hashmap!{}));
        expect!(result).to(be_err().value(format!("Expected a HAL+JSON response from the pact broker, but got a response with no '_links'. URL: '{}', LINK: 'hal2'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_links_are_not_correctly_formed() {
        init().unwrap_or(());
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource with invalid links", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":[{\"next\":{\"href\":\"abc\"}},{\"prev\":{\"href\":\"def\"}}]}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("any", hashmap!{}));
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_an_error_if_the_previous_resource_does_not_have_the_link() {
        let pact_broker = PactBuilder::new("RustPactVerifier", "PactBrokerStub")
            .interaction("a request to a hal resource", |i| {
                i.request.path("/");
                i.response
                    .header("Content-Type", "application/hal+json")
                    .body("{\"_links\":{\"next\":{\"href\":\"/abc\"},\"prev\":{\"href\":\"/def\"}}}");
            })
            .start_mock_server();

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("any", hashmap!{}));
        expect!(result).to(be_err().value(format!("Link 'any' was not found in the response, only the following links where found: \"next, prev\". URL: '{}', LINK: 'any'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_link() {
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

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("next", hashmap!{}));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_handles_absolute_resource_links() {
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

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("next", hashmap!{}));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_link_returns_the_resource_for_the_templated_link() {
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

        let mut client = HALClient{ url: pact_broker.url().to_string(), .. HALClient::default() };
        let result = wait(client.clone().fetch(s!("/")));
        expect!(result.clone()).to(be_ok());
        client.path_info = result.ok();
        let result = wait(client.clone().fetch_link("document", hashmap!{ s!("id") => s!("abc") }));
        expect!(result).to(be_ok().value(serde_json::Value::String(s!("Yay! You found your way here"))));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_empty_list_if_there_are_no_pacts() {
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

        let result = wait(fetch_pacts_from_broker(pact_broker.url().to_string(), s!("sad_provider")));
        expect!(result).to(be_err().value(format!("No pacts for provider 'sad_provider' where found in the pact broker. URL: '{}'",
            pact_broker.url())));
    }

    #[test]
    fn fetch_pacts_from_broker_returns_a_list_of_pacts() {
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

        let result = wait(fetch_pacts_from_broker(pact_broker.url().to_string(), s!("happy_provider")));
        expect!(result.clone()).to(be_ok());
        let pacts = result.unwrap();
        expect!(pacts.len()).to(be_equal_to(2));
        for pact in pacts {
            expect!(pact).to(be_ok());
        }
    }
}
