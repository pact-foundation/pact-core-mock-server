//! The `pact_consumer` crate provides tools for writing consumer [Pact
//! tests][pact]. It implements the [V3 Pact specification][spec]. You can also
//! use it as a simple HTTP mocking library for Rust.
//!
//! [pact]: https://docs.pact.io/ [spec]:
//! https://github.com/pact-foundation/pact-specification
//!
//! ## What is Pact?
//!
//! [Pact][pact] is a [cross-language standard][spec] for testing the
//! communication between the consumer of a REST API, and the code that provides
//! that API. Test cases are written from the consumer's perspective, and they
//! can then be exported testing the provider.
//!
//! The big advantages of Pact are:
//!
//! 1. The mocks you write to test the client can also be reused to verify that
//!    the server would actually respond the way the client expects. This gives
//!    the end-to-end assurance of integration tests (well, almost), but with
//!    the speed and convenience of unit tests.
//! 2. Pact has been implemented in many popular languages, so you can test
//!    clients and servers in multiple languages.
//!
//! Whenever possible, we try to use vocabulary similar to the Ruby or
//! JavaScript API for basic concepts, and we try to provide the same behavior.
//! But we offer many handy builder methods to make tests cleaner.
//!
//! ## How to use it
//!
//! To use this crate, add it to your `[dev-dependencies]` in your `Cargo.toml`:
//!
//! ```toml
//! [dev-dependencies]
//! pact_consumer = "0.7"
//! ```
//!
//! Once this is done, you can then write the following inside a function marked
//! with `#[tokio::test]`:
//!
//! ```
//! # tokio_test::block_on(async {
//! use pact_consumer::prelude::*;
//!
//! // Define the Pact for the test, specify the names of the consuming
//! // application and the provider application.
//! let pact = PactBuilder::new("Consumer", "Alice Service")
//!     // Start a new interaction. We can add as many interactions as we want.
//!     .interaction("a retrieve Mallory request", "", |mut i| async move {
//!         // Defines a provider state. It is optional.
//!         i.given("there is some good mallory");
//!         // Define the request, a GET (default) request to '/mallory'.
//!         i.request.path("/mallory");
//!         // Define the response we want returned. We assume a 200 OK
//!         // response by default.
//!         i.response
//!             .content_type("text/plain")
//!             .body("That is some good Mallory.");
//!         // Return the interaction builder back to the pact framework
//!         i
//!     })
//!     .await
//!     .build();
//! # });
//! ```
//!
//! You can than use an HTTP client like `reqwest` to make requests against your
//! server.
//!
//! ```rust
//! # tokio_test::block_on(async {
//! # use pact_models::pact::Pact;
//! # use std::io::Read;
//! # use pact_consumer::prelude::*;
//! # let alice_service = PactBuilder::new("Consumer", "Alice Service")
//! #     // Start a new interaction. We can add as many interactions as we want.
//! #     .interaction("a retrieve Mallory request", "", |mut i| async move {
//! #         // Defines a provider state. It is optional.
//! #         i.given("there is some good mallory");
//! #         // Define the request, a GET (default) request to '/mallory'.
//! #         i.request.path("/mallory");
//! #         // Define the response we want returned. We assume a 200 OK
//! #         // response by default.
//! #         i.response
//! #             .content_type("text/plain")
//! #             .body("That is some good Mallory.");
//! #         // Return the interaction builder back to the pact framework
//! #         i
//! #     }).await.start_mock_server();
//!
//! // You would use your actual client code here.
//! let mallory_url = alice_service.path("/mallory");
//! let mut response = reqwest::get(mallory_url).await.expect("could not fetch URL")
//!   .text().await.expect("Could not read response body");
//! assert_eq!(response, "That is some good Mallory.");
//!
//! // When `alice_service` goes out of scope, your pact will be validated,
//! // and the test will fail if the mock server didn't receive matching
//! // requests.
//! # });
//! ```
//!
//! ## Matching using patterns
//!
//! You can also use patterns like `like!`, `each_like!` or `term!` to allow
//! more general matches, and you can build complex patterns using the
//! `json_pattern!` macro:
//!
//! ```
//! # tokio_test::block_on(async {
//! use pact_consumer::prelude::*;
//! use pact_consumer::*;
//!
//! PactBuilder::new("quotes client", "quotes service")
//!     .interaction("add a new quote to the database", "", |mut i| async move {
//!         i.request
//!             .post()
//!             .path("/quotes")
//!             .json_utf8()
//!             .json_body(json_pattern!({
//!                  // Allow the client to send any string as a quote.
//!                  // When testing the server, use "Eureka!".
//!                  "quote": like!("Eureka!"),
//!                  // Allow the client to send any string as an author.
//!                  // When testing the server, use "Archimedes".
//!                  "by": like!("Archimedes"),
//!                  // Allow the client to send an array of strings.
//!                  // When testing the server, send a single-item array
//!                  // containing the string "greek".
//!                  "tags": each_like!("greek"),
//!              }));
//!
//!         i.response
//!             .created()
//!             // Return a location of "/quotes/12" to the client. When
//!             // testing the server, allow it to return any numeric ID.
//!             .header("Location", term!("^/quotes/[0-9]+$", "/quotes/12"));
//!         i
//!     });
//! # });
//! ```
//!
//! The key insight here is this "pact" can be used to test both the client and
//! the server:
//!
//! - When testing the **client**, we allow the request to be anything which
//!   matches the patterns—so `"quote"` can be any string, not just `"Eureka!"`.
//!   But we respond with the specified values, such as `"/quotes/12"`.
//! - When testing the **server**, we send the specified values, such as
//!   `"Eureka!"`. But we allow the server to respond with anything matching the
//!   regular expression `^/quotes/[0-9]+$`, because we don't know what database
//!   ID it will use.
//!
//! Also, when testing the server, we may need to set up particular database
//! fixtures. This can be done using the string passed to `given` in the
//! examples above.
//!
//! ## Testing using domain objects
//!
//! Normally, it's best to generate your JSON using your actual domain objects.
//! This is easier, and it reduces duplication in your code.
//!
//! ```
//! use pact_consumer::prelude::*;
//! use pact_consumer::{each_like, each_like_helper, json_pattern};
//! use serde::{Deserialize, Serialize};
//!
//! /// Our application's domain object representing a user.
//! #[derive(Deserialize, Serialize)]
//! struct User {
//!     /// All users have this field.
//!     name: String,
//!
//!     /// The server may omit this field when sending JSON, or it may send it
//!     /// as `null`.
//!     comment: Option<String>,
//! }
//!
//! // Create our example user using our normal application objects.
//! let example = User {
//!     name: "J. Smith".to_owned(),
//!     comment: None,
//! };
//!
//! # tokio_test::block_on(async move {
//! PactBuilder::new("consumer", "provider")
//!     .interaction("get all users", "", |mut i| async move {
//!         i.given("a list of users in the database");
//!         i.request.path("/users");
//!         i.response
//!             .json_utf8()
//!             .json_body(each_like!(
//!                 // Here, `strip_null_fields` will remove `comment` from
//!                 // the generated JSON, allowing our pattern to match
//!                 // missing comments, null comments, and comments with
//!                 // strings.
//!                 strip_null_fields(serde_json::json!(example)),
//!             ));
//!         i
//!     })
//!     .await
//!     .build();
//! # });
//! ```
//!
//! For more advice on writing good pacts, see [Best Practices][].
//!
//! [Best Practices]: https://docs.pact.io/best_practices/consumer.html
#![warn(missing_docs)]

// Child modules which define macros (must be first because macros are resolved)
// in source inclusion order).
#[macro_use]
pub mod patterns;
#[cfg(test)]
#[macro_use]
mod test_support;

// Other child modules.
pub mod builders;
pub mod mock_server;
pub mod util;

/// A "prelude" or a default list of import types to include. This includes
/// the basic DSL, but it avoids including rarely-used types.
///
/// ```
/// use pact_consumer::prelude::*;
/// ```
pub mod prelude {
    pub use crate::builders::{HttpPartBuilder, PactBuilder};
    pub use crate::mock_server::{StartMockServer, ValidatingMockServer};
    pub use crate::patterns::{EachLike, Like, Term};
    pub use crate::patterns::{JsonPattern, Pattern, StringPattern};
    pub use crate::patterns::{DateTime};
    pub use crate::util::strip_null_fields;
}

/// Consumer version
pub const PACT_CONSUMER_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
