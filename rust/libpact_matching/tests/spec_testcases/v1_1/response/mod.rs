#[allow(unused_imports)]
use env_logger;
#[allow(unused_imports)]
use pact_matching::models::PactSpecification;
#[allow(unused_imports)]
use pact_matching::models::Response;
#[allow(unused_imports)]
use pact_matching::match_response;
#[allow(unused_imports)]
use rustc_serialize::json::Json;
#[allow(unused_imports)]
use expectest::prelude::*;
mod body;
mod headers;
mod status;
