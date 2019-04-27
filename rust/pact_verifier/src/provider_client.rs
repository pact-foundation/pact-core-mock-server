use super::*;
use pact_matching::models::*;
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use std::str::FromStr;
use std::collections::hash_map::HashMap;
use std::io::Read;
use tokio::runtime::Runtime;
use hyper::client::Client;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper::http::request::{Builder as RequestBuilder};
use hyper::Body;
use hyper::error::Error as HyperError;
use hyper::client::ResponseFuture;
use hyper::Method;
use hyper::http::header::{HeaderMap, HeaderName};
use hyper::http::header::CONTENT_TYPE;
use hyper::rt::{Future, Stream};

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

fn make_request(base_url: &String, request: &Request) -> Result<ResponseFuture, Box<Error>> {
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

            Ok(Client::new().request(hyper_request))
        },
        Err(err) => Err(err.into())
    }
}

fn extract_headers(headers: &HeaderMap) -> Option<HashMap<String, String>> {
    if headers.len() > 0 {
        Some(headers.iter()
            .map(|(name, value)| {
                (name.as_str().into(), value.to_str().unwrap().into())
            })
            .collect()
        )
    } else {
        None
    }
}

pub fn extract_body(hyper_body: Result<hyper::Chunk, HyperError>) -> Result<OptionalBody, HyperError> {
    match hyper_body {
        Ok(chunk) => {
            let bytes = chunk.into_bytes();
            if bytes.len() > 0 {
                Ok(OptionalBody::Present(bytes.to_vec()))
            } else {
                Ok(OptionalBody::Empty)
            }
        },
        Err(err) => {
            warn!("Failed to read request body: {}", err);
            Ok(OptionalBody::Missing)
        }
    }
}

fn hyper_response_to_pact_response(response: HyperResponse<Body>) -> impl Future<Item = Response, Error = HyperError> {
    debug!("Received response: {:?}", response);

    let status = response.status().as_u16();
    let headers = extract_headers(response.headers());

    response.into_body()
        .concat2()
        .then(extract_body)
        .map(|body| {
            Response {
                status: status,
                headers: headers,
                body: body,
                matching_rules: MatchingRules::default(),
                generators: Generators::default()
            }
        })
}

pub fn make_provider_request(provider: &ProviderInfo, request: &Request, runtime: &mut Runtime) -> Result<Response, Box<Error>> {
    debug!("Sending {:?} to provider", request);
    match make_request(&format!("{}://{}:{}{}", provider.protocol, provider.host, provider.port,
        provider.path), request) {
        Ok(ref mut response_future) => {
            let pact_response_future = response_future
                .and_then(hyper_response_to_pact_response);

            runtime.block_on(pact_response_future)
                .map_err(|err| {
                    debug!("Response processing failed: {}", err);
                    err.into()
                })
        },
        Err(err) => {
            debug!("Request failed: {}", err);
            Err(err)
        }
    }
}

pub fn make_state_change_request(provider: &ProviderInfo, request: &Request, runtime: &mut Runtime) -> Result<(), String> {
    debug!("Sending {:?} to state change handler", request);
    let client = Client::new();
    match make_request(&provider.state_change_url.clone().unwrap(), request) {
        Ok(ref mut response_future) => {
            let response = runtime.block_on(response_future);
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
