use super::*;
use pact_matching::models::*;
use pact_matching::s;
use std::collections::hash_map::HashMap;
use std::convert::TryFrom;
use futures::future::*;
use log::*;
use reqwest::{RequestBuilder, Client, Error};
use http::header::CONTENT_TYPE;
use itertools::Itertools;
use http::{Method, HeaderMap, HeaderValue};
use http::method::InvalidMethod;
use http::header::{HeaderName, InvalidHeaderName, InvalidHeaderValue};
use pact_matching::models::content_types::ContentType;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ProviderClientError {
    RequestMethodError(String, InvalidMethod),
    RequestHeaderNameError(String, InvalidHeaderName),
    RequestHeaderValueError(String, InvalidHeaderValue),
    RequestBodyError(String),
    ResponseError(String),
    ResponseStatusCodeError(u16),
}

impl From<reqwest::Error> for ProviderClientError {
  fn from(err: Error) -> Self {
    ProviderClientError::ResponseError(err.to_string())
  }
}

pub fn join_paths(base: &str, path: String) -> String {
    let mut full_path = s!(base.trim_end_matches('/'));
    full_path.push('/');
    full_path.push_str(path.trim_start_matches('/'));
    full_path
}

fn create_native_request(client: &Client, base_url: &str, request: &Request) -> Result<RequestBuilder, ProviderClientError> {
  let url = join_paths(base_url, request.path.clone());
  let mut builder = client.request(Method::from_bytes(
    &request.method.clone().into_bytes()).unwrap_or(Method::GET), &url);

  if let Some(query) = &request.query {
    builder = builder.query(&query.iter()
      .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
      .flat_map(|(k, v)| {
        v.iter().map(|v| (k, v)).collect_vec()
      }).collect_vec());
  }

  if let Some(headers) = &request.headers {
    let mut header_map = HeaderMap::new();
    for (k, vals) in headers {
      for header_value in vals {
        let header_name = HeaderName::try_from(k)
          .map_err(|err| ProviderClientError::RequestHeaderNameError(
            format!("Failed to parse header value: {}", header_value), err))?;
        header_map.append(header_name,  HeaderValue::from_str(header_value.as_str())
          .map_err(|err| ProviderClientError::RequestHeaderValueError(
            format!("Failed to parse header value: {}", header_value), err))?);
      }
    }
    builder = builder.headers(header_map);
  }

  match request.body {
    OptionalBody::Present(ref s, _) => builder = builder.body(s.clone()),
    OptionalBody::Null => {
      if request.content_type().unwrap_or_default().is_json() {
        builder = builder.body("null");
      }
    },
    _ => ()
  };

  Ok(builder)
}

fn extract_headers(headers: &HeaderMap) -> Option<HashMap<String, Vec<String>>> {
  if !headers.is_empty() {
    let result = headers.keys()
      .map(|name| {
        let values = headers.get_all(name);
        let parsed_vals: Vec<Result<String, ()>> = values.iter()
          .map(|val| val.to_str()
            .map(|v| v.to_string())
            .map_err(|err| log::warn!("Failed to parse HTTP header value: {}", err))
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

async fn extract_body(response: reqwest::Response, pact_response: &Response) -> Result<OptionalBody, reqwest::Error> {
  let body = response.bytes().await?;
  if !body.is_empty() {
    Ok(OptionalBody::Present(body.to_vec(), pact_response.content_type().map(|c| c.to_string())))
  } else {
    Ok(OptionalBody::Empty)
  }
}

async fn native_response_to_pact_response(
    native_response: reqwest::Response
) -> Result<Response, reqwest::Error> {
  debug!("Received response: {:?}", native_response);

  let status = native_response.status().as_u16();
  let headers = extract_headers(native_response.headers());
  let response = Response {
    status,
    headers,
    .. Response::default()
  };

  let body = extract_body(native_response, &response).await?;

  let response = Response {
    body,
    .. response.clone()
  };
  info!("Received response: {}", response);
  Ok(response)
}

/// This function makes the actual request to the provider, executing any request filter before
/// executing the request
pub async fn make_provider_request<F: RequestFilterExecutor>(
  provider: &ProviderInfo,
  request: &Request,
  options: &VerificationOptions<F>,
  client: &reqwest::Client
) -> Result<Response, ProviderClientError> {
  let request_filter_option = options.request_filter.as_ref();
  let request = if request_filter_option.is_some() {
    let request_filter = request_filter_option.unwrap();
    info!("Invoking request filter for request");
    request_filter.call(request)
  } else {
    request.clone()
  };

  let base_url = match provider.port {
    Some(port) => format!("{}://{}:{}{}", provider.protocol, provider.host, port, provider.path),
    None => format!("{}://{}{}", provider.protocol, provider.host, provider.path),
  };

  info!("Sending request to provider at {}", base_url);
  debug!("Provider details = {:?}", provider);
  debug!("Sending request {}", request);
  trace!("body: {}", request.body.str_value());
  let request = create_native_request(client, &base_url, &request)?;

  let response = request.send()
    .and_then(native_response_to_pact_response)
    .await
    .map_err(|err| ProviderClientError::ResponseError(err.to_string()))?;

  Ok(response)
}

/// Make a state change request. If the response returns a JSON body, convert that into a HashMap
/// and return it
pub async fn make_state_change_request(
  client: &reqwest::Client,
  state_change_url: &str,
  request: &Request
) -> Result<HashMap<String, Value>, ProviderClientError> {
  log::debug!("Sending {} to state change handler", request);

  let request = create_native_request(client, state_change_url, request)?;
  let result = request.send().await;

  match result {
    Ok(response) => {
      debug!("State change request: {:?}", response);
      if response.status().is_success() {
        if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
          if let Ok(content_type) = ContentType::parse(content_type.to_str().unwrap_or_default()) {
            if content_type.is_json() {
              let body = response.bytes().await?;
              match serde_json::from_slice::<Value>(&body) {
                Ok(body) => match body {
                  Value::Object(map) => Ok(map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                  _ => Ok(hashmap!{})
                },
                Err(_) => Ok(hashmap!{})
              }
            } else {
              Ok(hashmap!{})
            }
          } else {
            Ok(hashmap!{})
          }
        } else {
          Ok(hashmap!{})
        }
      } else {
        Err(ProviderClientError::ResponseStatusCodeError(response.status().as_u16()))
      }
    },
    Err(err) => {
      debug!("State change request failed with error {}", err);
      Err(ProviderClientError::ResponseError(err.to_string()))
    }
  }
}

pub fn provider_client_error_to_string(err: ProviderClientError) -> String {
  match err {
    ProviderClientError::RequestMethodError(ref method, _) =>
      format!("Invalid request method: '{}'", method),
    ProviderClientError::RequestHeaderNameError(ref name, _) =>
      format!("Invalid header name: '{}'", name),
    ProviderClientError::RequestHeaderValueError(ref value, _) =>
      format!("Invalid header value: '{}'", value),
    ProviderClientError::RequestBodyError(ref message) =>
      format!("Invalid request body: '{}'", message),
    ProviderClientError::ResponseError(ref message) =>
      format!("Invalid response: {}", message),
    ProviderClientError::ResponseStatusCodeError(ref code) =>
      format!("Invalid status code: {}", code)
  }
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use expectest::expect;
  use super::{join_paths, create_native_request};
  use pact_matching::s;
  use pact_matching::models::{Request, OptionalBody};
  use maplit::*;
  use itertools::Itertools;

  #[test]
  fn join_paths_test() {
      expect!(join_paths(&s!(""), s!(""))).to(be_equal_to(s!("/")));
      expect!(join_paths(&s!("/"), s!(""))).to(be_equal_to(s!("/")));
      expect!(join_paths(&s!(""), s!("/"))).to(be_equal_to(s!("/")));
      expect!(join_paths(&s!("/"), s!("/"))).to(be_equal_to(s!("/")));
      expect!(join_paths(&s!("/a/b"), s!("/c/d"))).to(be_equal_to(s!("/a/b/c/d")));
  }

  #[test]
  fn convert_request_to_native_request_test() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request::default();
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body()).to(be_none());
  }

  #[test]
  fn convert_request_to_native_request_with_query_parameters() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request {
      query: Some(hashmap!{
        "a".to_string() => vec!["b".to_string()],
        "c".to_string() => vec!["d".to_string(), "e".to_string()]
      }),
      .. Request::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/?a=b&c=d&c=e"));
  }

  #[test]
  fn convert_request_to_native_request_with_headers() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request {
      headers: Some(hashmap! {
        "A".to_string() => vec!["B".to_string()],
        "B".to_string() => vec!["C".to_string(), "D".to_string()]
      }),
      .. Request::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));

    let headers = request_builder.headers();
    expect!(headers.len()).to(be_equal_to(3));
    expect!(&headers["A"]).to(be_equal_to("B"));
    expect!(&headers["B"]).to(be_equal_to("C"));
    expect!(headers.get_all("B").iter().map(|v| v.to_str().unwrap()).collect_vec())
      .to(be_equal_to(vec!["C", "D"]));
  }

  #[test]
  fn convert_request_to_native_request_with_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request {
      body: OptionalBody::from("body"),
      .. Request::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body().unwrap().as_bytes()).to(be_some().value("body".as_bytes()));
  }

  #[test]
  fn convert_request_to_native_request_with_null_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request {
      body: OptionalBody::Null,
      .. Request::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body()).to(be_none());
  }

  #[test]
  fn convert_request_to_native_request_with_json_null_body() {
    let client = reqwest::Client::new();
    let base_url = "http://example.test:8080".to_string();
    let request = Request {
      headers: Some(hashmap! {
        "Content-Type".to_string() => vec!["application/json".to_string()]
      }),
      body: OptionalBody::Null,
      .. Request::default()
    };
    let request_builder = create_native_request(&client, &base_url, &request).unwrap().build().unwrap();

    expect!(request_builder.method()).to(be_equal_to("GET"));
    expect!(request_builder.url().as_str()).to(be_equal_to("http://example.test:8080/"));
    expect!(request_builder.body().unwrap().as_bytes()).to(be_some().value("null".as_bytes()));
  }
}
