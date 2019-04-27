use super::*;
use pact_matching::models::*;
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use std::str::FromStr;
use std::collections::hash_map::HashMap;
use std::io::Read;
use tokio::runtime::Runtime;
use hyper::client::Client;
use hyper::client::connect::HttpConnector;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper::http::request::{Builder as RequestBuilder};
use hyper::Body;
use hyper::error::Error as HyperError;
use hyper::Method;
use hyper::http::header::HeaderName;
use hyper::http::header::CONTENT_TYPE;

pub fn join_paths(base: &String, path: String) -> String {
    let mut full_path = s!(base.trim_right_matches("/"));
    full_path.push('/');
    full_path.push_str(path.trim_left_matches("/"));
    full_path
}

fn setup_headers(builder: &mut RequestBuilder, headers: &Option<HashMap<String, String>>) -> Result<(), Box<Error>> {
    let mut hyper_headers = builder.headers_mut().unwrap();
    match headers {
        Some(header_map) => {
            for (k, v) in header_map {
                // FIXME?: Headers are not sent in "raw" mode.
                // Names are converted to lower case and values are parsed.
                hyper_headers.insert(
                    HeaderName::from_bytes(k.as_bytes())?,
                    v.parse()?
                );
            }

            if !header_map.keys().any(|k| k.to_lowercase() == "content-type") {
                hyper_headers.insert(CONTENT_TYPE, "application/json".parse()?);
            }
        }
    }
    Ok(())
}

fn make_request(base_url: &String, request: &Request, runtime: &mut Runtime) -> Result<HyperResponse<Body>, Box<Error>> {
    match Method::from_str(&request.method) {
        Ok(method) => {
            let mut url = join_paths(base_url, request.path.clone());
            if request.query.is_some() {
                url.push('?');
                url.push_str(&build_query_string(request.query.clone().unwrap()));
            }
            debug!("Making request to '{}'", url);
            let mut builder = HyperRequest::builder()
                .method(method)
                .uri(url);
            setup_headers(&mut builder, &request.headers())?;

            let mut hyper_request = builder
                .body(match request.body {
                    OptionalBody::Present(ref s) => Body::from(s.clone()),
                    OptionalBody::Null => {
                        if request.content_type() == "application/json" {
                            Body::from("null")
                        } else {
                            Body::empty()
                        }
                    },
                    _ => Body::empty()
                })?;

            runtime.block_on(
                Client::new().request(hyper_request)
            ).map_err(|err| err.into())

            /*
            let hyper_request = client.request(method, &url)
                .headers(setup_headers(&request.headers.clone()));
            match request.body {
                OptionalBody::Present(ref s) => hyper_request.body(s.as_slice()),
                OptionalBody::Null => {
                    if request.content_type() == "application/json" {
                        hyper_request.body("null")
                    } else {
                        hyper_request
                    }
                },
                _ => hyper_request
            }.send()
            */

        },
        Err(err) => Err(err.into())
    }
}

fn extract_headers(headers: &Headers) -> Option<HashMap<String, String>> {
    if headers.len() > 0 {
        Some(headers.iter().map(|h| (s!(h.name()), h.value_string()) ).collect())
    } else {
        None
    }
}

pub fn extract_body(response: &mut HyperResponse<Body>) -> OptionalBody {
    let mut buffer = Vec::new();
    match response.read_to_end(&mut buffer) {
        Ok(size) => if size > 0 {
                OptionalBody::Present(buffer)
            } else {
                OptionalBody::Empty
            },
        Err(err) => {
            warn!("Failed to read request body: {}", err);
            OptionalBody::Missing
        }
    }
}

fn hyper_response_to_pact_response(response: &mut HyperResponse<Body>) -> Response {
    Response {
        status: response.status().as_u16(),
        headers: extract_headers(&response.headers()),
        body: extract_body(response),
        matching_rules: MatchingRules::default(),
        generators: Generators::default()
    }
}

pub fn make_provider_request(provider: &ProviderInfo, request: &Request, runtime: &mut Runtime) -> Result<Response, Box<Error>> {
    debug!("Sending {:?} to provider", request);
    match make_request(&format!("{}://{}:{}{}", provider.protocol, provider.host, provider.port,
        provider.path), request, &mut runtime) {
        Ok(ref mut response) => {
            debug!("Received response: {:?}", response);
            Ok(hyper_response_to_pact_response(response))
        },
        Err(err) => {
            debug!("Request failed: {}", err);
            Err(err)
        }
    }
}

pub fn make_state_change_request(provider: &ProviderInfo, request: &Request) -> Result<(), String> {
    debug!("Sending {:?} to state change handler", request);
    let client = Client::new();
    match make_request(&provider.state_change_url.clone().unwrap(), request, &client) {
        Ok(ref mut response) => {
            debug!("Received response: {:?}", response);
            if response.status.is_success() {
                Ok(())
            } else {
                debug!("Request failed: {}", response.status);
                Err(format!("State change request failed: {}", response.status))
            }
        },
        Err(err) => {
            debug!("Request failed: {}", err);
            Err(format!("State change request failed: {}", err))
        }
    }
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use super::join_paths;

    #[test]
    fn join_paths_test() {
        expect!(join_paths(&s!(""), s!(""))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/"), s!(""))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!(""), s!("/"))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/"), s!("/"))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/a/b"), s!("/c/d"))).to(be_equal_to(s!("/a/b/c/d")));
    }

}
