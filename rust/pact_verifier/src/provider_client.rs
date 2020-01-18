use super::*;
use pact_matching::models::*;
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use pact_matching::s;
use std::str::FromStr;
use std::error::Error;
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
use futures::future::*;

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
    let mut full_path = s!(base.trim_end_matches("/"));
    full_path.push('/');
    full_path.push_str(path.trim_start_matches("/"));
    full_path
}

fn setup_headers(builder: &mut RequestBuilder, headers: &Option<HashMap<String, Vec<String>>>) -> Result<(), ProviderClientError> {
  let hyper_headers = builder.headers_mut().unwrap();
  match headers {
    Some(header_map) => {
      for (k, v) in header_map {
        for val in v {
          // FIXME?: Headers are not sent in "raw" mode.
          // Names are converted to lower case and values are parsed.
          hyper_headers.append(
            HeaderName::from_bytes(k.as_bytes())
              .map_err(|err| ProviderClientError::RequestHeaderNameError(k.clone(), err))?,
            val.parse::<HeaderValue>()
              .map_err(|err| ProviderClientError::RequestHeaderValueError(val.clone(), err))?
          );
        }
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
    log::debug!("Making request to '{}'", url);
    let mut builder = HyperRequest::builder()
        .method(
            Method::from_str(&request.method)
                .map_err(|err| ProviderClientError::RequestMethodError(request.method.clone(), err))?
        )
        .uri(url);
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

fn extract_headers(headers: &HeaderMap) -> Option<HashMap<String, Vec<String>>> {
  if !headers.is_empty() {
    let result = headers.keys()
      .map(|name| {
        let values = headers.get_all(name);
        let parsed_vals: Vec<Result<String, ()>> = values.iter()
          .map(|val| val.to_str()
            .map(|v| v.to_string())
            .map_err(|err| {
              log::warn!("Failed to parse HTTP header value: {}", err);
              ()
            })
          ).collect();
        (name.as_str().into(), parsed_vals.iter().cloned()
          .filter(|val| val.is_ok())
          .map(|val| val.unwrap_or_default())
          .collect())
      })
      .collect();

    Some(result)
  } else {
    None
  }
}

pub fn extract_body(hyper_body: Result<bytes::Bytes, HyperError>) -> Result<OptionalBody, HyperError> {
    match hyper_body {
        Ok(bytes) => {
            if bytes.len() > 0 {
                Ok(OptionalBody::Present(bytes.to_vec()))
            } else {
                Ok(OptionalBody::Empty)
            }
        },
        Err(err) => {
            log::warn!("Failed to read request body: {}", err);
            Ok(OptionalBody::Missing)
        }
    }
}

async fn hyper_response_to_pact_response(
    response: HyperResponse<Body>
) -> Result<Response, HyperError> {
    log::debug!("Received response: {:?}", response);

    let status = response.status().as_u16();
    let headers = extract_headers(response.headers());
    let body = extract_body(hyper::body::to_bytes(response.into_body()).await)?;

    Ok(
        Response {
            status,
            headers,
            body,
            matching_rules: MatchingRules::default(),
            generators: Generators::default(),
        }
    )
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

pub async fn make_provider_request(
    provider: &ProviderInfo,
    request: &Request
) -> Result<Response, ProviderClientError> {
    log::debug!("Sending {:?} to provider", request);

    let base_url = format!("{}://{}:{}{}", provider.protocol, provider.host, provider.port, provider.path);
    let request = create_hyper_request(&base_url, request)?;

    let response = Client::new().request(request)
        .and_then(hyper_response_to_pact_response)
        .await
        .map_err(|err| ProviderClientError::ResponseError(err.description().into()))?;

    Ok(response)
}

pub async fn make_state_change_request(
    provider: &ProviderInfo,
    request: &Request
) -> Result<(), ProviderClientError> {
    log::debug!("Sending {:?} to state change handler", request);

    let request = create_hyper_request(&provider.state_change_url.clone().unwrap(), request)?;
    let result = Client::new().request(request).await;

    check_hyper_response_status(result)
}

#[cfg(test)]
mod tests {
    use expectest::prelude::*;
    use expectest::expect;
    use super::join_paths;
    use pact_matching::s;

    #[test]
    fn join_paths_test() {
        expect!(join_paths(&s!(""), s!(""))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/"), s!(""))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!(""), s!("/"))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/"), s!("/"))).to(be_equal_to(s!("/")));
        expect!(join_paths(&s!("/a/b"), s!("/c/d"))).to(be_equal_to(s!("/a/b/c/d")));
    }

}
