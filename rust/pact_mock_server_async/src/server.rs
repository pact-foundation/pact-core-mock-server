use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};

use hyper::{Body, Response, Server, Error};
use hyper::service::service_fn_ok;
use futures::future::Future;

pub fn start(
    id: String,
    pact: Pact,
    port: u16,
    shutdown: impl Future<Item = (), Error = ()>,
) -> (impl Future<Item = (), Error = Error>, u16) {
    let addr = ([0, 0, 0, 0], port).into();
    let server = Server::bind(&addr)
        .serve(|| {
            service_fn_ok(|_req| {
                Response::new(Body::from("Hello World"))
            })
        });

    let port = server.local_addr().port();

    (server.with_graceful_shutdown(shutdown), port)
}
