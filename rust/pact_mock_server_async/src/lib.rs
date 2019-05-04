extern crate pact_matching;
extern crate hyper;
extern crate futures;
extern crate tokio;
extern crate log;

mod server;

pub fn run_server_test() {
    let pact = pact_matching::models::Pact::default();

    let f = server::start("yo".into(), pact, 0, futures::future::done(Ok(())));
}