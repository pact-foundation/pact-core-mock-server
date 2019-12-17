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
    fn start_mock_server(&self) -> BackgroundMockServer;

    /// Asynchronously spawn a mock server onto the current tokio runtime.
    /// Note that the returned server should be shut down after use by calling server.shutdown().await.
    fn spawn_mock_server(&self) -> BoxFuture<'static, SpawnedMockServer>;
}

impl StartMockServer for Pact {
    fn start_mock_server(&self) -> BackgroundMockServer {
        BackgroundMockServer::new(self.clone())
    }

    fn spawn_mock_server(&self) -> BoxFuture<'static, SpawnedMockServer> {
        SpawnedMockServer::new(self.clone()).boxed()
    }
}

///
/// A trait for accessing the properties of a mock server.
///
pub trait ValidatingMockServer {
    /// The URL of our mock server. You can make normal HTTP requests using this
    /// as the base URL.
    fn url(&self) -> &Url;

    /// Given a path string, return a URL pointing to that path on the mock
    /// server. If the `path` cannot be parsed as URL, **this function will
    /// panic**. For a non-panicking version, call `.url()` instead and build
    /// this path yourself.
    fn path<P: AsRef<str>>(&self, path: P) -> Url;

    /// Returns the current status of the mock server
    fn status(&self) -> Vec<MatchResult>;
}

/// Inner struct for shared data and behaviour of mock servers
struct InnerServer {
    // A description of our mock server, for use in error messages.
    description: String,
    // The URL of our mock server.
    url: Url,
    // The mock server instance
    mock_server: mock_server::MockServer,
    // The running server's asynchronous join handle
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl InnerServer {
    fn new(
        mock_server: mock_server::MockServer,
        server_handle: tokio::task::JoinHandle<()>
    ) -> InnerServer {
        let description = format!("{}/{}", mock_server.pact.consumer.name, mock_server.pact.provider.name);
        let url_str = mock_server.url();
        InnerServer {
            description,
            url: url_str.parse().expect("invalid mock server URL"),
            mock_server,
            server_handle: Some(server_handle)
        }
    }

    /// Initialize the inner mock server instance
    async fn init_mock_server(
        pact: Pact
    ) -> Result<(mock_server::MockServer, impl std::future::Future<Output = ()>), String> {
        mock_server::MockServer::new(
            "".into(),
            pact,
            ([0, 0, 0, 0], 0 as u16).into()
        ).await
    }

    /// Helper function called by our `drop` implementation. This basically exists
    /// so that it can return `Err(message)` whenever needed without making the
    /// flow control in `drop` ultra-complex.
    fn drop_helper(&mut self) -> Result<(), String> {
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

///
/// A mock server running synchronously, in a background thread.
/// Convenient when used with synchronous code.
///
pub struct BackgroundMockServer {
    inner: InnerServer,
    runtime: tokio::runtime::Runtime
}

impl BackgroundMockServer {
    /// Create a new mock server which handles requests as described in the
    /// pact, and runs in a background thread
    pub fn new(pact: Pact) -> BackgroundMockServer {
        let mut runtime = tokio::runtime::Builder::new()
            .threaded_scheduler()
            .num_threads(1)
            .enable_all()
            .build()
            .unwrap();

        let (mock_server, future) = runtime.block_on(InnerServer::init_mock_server(pact))
            .expect("error initializing mock server");

        BackgroundMockServer {
            inner: InnerServer::new(mock_server, runtime.spawn(future)),
            runtime,
        }
    }

    fn drop_helper(&mut self) -> Result<(), String> {
        let server_handle = self.inner.server_handle
            .take()
            .expect("Server was already shut down");
        self.inner.mock_server.shutdown().ok();

        self.runtime.block_on(server_handle).unwrap();

        self.inner.drop_helper()
    }
}

impl ValidatingMockServer for BackgroundMockServer {
    fn url(&self) -> &Url {
        &self.inner.url
    }

    fn path<P: AsRef<str>>(&self, path: P) -> Url {
        // We panic here because this a _test_ library, the `?` operator is
        // useless in tests, and filling up our test code with piles of `unwrap`
        // calls is ugly.
        self.inner.url.join(path.as_ref()).expect("could not parse URL")
    }

    fn status(&self) -> Vec<MatchResult> {
        self.inner.mock_server.mismatches()
    }
}

impl Drop for BackgroundMockServer {
    fn drop(&mut self) {
        let result = self.drop_helper();
        if let Err(msg) = result {
            panic_or_print_error(&msg);
        }
    }
}

///
/// A mock server implementation that is spawned onto the implicit current
/// tokio runtime that drives the current unit test via tokio::test.
/// After use, the server must be explitcly and asynchronously disposed by a call to
/// shutdown(). If not, the test runtime might end up deadlocked.
///
#[must_use = "Remember to call shutdown().await"]
pub struct SpawnedMockServer {
    inner: InnerServer
}

impl SpawnedMockServer {
    /// Asynchronously create and spawn a new mock server
    pub async fn new(pact: Pact) -> SpawnedMockServer {
        let (mock_server, future) = InnerServer::init_mock_server(pact)
            .await
            .expect("error initializing mock server");

        SpawnedMockServer {
            inner: InnerServer::new(mock_server, tokio::spawn(future))
        }
    }

    /// Asynchronously shut down the server.
    /// This is necessary before the test runtime gets destroyed.
    /// The reason is that the mock server is spawned as a background task.
    /// When the tokio test runtime tries to shut down, it will wait for
    /// all its running tasks to complete. Only by first awaiting the server's
    /// JoinHandle can the shutdown procedure be properly executed.
    pub async fn shutdown(mut self) {
        // Send the server shutdown signal
        self.inner.mock_server.shutdown().unwrap();

        let server_handle = self.inner.server_handle
            .take()
            .expect("Server was already shut down");

        // Await the running server's task's join handle
        server_handle.await.unwrap();
    }

    fn drop_helper(&mut self) -> Result<(), String> {
        if self.inner.server_handle.is_some() {
            panic!("Running in spawned mode, the server should be shut down using shutdown().await");
        }
        self.inner.drop_helper()
    }
}

impl ValidatingMockServer for SpawnedMockServer {
    fn url(&self) -> &Url {
        &self.inner.url
    }

    fn path<P: AsRef<str>>(&self, path: P) -> Url {
        // We panic here because this a _test_ library, the `?` operator is
        // useless in tests, and filling up our test code with piles of `unwrap`
        // calls is ugly.
        self.inner.url.join(path.as_ref()).expect("could not parse URL")
    }

    fn status(&self) -> Vec<MatchResult> {
        self.inner.mock_server.mismatches()
    }
}

impl Drop for SpawnedMockServer {
    fn drop(&mut self) {
        let result = self.drop_helper();
        if let Err(msg) = result {
            panic_or_print_error(&msg);
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
