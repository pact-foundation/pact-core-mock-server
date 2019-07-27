use matching::{MatchResult, match_request};

use pact_matching::models::{Pact, Request, OptionalBody};
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use pact_matching::models::parse_query_string;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::{log, error, warn, info, debug};
use hyper::{Body, Response, Server, Error};
use hyper::http::response::{Builder as ResponseBuilder};
use hyper::http::header::{HeaderName, HeaderValue};
use hyper::service::service_fn;
use futures::future;
use futures::future::Future;
use futures::stream::Stream;
use serde_json::json;

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

fn extract_query_string(uri: &hyper::Uri) -> Option<HashMap<String, Vec<String>>> {
    uri.path_and_query()
        .and_then(|path_and_query| path_and_query.query())
        .and_then(|query| parse_query_string(&query.into()))
}

fn extract_headers(headers: &hyper::HeaderMap) -> Result<Option<HashMap<String, String>>, InteractionError> {
    if headers.len() > 0 {
        let result: Result<HashMap<String, String>, InteractionError> = headers.keys()
            .map(|name| -> Result<(String, String), InteractionError> {
                let values = headers.get_all(name);
                let mut iter = values.iter();

                let first_value = iter.next().unwrap();

                if iter.next().is_some() {
                    warn!("Multiple headers associated with '{}', but only the first is used", name);
                }

                Ok((
                    name.as_str().into(),
                    first_value.to_str()
                        .map_err(|_| InteractionError::RequestHeaderEncodingError)?
                        .into()
                    )
                )
            })
            .collect();

        result.map(|map| Some(map))
    } else {
        Ok(None)
    }
}

fn extract_body(chunk: hyper::Chunk) -> OptionalBody {
    let bytes = chunk.into_bytes();
    if bytes.len() > 0 {
        OptionalBody::Present(bytes.to_vec())
    } else {
        OptionalBody::Empty
    }
}

fn hyper_request_to_pact_request(req: hyper::Request<Body>) -> impl Future<Item = Request, Error = InteractionError> {
    let method = req.method().to_string();
    let path = extract_path(req.uri());
    let query = extract_query_string(req.uri());
    let headers = extract_headers(req.headers());

    future::done(headers)
        .and_then(move |headers| {
            req.into_body()
                .concat2()
                .map_err(|_| InteractionError::RequestBodyError)
                .map(|body_chunk| (headers, body_chunk))
        })
        .and_then(|(headers, body_chunk)|
            Ok(Request {
                method: method,
                path: path,
                query: query,
                headers: headers,
                body: extract_body(body_chunk),
                matching_rules: MatchingRules::default(),
                generators: Generators::default()
            })
        )
}

fn set_hyper_headers(builder: &mut ResponseBuilder, headers: &Option<HashMap<String, String>>) -> Result<(), InteractionError> {
    let hyper_headers = builder.headers_mut().unwrap();
    match headers {
        Some(header_map) => {
            for (k, v) in header_map {
                // FIXME?: Headers are not sent in "raw" mode.
                // Names are converted to lower case and values are parsed.
                hyper_headers.insert(
                    HeaderName::from_bytes(k.as_bytes())
                        .map_err(|err| {
                            error!("Invalid header name '{}' ({})", k, err);
                            InteractionError::ResponseHeaderEncodingError
                        })?,
                    v.parse::<HeaderValue>()
                        .map_err(|err| {
                            error!("Invalid header value '{}': '{}' ({})", k, v, err);
                            InteractionError::ResponseHeaderEncodingError
                        })?
                );
            }
        },
        _ => {}
    }
    Ok(())
}

fn error_body(request: &Request, error: &String) -> String {
    let body = json!({ "error" : format!("{} : {:?}", error, request) });
    body.to_string()
}

fn match_result_to_hyper_response(request: &Request, match_result: MatchResult) -> Result<Response<Body>, InteractionError> {
    match match_result {
        MatchResult::RequestMatch(ref interaction) => {
            let response = pact_matching::generate_response(&interaction.response);
            info!("Request matched, sending response {:?}", response);
            info!("     body: '{}'\n\n", interaction.response.body.str_value());
            info!("     body: '{}'\n\n", interaction.response.body.str_value());

            let mut builder = Response::builder();

            builder.status(response.status);
            builder.header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*");
            set_hyper_headers(&mut builder, &response.headers)?;

            builder.body(match response.body {
                OptionalBody::Present(ref s) => Body::from(s.clone()),
                _ => Body::empty()
            })
                .map_err(|_| InteractionError::ResponseBodyError)
        },
        _ => {
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

fn handle_request(
    req: hyper::Request<Body>,
    pact: Arc<Pact>,
    matches: Arc<Mutex<Vec<MatchResult>>>
) -> impl Future<Item = Response<Body>, Error = InteractionError> {
    debug!("Creating pact request from hyper request");

    hyper_request_to_pact_request(req)
        .and_then(move |request| {
            info!("Received request {:?}", request);
            let match_result = match_request(&request, &pact.interactions);

            matches.lock().unwrap().push(match_result.clone());

            match_result_to_hyper_response(&request, match_result)
        })
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

pub fn create_and_bind(
    pact: Pact,
    port: u16,
    shutdown: impl Future<Item = (), Error = ()>,
    matches: Arc<Mutex<Vec<MatchResult>>>,
) -> Result<(impl Future<Item = (), Error = ()>, std::net::SocketAddr), hyper::Error> {
    let pact = Arc::new(pact);
    let addr = ([0, 0, 0, 0], port).into();

    let server = Server::try_bind(&addr)?
        .serve(move || {
            let pact = pact.clone();
            let matches = matches.clone();

            service_fn(move |req| {
                handle_request(req, pact.clone(), matches.clone())
                    .then(handle_mock_request_error)
            })
        });

    let socket_addr = server.local_addr();

    let prepared_server = server
        .with_graceful_shutdown(shutdown)
        .map_err(move |err| {
            eprintln!("server error: {}", err);
        });

    Ok((prepared_server, socket_addr))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::current_thread::Runtime;

    #[test]
    fn can_fetch_results_on_current_thread() {
        let mut runtime = Runtime::new().unwrap();

        let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel();
        let matches = Arc::new(Mutex::new(vec![]));

        let (future, _) = create_and_bind(Pact::default(), 0, shutdown_rx.map_err(|_| ()), matches.clone()).unwrap();

        runtime.spawn(future);
        shutdown_tx.send(()).unwrap();

        // Server has shut down, now flush the server future from runtime
        runtime.run().unwrap();

        // 0 matches have been produced
        let all_matches = matches.lock().unwrap().clone();
        assert_eq!(all_matches, vec![]);
    }
}
