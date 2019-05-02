use super::*;
use pact_matching::models::*;
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use std::str::FromStr;
use std::collections::hash_map::HashMap;
use hyper::client::Client;
use hyper::{Request as HyperRequest, Response as HyperResponse};
use hyper::http::request::{Builder as RequestBuilder};
use hyper::Body;
use hyper::error::Error as HyperError;
use hyper::Method;
use hyper::http::method::InvalidMethod;
use hyper::http::header::{HeaderMap, HeaderName, HeaderValue, InvalidHeaderName, InvalidHeaderValue};
use hyper::http::header::CONTENT_TYPE;
use tokio::runtime::current_thread::Runtime;
use futures::future;
use futures::future::Future;
use futures::stream::Stream;

#[derive(Debug)]
pub enum ProviderClientError {
    RequestMethodError(String, InvalidMethod),
    RequestHeaderNameError(String, InvalidHeaderName),
    RequestHeaderValueError(String, InvalidHeaderValue),
    RequestBodyError(String),
    ResponseError(String),
    ResponseStatusCodeError(u16),
}

pub fn join_paths(base: &String, path: String) -> String {
    let mut full_path = s!(base.trim_right_matches("/"));
    full_path.push('/');
    full_path.push_str(path.trim_left_matches("/"));
    full_path
}

fn setup_headers(builder: &mut RequestBuilder, headers: &Option<HashMap<String, String>>) -> Result<(), ProviderClientError> {
    let mut hyper_headers = builder.headers_mut().unwrap();
    match headers {
        Some(header_map) => {
            for (k, v) in header_map {
                // FIXME?: Headers are not sent in "raw" mode.
                // Names are converted to lower case and values are parsed.
                hyper_headers.insert(
                    HeaderName::from_bytes(k.as_bytes())
                        .map_err(|err| ProviderClientError::RequestHeaderNameError(k.clone(), err))?,
                    v.parse::<HeaderValue>()
                        .map_err(|err| ProviderClientError::RequestHeaderValueError(v.clone(), err))?
                );
            }

            if !header_map.keys().any(|k| k.to_lowercase() == "content-type") {
                hyper_headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());
            }
        },
        _ => {}
    }
    Ok(())
}

fn create_hyper_request(base_url: &String, request: &Request) -> Result<HyperRequest<Body>, ProviderClientError> {
    let mut url = join_paths(base_url, request.path.clone());
    if request.query.is_some() {
        url.push('?');
        url.push_str(&build_query_string(request.query.clone().unwrap()));
    }
    debug!("Making request to '{}'", url);
    let mut builder = HyperRequest::builder();
    builder.method(
        Method::from_str(&request.method)
            .map_err(|err| ProviderClientError::RequestMethodError(request.method.clone(), err))?
    );
    builder.uri(url);
    setup_headers(&mut builder, &request.headers())?;

    let hyper_request = builder
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
        })
        .map_err(|err| ProviderClientError::RequestBodyError(err.description().into()))?;

    Ok(hyper_request)
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
        .map(move |body| {
            Response {
                status: status,
                headers: headers,
                body: body,
                matching_rules: MatchingRules::default(),
                generators: Generators::default()
            }
        })
}

fn check_hyper_response_status(result: Result<HyperResponse<Body>, HyperError>) -> Result<(), ProviderClientError> {
    match result {
        Ok(response) => {
            if response.status().is_success() {
                Ok(())
            } else {
                Err(ProviderClientError::ResponseStatusCodeError(response.status().as_u16()))
            }
        },
        Err(err) => Err(ProviderClientError::ResponseError(err.description().into()))
    }
}

pub fn make_provider_request(provider: &ProviderInfo, request: &Request, runtime: &mut Runtime) -> Result<Response, ProviderClientError> {
    debug!("Sending {:?} to provider", request);
    let base_url = format!("{}://{}:{}{}", provider.protocol, provider.host, provider.port, provider.path);

    runtime.block_on(
        future::done(create_hyper_request(&base_url, request))
            .and_then(|request| {
                Client::new().request(request)
                    .and_then(hyper_response_to_pact_response)
                    .map_err(|err| ProviderClientError::ResponseError(err.description().into()))
            })
    ).map_err(|err| {
        debug!("Request failed: {:?}", err);
        err
    })
}

pub fn make_state_change_request(provider: &ProviderInfo, request: &Request, runtime: &mut Runtime) -> Result<(), ProviderClientError> {
    debug!("Sending {:?} to state change handler", request);

    runtime.block_on(
        future::done(create_hyper_request(&provider.state_change_url.clone().unwrap(), request))
            .and_then(|request| {
                Client::new().request(request)
                    .then(check_hyper_response_status)
            })
    )
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
