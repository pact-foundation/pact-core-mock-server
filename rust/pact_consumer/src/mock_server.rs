//! Support for mock HTTP servers that verify pacts.

use pact_matching::models::*;
use pact_mock_server::*;
use pact_mock_server::matching::MatchResult;
use std::{
  fmt::Write as FmtWrite,
  env,
  io::{self, prelude::*},
  thread
};
use url::Url;
use tokio;

/// This trait is implemented by types which allow us to start a mock server.
pub trait StartMockServer {
    /// Start a mock server running in a background thread.
    fn start_mock_server(&self) -> ValidatingMockServer;
    /// Start a new async mock server using a custom function for spawning its future
    fn start_mock_server_async<S>(&self, spawner: S) -> ValidatingMockServer
        where S: FnOnce(Box<dyn futures::Future<Item = (), Error = ()> + 'static + Send>);
}

impl StartMockServer for Pact {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::with_background_runtime(self.clone())
    }
    fn start_mock_server_async<S>(&self, spawner: S) -> ValidatingMockServer
        where S: FnOnce(Box<dyn futures::Future<Item = (), Error = ()> + 'static + Send>) {
        ValidatingMockServer::with_spawner(self.clone(), spawner)
    }
}

enum Mode {
    Background(tokio::runtime::Runtime),
    Async
}

/// A mock HTTP server that handles the requests described in a `Pact`, intended
/// for use in tests, and validates that the requests made to that server are
/// correct.
///
/// Because this is intended for use in tests, it will panic if something goes
/// wrong.
pub struct ValidatingMockServer {
    // A description of our mock server, for use in error messages.
    description: String,
    // The URL of our mock server.
    url: Url,
    // The mock server instance
    mock_server: mock_server::MockServer,
    // The running mode of our mock server.
    _mode: Mode
}

impl ValidatingMockServer {
    /// Create a new mock server with a function that spawns a future
    pub fn with_spawner<S>(pact: Pact, spawner: S) -> ValidatingMockServer
        where S: FnOnce(Box<dyn futures::Future<Item = (), Error = ()> + 'static + Send>)
    {
        ValidatingMockServer::with_mode_and_spawner(pact, Mode::Async, spawner)
    }

    /// Create a new mock server which handles requests as described in the
    /// pact, and runs in a background thread
    pub fn with_background_runtime(pact: Pact) -> ValidatingMockServer {
        let runtime = tokio::runtime::Builder::new()
            .core_threads(1)
            .blocking_threads(1)
            .build()
            .unwrap();

        let executor = runtime.executor();

        ValidatingMockServer::with_mode_and_spawner(pact, Mode::Background(runtime), |future| {
            executor.spawn(future)
        })
    }

    fn with_mode_and_spawner<S>(pact: Pact, mode: Mode, spawner: S) -> ValidatingMockServer
        where S: FnOnce(Box<dyn futures::Future<Item = (), Error = ()> + 'static + Send>)
    {
        let (mock_server, future) = mock_server::MockServer::new("".into(), pact, 0)
            .expect("error starting mock server");

        spawner(Box::new(future));

        let description = format!("{}/{}", mock_server.pact.consumer.name, mock_server.pact.provider.name);
        let url_str = mock_server.url();
        ValidatingMockServer {
            description,
            url: url_str.parse().expect("invalid mock server URL"),
            mock_server: mock_server,
            _mode: mode
        }
    }

    /// The URL of our mock server. You can make normal HTTP requests using this
    /// as the base URL.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Given a path string, return a URL pointing to that path on the mock
    /// server. If the `path` cannot be parsed as URL, **this function will
    /// panic**. For a non-panicking version, call `.url()` instead and build
    /// this path yourself.
    pub fn path<P: AsRef<str>>(&self, path: P) -> Url {
        // We panic here because this a _test_ library, the `?` operator is
        // useless in tests, and filling up our test code with piles of `unwrap`
        // calls is ugly.
        self.url.join(path.as_ref()).expect("could not parse URL")
    }

    /// Helper function called by our `drop` implementation. This basically exists
    /// so that it can return `Err(message)` whenever needed without making the
    /// flow control in `drop` ultra-complex.
    fn drop_helper(&mut self) -> Result<(), String> {
        // Kill the server
        self.mock_server.shutdown()?;

        // Read in all match results
        self.mock_server.read_match_results_from_server();

        // Look up any mismatches which occurred.
        let mismatches = self.mock_server.mismatches();

        if mismatches.is_empty() {
            // Success! Write out the generated pact file.
            self.mock_server.write_pact(&Some(env::var("PACT_OUTPUT_DIR").unwrap_or("target/pacts".to_owned())))
                .map_err(|err| format!("error writing pact: {}", err))?;
            Ok(())
        } else {
            // Failure. Format our errors.
            let mut msg = format!(
                "mock server {} failed verification:\n",
                self.description,
            );
            for mismatch in mismatches {
                match mismatch {
                    MatchResult::RequestMatch(_) => {
                        unreachable!("list of mismatches contains a match");
                    }
                    MatchResult::RequestMismatch(interaction, mismatches) => {
                        let _ = writeln!(
                            &mut msg,
                            "- interaction {:?}:",
                            interaction.description,
                        );
                        for m in mismatches {
                            let _ = writeln!(&mut msg, "  - {}", m.description());
                        }
                    }
                    MatchResult::RequestNotFound(request) => {
                        let _ = writeln!(&mut msg, "- received unexpected request:");
                        let _ = writeln!(&mut msg, "{:#?}", request);
                    }
                    MatchResult::MissingRequest(interaction) => {
                        let _ = writeln!(
                            &mut msg,
                            "- interaction {:?} expected, but never occurred",
                            interaction.description,
                        );
                        let _ = writeln!(&mut msg, "{:#?}", interaction.request);
                    }
                }
            }
            Err(msg)
        }
    }
}

/// Either panic with `msg`, or if we're already in the middle of a panic,
/// just print `msg` to standard error.
fn panic_or_print_error(msg: &str) {
    if thread::panicking() {
        // The current thread is panicking, so don't try to panic again, because
        // double panics don't print useful explanations of why the test failed.
        // Instead, just print to `stderr`. Ignore any errors, because there's
        // not much we can do if we can't panic and we can't write to `stderr`.
        let _ = writeln!(io::stderr(), "{}", msg);
    } else {
        panic!("{}", msg);
    }
}

impl Drop for ValidatingMockServer {
    fn drop(&mut self) {
        let result = self.drop_helper();
        if let Err(msg) = result {
            panic_or_print_error(&msg);
        }
    }
}
