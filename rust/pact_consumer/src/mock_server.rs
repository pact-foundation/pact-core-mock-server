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
use futures::future::*;

/// This trait is implemented by types which allow us to start a mock server.
pub trait StartMockServer {
    /// Start a mock server running in a background thread.
    fn start_mock_server(&self) -> ValidatingMockServer;

    /// Asynchronously spawn a mock server onto the current tokio runtime.
    fn spawn_mock_server(&self) -> BoxFuture<'static, ValidatingMockServer>;
}

impl StartMockServer for Pact {
    fn start_mock_server(&self) -> ValidatingMockServer {
        ValidatingMockServer::start_on_background_runtime(self.clone())
    }

    fn spawn_mock_server(&self) -> BoxFuture<'static, ValidatingMockServer> {
        ValidatingMockServer::spawn_on_current_runtime(self.clone()).boxed()
    }
}

enum Runtime {
    Background(tokio::runtime::Runtime),
    Current
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
    // The running server's join handle
    server_handle: Option<tokio::task::JoinHandle<()>>,
    // The runtime configuration of our mock server.
    runtime: Runtime,
}

impl ValidatingMockServer {
    /// Create a new mock server which handles requests as described in the
    /// pact, and runs in a background thread
    pub fn start_on_background_runtime(pact: Pact) -> ValidatingMockServer {
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .num_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let (mock_server, future) = runtime.block_on(ValidatingMockServer::init_mock_server(pact))
            .expect("error initializing mock server");

        let server_handle = runtime.spawn(future);

        ValidatingMockServer::with_mock_server_and_runtime(
            mock_server,
            server_handle,
            Runtime::Background(runtime)
        )
    }

    /// Asynchronously create and spawn a new mock server
    pub async fn spawn_on_current_runtime(pact: Pact) -> ValidatingMockServer {
        let (mock_server, future) = ValidatingMockServer::init_mock_server(pact)
            .await
            .expect("error initializing mock server");

        let server_handle = tokio::spawn(future);

        ValidatingMockServer::with_mock_server_and_runtime(
            mock_server,
            server_handle,
            Runtime::Current
        )
    }

    // Initialize this struct
    fn with_mock_server_and_runtime(
        mock_server: mock_server::MockServer,
        server_handle: tokio::task::JoinHandle<()>,
        runtime: Runtime
    ) -> ValidatingMockServer {
        let description = format!("{}/{}", mock_server.pact.consumer.name, mock_server.pact.provider.name);
        let url_str = mock_server.url();
        ValidatingMockServer {
            description,
            url: url_str.parse().expect("invalid mock server URL"),
            mock_server,
            server_handle: Some(server_handle),
            runtime: runtime
        }
    }

    // Initialize the inner mock server instance
    async fn init_mock_server(
        pact: Pact
    ) -> Result<(mock_server::MockServer, impl std::future::Future<Output = ()>), String> {
        mock_server::MockServer::new(
            "".into(),
            pact,
            ([0, 0, 0, 0], 0 as u16).into()
        ).await
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

    /// Returns the current status of the mock server
    pub fn status(&self) -> Vec<MatchResult> {
      self.mock_server.mismatches()
    }

    /// Asynchronously shut down the server. Only needed for "spawned" mode.
    pub async fn shutdown(mut self) {
        self.mock_server.shutdown().unwrap();
        if let Some(server_handle) = self.server_handle.take() {
            server_handle.await.unwrap();
        }
    }

    /// Helper function called by our `drop` implementation. This basically exists
    /// so that it can return `Err(message)` whenever needed without making the
    /// flow control in `drop` ultra-complex.
    fn drop_helper(&mut self) -> Result<(), String> {
        // Kill the server
        match &mut self.runtime {
            Runtime::Background(runtime) => {
                let server_handle = self.server_handle
                    .take()
                    .expect("Server was already shut down");
                self.mock_server.shutdown().ok();

                runtime.block_on(server_handle).unwrap();
            },
            Runtime::Current => {
                if self.server_handle.is_some() {
                    panic!("Running in spawned mode, the server should be shut down using shutdown().await");
                }
            }
        }

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
