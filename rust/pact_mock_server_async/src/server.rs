use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use pact_matching::models::parse_query_string;

use std::collections::{BTreeMap, HashMap};
use log::{log, debug, warn};
use hyper::{Body, Response, Server, Error};
use hyper::header::ToStrError;
use hyper::service::service_fn_ok;
use futures::future;
use futures::future::Future;
use futures::stream::Stream;

enum MockRequestError {
    InvalidHeaderEncoding,
    RequestBodyError
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

fn extract_headers(headers: &hyper::HeaderMap) -> Result<Option<HashMap<String, String>>, MockRequestError> {
    if headers.len() > 0 {
        let result: Result<HashMap<String, String>, MockRequestError> = headers.keys()
            .map(|name| -> Result<(String, String), MockRequestError> {
                let values = headers.get_all(name);
                let mut iter = values.iter();

                let first_value = iter.next().unwrap();

                if iter.next().is_some() {
                    warn!("Multiple headers associated with '{}', but only the first is used", name);
                }

                Ok((
                    name.as_str().into(),
                    first_value.to_str()
                        .map_err(|err| MockRequestError::InvalidHeaderEncoding)?
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

pub fn extract_body(chunk: hyper::Chunk) -> OptionalBody {
    let bytes = chunk.into_bytes();
    if bytes.len() > 0 {
        OptionalBody::Present(bytes.to_vec())
    } else {
        OptionalBody::Empty
    }
}

fn hyper_request_to_pact_request(req: hyper::Request<Body>) -> impl Future<Item = Request, Error = MockRequestError> {
    let method = req.method().to_string();
    let path = extract_path(req.uri());
    let query = extract_query_string(req.uri());
    let headers = extract_headers(req.headers());

    future::done(headers)
        .and_then(move |headers| {
            req.into_body()
                .concat2()
                .map_err(|_| MockRequestError::RequestBodyError)
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

pub fn start(
    id: String,
    pact: Pact,
    port: u16,
    shutdown: impl Future<Item = (), Error = ()>,
) -> (impl Future<Item = (), Error = Error>, u16) {
    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr)
        .serve(|| {
            service_fn_ok(|req| {
                debug!("Creating pact request from hyper request");
                let req = hyper_request_to_pact_request(req);
                Response::new(Body::from("Hello World"))
            })
        });

    let port = server.local_addr().port();

    (server.with_graceful_shutdown(shutdown), port)
}
