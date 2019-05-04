use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};
use pact_matching::models::matchingrules::*;
use pact_matching::models::generators::*;
use pact_matching::models::parse_query_string;

use std::collections::{BTreeMap, HashMap};
use log::{log, debug, warn};
use hyper::{Body, Response, Server, Error};
use hyper::service::service_fn_ok;
use futures::future::Future;
use futures::stream::Stream;

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

fn extract_headers(headers: &hyper::HeaderMap) -> Option<HashMap<String, String>> {
    if headers.len() > 0 {
        let v = headers.iter().map(|(name, value)| -> Result<(String, String), String> {
            Ok((name.as_str().into(), value.to_string()?))
        });
        None
    } else {
        None
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

fn hyper_request_to_pact_request(req: hyper::Request<Body>) -> impl Future<Item = Request, Error = hyper::Error> {
    let method = req.method().to_string();
    let path = extract_path(req.uri());
    let query = extract_query_string(req.uri());
    let headers = extract_headers(req.headers());

    req.into_body()
        .concat2()
        .and_then(|body_chunk| Ok(Request {
            method: req.method().to_string(),
            path: path,
            query: query,
            headers: headers,
            body: extract_body(body_chunk),
            matching_rules: MatchingRules::default(),
            generators: Generators::default()
        }))
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
