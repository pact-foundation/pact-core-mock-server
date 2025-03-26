use std::borrow::BorrowMut;
use std::collections::HashMap;
#[cfg(feature = "tls")]  use std::io;
use std::net::SocketAddr;
#[cfg(feature = "tls")] use std::pin::Pin;
use std::sync::{Arc, Mutex};

#[cfg(feature = "tls")] use futures::prelude::*;
#[cfg(feature = "tls")] use futures::StreamExt;
#[cfg(feature = "tls")] use futures::task::{Context, Poll};
use hyper::{Body, Error, Response, Server};
use hyper::http::header::{HeaderName, HeaderValue};
use hyper::http::response::Builder as ResponseBuilder;
use hyper::service::make_service_fn;
use hyper::service::service_fn;
use maplit::*;
use pact_models::bodies::OptionalBody;
use pact_models::generators::GeneratorTestMode;
use pact_models::http_parts::HttpPart;
use pact_models::pact::Pact;
use pact_models::query_strings::parse_query_string;
use pact_models::v4::http_parts::HttpRequest;
#[cfg(feature = "tls")] use rustls::ServerConfig;
use serde_json::json;
#[cfg(feature = "tls")] use tokio::net::{TcpListener, TcpStream};
use tokio::task_local;
#[cfg(feature = "tls")] use tokio_rustls::server::TlsStream;
#[cfg(feature = "tls")] use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, trace, warn};

use crate::matching::{match_request, MatchResult};
use crate::mock_server::MockServer;

task_local! {
  /// Log ID to accumulate logs against
  #[allow(missing_docs)]
  #[deprecated(note = "This must be moved to the FFI crate")]
  pub static LOG_ID: String;
}

#[derive(Debug, Clone)]
enum InteractionError {
    RequestHeaderEncodingError,
    RequestBodyError,
    ResponseHeaderEncodingError,
    ResponseBodyError
}

fn extract_path(uri: &hyper::Uri) -> String {
    uri.path_and_query()
        .map(|path_and_query| path_and_query.path())
        .unwrap_or("/")
        .into()
}

fn extract_query_string(uri: &hyper::Uri) -> Option<HashMap<String, Vec<Option<String>>>> {
  debug!("Extracting query from uri {:?}", uri);
  uri.path_and_query()
    .and_then(|path_and_query| {
      trace!("path_and_query -> {:?}", path_and_query);
      path_and_query.query()
    })
    .and_then(|query| parse_query_string(query))
}

fn extract_headers(headers: &hyper::HeaderMap) -> Result<Option<HashMap<String, Vec<String>>>, InteractionError> {
  if !headers.is_empty() {
    let result: Result<HashMap<String, Vec<String>>, InteractionError> = headers.keys()
      .map(|name| -> Result<(String, Vec<String>), InteractionError> {
        let values = headers.get_all(name);
        let parsed_vals: Vec<Result<String, InteractionError>> = values.iter()
          .map(|val| val.to_str()
            .map(|v| v.to_string())
            .map_err(|err| {
              warn!("Failed to parse HTTP header value: {}", err);
              InteractionError::RequestHeaderEncodingError
            })
          ).collect();
        if parsed_vals.iter().find(|val| val.is_err()).is_some() {
          Err(InteractionError::RequestHeaderEncodingError)
        } else {
          Ok((name.as_str().into(), parsed_vals.iter().cloned()
            .map(|val| val.unwrap_or_default())
            .flat_map(|val| val.split(",").map(|v| v.to_string()).collect::<Vec<String>>())
            .map(|val| val.trim().to_string())
            .collect()))
        }
      })
      .collect();

    result.map(|map| Some(map))
  } else {
    Ok(None)
  }
}

fn extract_body(bytes: bytes::Bytes, request: &HttpRequest) -> OptionalBody {
    if bytes.len() > 0 {
      OptionalBody::Present(bytes, request.content_type(), None)
    } else {
      OptionalBody::Empty
    }
}

async fn hyper_request_to_pact_request(req: hyper::Request<Body>) -> Result<HttpRequest, InteractionError> {
    let method = req.method().to_string();
    let path = extract_path(req.uri());
    let query = extract_query_string(req.uri());
    let headers = extract_headers(req.headers())?;

    let body_bytes = hyper::body::to_bytes(req.into_body())
        .await
        .map_err(|_| InteractionError::RequestBodyError)?;

    let request = HttpRequest {
      method,
      path,
      query,
      headers,
      .. HttpRequest::default()
    };

    Ok(HttpRequest {
      body: extract_body(body_bytes, &request),
      .. request.clone()
    })
}

fn set_hyper_headers(builder: &mut ResponseBuilder, headers: &Option<HashMap<String, Vec<String>>>) -> Result<(), InteractionError> {
    let hyper_headers = builder.headers_mut().unwrap();
    match headers {
        Some(header_map) => {
            for (k, v) in header_map {
                for val in v {
                    // FIXME?: Headers are not sent in "raw" mode.
                    // Names are converted to lower case and values are parsed.
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

async fn match_result_to_hyper_response(
  request: &HttpRequest,
  match_result: MatchResult,
  mock_server: Arc<Mutex<MockServer>>
) -> Result<Response<Body>, InteractionError> {
  let (context, cors_preflight) = {
    let ms = mock_server.lock().unwrap();
    (
      hashmap!{
        "mockServer" => json!({
          "url": ms.url(),
          "port": ms.port
        })
      },
      ms.config.cors_preflight
    )
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
        OptionalBody::Present(ref s, _, _) => Body::from(s.clone()),
        _ => Body::empty()
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
          .body(Body::empty())
          .map_err(|_| InteractionError::ResponseBodyError)
      } else {
        Response::builder()
          .status(500)
          .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
          .header(hyper::header::CONTENT_TYPE, "application/json; charset=utf-8")
          .header("X-Pact", match_result.match_key())
          .body(Body::from(error_body(&request, &match_result.match_key())))
          .map_err(|_| InteractionError::ResponseBodyError)
      }
    }
  }
}

async fn handle_request(
  req: hyper::Request<Body>,
  pact: Arc<dyn Pact + Send + Sync>,
  matches: Arc<Mutex<Vec<MatchResult>>>,
  mock_server: Arc<Mutex<MockServer>>
) -> Result<Response<Body>, InteractionError> {
  debug!("Creating pact request from hyper request");

  {
    let mut guard = mock_server.lock().unwrap();
    let mock_server = guard.borrow_mut();
    mock_server.metrics.requests = mock_server.metrics.requests + 1;
    mock_server.metrics.requests_by_path.entry(req.uri().path().to_string())
      .and_modify(|e| *e += 1)
      .or_insert(1);
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

  // This unwrap is safe, as all pact models can be upgraded to V4 format
  let v4_pact = pact.as_v4_pact().unwrap();
  let match_result = match_request(&pact_request, &v4_pact).await;

  matches.lock().unwrap().push(match_result.clone());

  match_result_to_hyper_response(&pact_request, match_result, mock_server).await
}

// TODO: Should instead use some form of X-Pact headers
fn handle_mock_request_error(result: Result<Response<Body>, InteractionError>) -> Result<Response<Body>, Error> {
    match result {
        Ok(response) => Ok(response),
        Err(error) => {
            let response = match error {
                InteractionError::RequestHeaderEncodingError => Response::builder()
                    .status(400)
                    .body(Body::from("Found an invalid header encoding")),
                InteractionError::RequestBodyError => Response::builder()
                    .status(500)
                    .body(Body::from("Could not process request body")),
                InteractionError::ResponseBodyError => Response::builder()
                    .status(500)
                    .body(Body::from("Could not process response body")),
                InteractionError::ResponseHeaderEncodingError => Response::builder()
                    .status(500)
                    .body(Body::from("Could not set response header"))
            };
            Ok(response.unwrap())
        }
    }
}

// Create and bind the server, but do not start it.
// Returns a future that drives the server.
// The reason that the function itself is still async (even if it performs
// no async operations) is that it needs a tokio context to be able to call try_bind.
pub(crate) async fn create_and_bind(
  pact: Box<dyn Pact + Send + Sync>,
  addr: SocketAddr,
  shutdown: impl std::future::Future<Output = ()>,
  matches: Arc<Mutex<Vec<MatchResult>>>,
  mock_server: Arc<Mutex<MockServer>>,
  mock_server_id: &String
) -> Result<(impl std::future::Future<Output = ()>, SocketAddr), hyper::Error> {
  let pact = pact.arced();
  let ms_id = Arc::new(mock_server_id.clone());

  let server = Server::try_bind(&addr)?
    .serve(make_service_fn(move |_| {
      let pact = pact.clone();
      let matches = matches.clone();
      let mock_server = mock_server.clone();
      let mock_server_id = ms_id.clone();

      LOG_ID.scope(mock_server_id.to_string(), async move {
        Ok::<_, hyper::Error>(
          service_fn(move |req| {
            let pact = pact.clone();
            let matches = matches.clone();
            let mock_server = mock_server.clone();
            let mock_server_id = mock_server_id.clone();

            LOG_ID.scope(mock_server_id.to_string(), async move {
              handle_mock_request_error(
                handle_request(req, pact, matches, mock_server).await
              )
            })
          })
        )
      })
    }));

  let socket_addr = server.local_addr();

  Ok((
      // This is the future that drives the server:
      async {
          let _ = server
              .with_graceful_shutdown(shutdown)
              .await;
      },
      socket_addr
  ))
}

// Taken from https://github.com/ctz/hyper-rustls/blob/master/examples/server.rs
#[cfg(feature = "tls")]
struct HyperAcceptor {
  stream: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + Send>>
}

#[cfg(feature = "tls")]
impl hyper::server::accept::Accept for HyperAcceptor {
  type Conn = TlsStream<TcpStream>;
  type Error = io::Error;

  fn poll_accept(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
  ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
    self.as_mut().stream.poll_next_unpin(cx)
  }
}

#[cfg(feature = "tls")]
pub(crate) async fn create_and_bind_tls(
  pact: Box<dyn Pact + Send + Sync>,
  addr: SocketAddr,
  shutdown: impl std::future::Future<Output = ()>,
  matches: Arc<Mutex<Vec<MatchResult>>>,
  tls_cfg: ServerConfig,
  mock_server: Arc<Mutex<MockServer>>
) -> Result<(impl std::future::Future<Output = ()>, SocketAddr), io::Error> {
  let tcp = TcpListener::bind(&addr).await?;
  let socket_addr = tcp.local_addr()?;
  let tls_acceptor = Arc::new(TlsAcceptor::from(Arc::new(tls_cfg)));
  let tls_stream = stream::unfold((Arc::new(tcp), tls_acceptor.clone()), |(listener, acceptor)| {
    async move {
      let (socket, _) = listener.accept().await.map_err(|err| {
        error!("Failed to accept TLS connection - {:?}", err);
        err
      }).ok()?;
      let stream = acceptor.accept(socket);
      Some((stream.await, (listener.clone(), acceptor.clone())))
    }
  });

  let pact = pact.arced();
  let server = Server::builder(HyperAcceptor {
    stream: tls_stream.boxed()
  })
    .serve(make_service_fn(move |_| {
      let pact = pact.clone();
      let matches = matches.clone();
      let mock_server = mock_server.clone();

      async {
        Ok::<_, hyper::Error>(
          service_fn(move |req| {
            let pact = pact.clone();
            let matches = matches.clone();
            let mock_server = mock_server.clone();

            async {
              handle_mock_request_error(
                handle_request(req, pact, matches, mock_server).await
              )
            }
          })
        )
      }
    }));

  Ok((
    // This is the future that drives the server:
    async {
      let _ = server
        .with_graceful_shutdown(shutdown)
        .await;
    },
    socket_addr
  ))
}

#[cfg(test)]
mod tests {
  use expectest::expect;
  use expectest::prelude::*;
  use hyper::header::{ACCEPT, CONTENT_TYPE, USER_AGENT};
  use hyper::HeaderMap;
  use pact_models::prelude::RequestResponsePact;

  use super::*;

  #[tokio::test]
  async fn can_fetch_results_on_current_thread() {
    let (shutdown_tx, shutdown_rx) = futures::channel::oneshot::channel();
    let matches = Arc::new(Mutex::new(vec![]));

    let (future, _) = create_and_bind(
      RequestResponsePact::default().boxed(),
      ([0, 0, 0, 0], 0 as u16).into(),
      async {
          shutdown_rx.await.ok();
      },
      matches.clone(),
      Arc::new(Mutex::new(MockServer::default())),
      &String::default()
    ).await.unwrap();

    let join_handle = tokio::task::spawn(future);

    shutdown_tx.send(()).unwrap();

    // Server has shut down, now flush the server future from runtime
    join_handle.await.unwrap();

    // 0 matches have been produced
    let all_matches = matches.lock().unwrap().clone();
    assert_eq!(all_matches, vec![]);
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
