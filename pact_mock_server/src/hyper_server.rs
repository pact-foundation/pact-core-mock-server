//! Mock server implementation using Hyper

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
#[cfg(feature = "tls")] use std::sync::Arc;
use std::time::Duration;

#[allow(unused_imports)] use anyhow::anyhow;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response};
use hyper::body::Incoming;
use hyper::header::{HeaderName, HeaderValue};
use hyper::http::response::Builder;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use itertools::Itertools;
use maplit::hashmap;
use pact_matching::logging::LOG_ID;
use pact_models::bodies::OptionalBody;
use pact_models::generators::GeneratorTestMode;
use pact_models::headers::parse_header;
use pact_models::http_parts::HttpPart;
use pact_models::query_strings::parse_query_string;
use pact_models::v4::calc_content_type;
use pact_models::v4::http_parts::HttpRequest;
use pact_models::v4::pact::V4Pact;
#[cfg(feature = "tls")] use rcgen::{CertifiedKey, generate_simple_self_signed};
#[cfg(feature = "tls")] use rustls::crypto::ring::default_provider;
#[cfg(feature = "tls")] use rustls::crypto::CryptoProvider;
#[cfg(feature = "tls")] use rustls::pki_types::PrivateKeyDer;
#[cfg(feature = "tls")] use rustls::ServerConfig;
use serde_json::json;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::Sender;
use tokio::task::{JoinHandle, JoinSet};
use tokio::time::sleep;
#[cfg(feature = "tls")] use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, trace, warn};

use crate::matching::{match_request, MatchResult};
use crate::mock_server::{MockServerConfig, MockServerEvent};
use crate::mock_server::MockServerEvent::{ConnectionFailed, ServerShutdown};

#[derive(Debug, Clone)]
pub(crate) enum InteractionError {
  RequestHeaderEncodingError,
  RequestBodyError,
  ResponseHeaderEncodingError,
  ResponseBodyError
}

impl Display for InteractionError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      InteractionError::RequestHeaderEncodingError => write!(f, "Found an invalid header encoding"),
      InteractionError::RequestBodyError => write!(f, "Could not process request body"),
      InteractionError::ResponseBodyError => write!(f, "Could not process response body"),
      InteractionError::ResponseHeaderEncodingError => write!(f, "Could not set response header")
    }
  }
}

impl std::error::Error for InteractionError {}

/// Create and bind the server, spawning the server loop onto the runtime and returning the bound
/// address, the send end of the shutdown channel and the receive end of the event channel
pub(crate) async fn create_and_bind(
  server_id: String,
  pact: V4Pact,
  addr: SocketAddr,
  config: MockServerConfig
) -> anyhow::Result<(SocketAddr, oneshot::Sender<()>, mpsc::Receiver<MockServerEvent>, JoinHandle<()>)> {
  let listener = TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;

  let mut join_set = JoinSet::new();
  let (shutdown_send, mut shutdown_recv) = oneshot::channel::<()>();
  let (event_send, event_recv) = mpsc::channel::<MockServerEvent>(256);

  let handle = tokio::spawn(async move {
    loop {
      let event_send = event_send.clone();
      let server_id = server_id.clone();
      let pact = pact.clone();
      let config = config.clone();

      select! {
        connection = listener.accept() => {
          match connection {
            Ok((stream, remote_address)) => {
              debug!("Received connection from remote {}", remote_address);
              let io = TokioIo::new(stream);
              join_set.spawn(LOG_ID.scope(server_id.clone(), async move {
                if let Err(err) = http1::Builder::new()
                  .keep_alive(config.keep_alive)
                  .serve_connection(io, service_fn(|req: Request<Incoming>| {
                    let pact = pact.clone();
                    let event_send = event_send.clone();
                    let config = config.clone();
                    LOG_ID.scope(server_id.clone(), async move {
                      handle_mock_request_error(
                        handle_request(req, pact.clone(), event_send.clone(), &local_addr, &config).await
                      )
                    })
                  }))
                  .await {
                    error!("failed to serve connection: {err}");
                    if let Err(err) = event_send.send(ConnectionFailed(err.to_string())).await {
                      error!("Failed to send ConnectionFailed event: {}", err);
                    }
                }
              }));
            },
            Err(e) => {
              error!("failed to accept connection: {e}");
              if let Err(err) = event_send.send(ConnectionFailed(e.to_string())).await {
                error!("Failed to send ConnectionFailed event: {}", err);
              }
            }
          }
        }

        _ = &mut shutdown_recv => {
          trace!("Received shutdown signal, waiting for existing connections to complete");
          while let Some(_) = join_set.join_next().await {};
          trace!("Existing connections complete, exiting main loop");
          if let Err(err) = event_send.send(ServerShutdown).await {
            error!("Failed to send ServerShutdown event: {}", err);
          }
          break;
        }
      }
    }

    trace!("Mock server main loop done");
  });

  Ok((local_addr, shutdown_send, event_recv, handle))
}

/// Create and bind the HTTPS server, spawning the server loop onto the runtime and returning the bound
/// address, the send end of the shutdown channel and the receive end of the event channel. If no
/// HTTPS configuration has been supplied, a self-signed certificate will be created to be used.
#[cfg(feature = "tls")]
pub(crate) async fn create_and_bind_https(
  server_id: String,
  pact: V4Pact,
  addr: SocketAddr,
  config: MockServerConfig
) -> anyhow::Result<(SocketAddr, oneshot::Sender<()>, mpsc::Receiver<MockServerEvent>, JoinHandle<()>)> {
  if CryptoProvider::get_default().is_none() {
    warn!("No TLS cryptographic provider has been configured, defaulting to the standard FIPS provider");
    CryptoProvider::install_default(default_provider())
      .map_err(|_| anyhow!("Failed to install the standard FIPS provider"))?;
  }

  let listener = TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;

  let mut join_set = JoinSet::new();
  let (shutdown_send, mut shutdown_recv) = oneshot::channel::<()>();
  let (event_send, event_recv) = mpsc::channel::<MockServerEvent>(256);

  let tls_config = match &config.tls_config {
    Some(config) => config.clone(),
    None => {
      let CertifiedKey { cert, key_pair } = generate_simple_self_signed(["localhost".to_string()])?;
      let private_key = PrivateKeyDer::try_from(key_pair.serialize_der())
        .map_err(|err| anyhow!(err))?;
      ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![ cert.der().clone() ], private_key)?
    }
  };
  let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));

  let handle = tokio::spawn(async move {
    loop {
      let event_send = event_send.clone();
      let server_id = server_id.clone();
      let pact = pact.clone();
      let config = config.clone();

      select! {
        connection = listener.accept() => {
          match connection {
            Ok((stream, remote_address)) => {
              debug!("Received connection from remote {}", remote_address);

              let tls_acceptor = tls_acceptor.clone();
              match tls_acceptor.accept(stream).await {
                Ok(tls_stream) => {
                  let io = TokioIo::new(tls_stream);
                  join_set.spawn(LOG_ID.scope(server_id.clone(), async move {
                    if let Err(err) = http1::Builder::new()
                      .keep_alive(false)
                      .serve_connection(io, service_fn(|req: Request<Incoming>| {
                        let pact = pact.clone();
                        let event_send = event_send.clone();
                        let config = config.clone();
                        LOG_ID.scope(server_id.clone(), async move {
                          handle_mock_request_error(
                            handle_request(req, pact.clone(), event_send.clone(), &local_addr, &config).await
                          )
                        })
                      }))
                      .await {
                        error!("failed to serve connection: {err}");
                        if let Err(err) = event_send.send(ConnectionFailed(err.to_string())).await {
                          error!("Failed to send ConnectionFailed event: {}", err);
                        }
                    }
                  }));
                },
                Err(err) => {
                  error!("failed to perform tls handshake: {err:#}");
                  if let Err(err) = event_send.send(ConnectionFailed(err.to_string())).await {
                    error!("Failed to send ConnectionFailed event: {}", err);
                  }
                }
              };
            },
            Err(e) => {
              error!("failed to accept connection: {e}");
              if let Err(err) = event_send.send(ConnectionFailed(e.to_string())).await {
                error!("Failed to send ConnectionFailed event: {}", err);
              }
            }
          }
        }

        _ = &mut shutdown_recv => {
          debug!("Received shutdown signal, waiting for existing connections to complete");
          while let Some(_) = join_set.join_next().await {};
          debug!("Waiting for event loop to complete");
          sleep(Duration::from_millis(100)).await;
          debug!("Existing connections complete, exiting main loop");
          drop(event_send);
          break;
        }
      }
    }
  });

  Ok((local_addr, shutdown_send, event_recv, handle))
}

/// Main hyper request handler
async fn handle_request(
  req: Request<Incoming>,
  pact: V4Pact,
  event_send: Sender<MockServerEvent>,
  local_addr: &SocketAddr,
  config: &MockServerConfig
) -> Result<Response<Full<Bytes>>, InteractionError> {
  let path = req.uri().path().to_string();
  debug!(%path, "Creating pact request from hyper request");

  if let Err(_) = event_send.send(MockServerEvent::RequestReceived(path)).await {
    error!("Failed to send RequestReceived event");
  }

  let pact_request = hyper_request_to_pact_request(req).await?;
  info!("Received request {} {}", pact_request.method, pact_request.path);
  if pact_request.has_text_body() {
    debug!(
      "
      ----------------------------------------------------------------------------------------
       method: {}
       path: {}
       query: {:?}
       headers: {:?}
       body: {} '{}'
      ----------------------------------------------------------------------------------------
      ",
      pact_request.method, pact_request.path, pact_request.query, pact_request.headers,
      pact_request.body, pact_request.body.display_string()
    );
  } else {
    debug!(
      "
      ----------------------------------------------------------------------------------------
       method: {}
       path: {}
       query: {:?}
       headers: {:?}
       body: {}
      ----------------------------------------------------------------------------------------
      ",
      pact_request.method, pact_request.path, pact_request.query, pact_request.headers,
      pact_request.body
    );
  }

  let match_result = match_request(&pact_request, &pact).await;

  if let Err(_) = event_send.send(MockServerEvent::RequestMatch(match_result.clone())).await {
    error!("Failed to send RequestMatch event");
  }

  match_result_to_hyper_response(&pact_request, &match_result, local_addr, config).await
}

fn handle_mock_request_error(result: Result<Response<Full<Bytes>>, InteractionError>) -> Result<Response<Full<Bytes>>, hyper::Error> {
  match result {
    Ok(response) => Ok(response),
    Err(error) => {
      let response = match error {
        InteractionError::RequestHeaderEncodingError => Response::builder()
          .status(500)
          .body(Full::new(Bytes::from("Found an invalid header encoding"))),
        InteractionError::RequestBodyError => Response::builder()
          .status(500)
          .body(Full::new(Bytes::from("Could not process request body"))),
        InteractionError::ResponseBodyError => Response::builder()
          .status(500)
          .body(Full::new(Bytes::from("Could not process response body"))),
        InteractionError::ResponseHeaderEncodingError => Response::builder()
          .status(500)
          .body(Full::new(Bytes::from("Could not set response header")))
      };
      Ok(response.unwrap())
    }
  }
}

fn extract_query_string(
  uri: &hyper::Uri
) -> Option<HashMap<String, Vec<Option<String>>>> {
  debug!("Extracting query from uri {:?}", uri);
  uri.query()
    .and_then(|query| {
      trace!("query -> {:?}", query);
      parse_query_string(query)
    })
}

fn extract_headers(
  headers: &hyper::HeaderMap
) -> Result<Option<HashMap<String, Vec<String>>>, InteractionError> {
  if !headers.is_empty() {
    let mut header_map = hashmap!{};
    for header in headers.keys() {
      let values = headers.get_all(header);
      let parsed_vals = values.iter()
        .map(|val| val.to_str()
          .map(|v| v.to_string())
          .map_err(|err| {
            warn!("Failed to parse HTTP header value: {}", err);
            InteractionError::RequestHeaderEncodingError
          })
        ).collect_vec();
      if parsed_vals.iter().find(|val| val.is_err()).is_some() {
        return Err(InteractionError::RequestHeaderEncodingError)
      } else {
        header_map.insert(header.as_str().to_string(), parsed_vals.iter().cloned()
          .map(|val| val.unwrap_or_default())
          .flat_map(|val| parse_header(header.as_str(), val.as_str()))
          .collect());
      }
    }
    Ok(Some(header_map))
  } else {
    Ok(None)
  }
}

fn extract_body(bytes: Bytes) -> OptionalBody {
  if bytes.len() > 0 {
    OptionalBody::Present(bytes, None, None)
  } else {
    OptionalBody::Empty
  }
}

async fn hyper_request_to_pact_request(req: Request<Incoming>) -> Result<HttpRequest, InteractionError> {
  let method = req.method().to_string();
  let path = req.uri().path().to_string();
  let query = extract_query_string(req.uri());
  let headers = extract_headers(req.headers())?;

  let body_bytes = req.collect().await
    .map(|b| b.to_bytes())
    .map_err(|err| {
      error!("Failed to read request body: {}", err);
      InteractionError::RequestBodyError
    })?;
  let body = extract_body(body_bytes);
  let content_type = calc_content_type(&body, &headers);

  Ok(HttpRequest {
    method,
    path,
    query,
    headers,
    body: body.with_content_type(content_type),
    .. HttpRequest::default()
  })
}

async fn match_result_to_hyper_response(
  request: &HttpRequest,
  match_result: &MatchResult,
  local_addr: &SocketAddr,
  config: &MockServerConfig
) -> Result<Response<Full<Bytes>>, InteractionError> {
  let cors_preflight = config.cors_preflight;
  let context = hashmap!{
    "mockServer" => json!({
      "url": local_addr.to_string(),
      "port": local_addr.port()
    })
  };

  let origin = match request.headers.clone() {
    Some(ref h) => h.iter()
      .find(|kv| kv.0.to_lowercase() == "origin")
      .map(|kv| kv.1.clone().join(", ")).unwrap_or("*".to_string()),
    None => "*".to_string()
  };

  match match_result {
    MatchResult::RequestMatch(_, ref response, _) => {
      debug!("Test context = {:?}", context);
      let response = pact_matching::generate_response(response, &GeneratorTestMode::Consumer, &context).await;
      info!("Request matched, sending response");
      if response.has_text_body() {
        debug!(
          "
          ----------------------------------------------------------------------------------------
           status: {}
           headers: {:?}
           body: {} '{}'
          ----------------------------------------------------------------------------------------
          ", response.status, response.headers, response.body, response.body.display_string()
        );
      } else {
        debug!(
          "
          ----------------------------------------------------------------------------------------
           status: {}
           headers: {:?}
           body: {}
          ----------------------------------------------------------------------------------------
          ", response.status, response.headers, response.body
        );
      }

      let mut builder = Response::builder()
        .status(response.status)
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, &origin)
        .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, "*")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH")
        .header(hyper::header::ACCESS_CONTROL_EXPOSE_HEADERS, "Location, Link")
        .header(hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true");

      set_hyper_headers(&mut builder, &response.headers)?;

      builder.body(match response.body {
        OptionalBody::Present(b, _, _) => Full::new(b),
        _ => Full::new(Bytes::new())
      })
        .map_err(|_| InteractionError::ResponseBodyError)
    },
    _ => {
      debug!("Request did not match: {}", match_result);
      if cors_preflight && request.method.to_uppercase() == "OPTIONS" {
        info!("Responding to CORS pre-flight request");
        let cors_headers = match request.headers.clone() {
          Some(ref h) => h.iter()
            .find(|kv| kv.0.to_lowercase() == "access-control-request-headers")
            .map(|kv| kv.1.clone().join(", ") + ", *").unwrap_or("*".to_string()),
          None => "*".to_string()
        };

        Response::builder()
          .status(204)
          .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, &origin)
          .header(hyper::header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD, POST, PUT, DELETE, CONNECT, OPTIONS, TRACE, PATCH")
          .header(hyper::header::ACCESS_CONTROL_ALLOW_HEADERS, cors_headers)
          .header(hyper::header::ACCESS_CONTROL_EXPOSE_HEADERS, "Location, Link")
          .header(hyper::header::ACCESS_CONTROL_ALLOW_CREDENTIALS, "true")
          .body(Full::new(Bytes::new()))
          .map_err(|_| InteractionError::ResponseBodyError)
      } else {
        Response::builder()
          .status(500)
          .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
          .header(hyper::header::CONTENT_TYPE, "application/json; charset=utf-8")
          .header("X-Pact", match_result.match_key())
          .body(Full::new(Bytes::from(error_body(&request, &match_result.match_key()))))
          .map_err(|_| InteractionError::ResponseBodyError)
      }
    }
  }
}

fn set_hyper_headers(builder: &mut Builder, headers: &Option<HashMap<String, Vec<String>>>) -> Result<(), InteractionError> {
  let hyper_headers = builder.headers_mut().unwrap();
  match headers {
    Some(header_map) => {
      for (k, v) in header_map {
        for val in v {
          hyper_headers.append(
          HeaderName::from_bytes(k.as_bytes())
              .map_err(|err| {
                  error!("Invalid header name '{}' ({})", k, err);
                  InteractionError::ResponseHeaderEncodingError
              })?,
          val.parse::<HeaderValue>()
              .map_err(|err| {
                  error!("Invalid header value '{}': '{}' ({})", k, val, err);
                  InteractionError::ResponseHeaderEncodingError
              })?
          );
        }
      }
    },
    _ => {}
  }
  Ok(())
}

fn error_body(request: &HttpRequest, error: &String) -> String {
  let body = json!({ "error" : format!("{} : {:?}", error, request) });
  body.to_string()
}

#[cfg(test)]
mod tests {
  use expectest::prelude::*;
  use hyper::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
  use hyper::HeaderMap;
  use pact_models::pact::Pact;
  use pact_models::sync_pact::RequestResponsePact;

  use super::*;

  #[tokio::test]
  async fn can_fetch_results_on_current_thread() {
    let (_addr, shutdown, mut events, handle) = create_and_bind(
      "can_fetch_results_on_current_thread".to_string(),
      RequestResponsePact::default().as_v4_pact().unwrap(),
      ([0, 0, 0, 0], 0u16).into(),
      MockServerConfig::default()
    ).await.unwrap();

    shutdown.send(()).unwrap();
    let _ = handle.await;

    // Only the shutdown event should be generated
    assert_eq!(events.len(), 1);
    assert_eq!(events.recv().await.unwrap(), ServerShutdown);
  }

  #[test]
  fn handle_hyper_headers_with_multiple_values() {
    let mut headers = HeaderMap::new();
    headers.append(ACCEPT, "application/xml, application/json".parse().unwrap());
    headers.append(USER_AGENT, "test".parse().unwrap());
    headers.append(USER_AGENT, "test2".parse().unwrap());
    headers.append(CONTENT_TYPE, "text/plain".parse().unwrap());
    let result = extract_headers(&headers);
    expect!(result).to(be_ok().value(Some(hashmap! {
      "accept".to_string() => vec!["application/xml".to_string(), "application/json".to_string()],
      "user-agent".to_string() => vec!["test".to_string(), "test2".to_string()],
      "content-type".to_string() => vec!["text/plain".to_string()]
    })));
  }
}
