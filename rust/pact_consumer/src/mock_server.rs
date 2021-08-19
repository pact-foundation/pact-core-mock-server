//! Support for mock HTTP servers that verify pacts.

use std::{
  env,
  fmt::Write as FmtWrite,
  io::{self, prelude::*},
  thread,
};
use std::sync::{Arc, Mutex};

use log::debug;
use url::Url;

use pact_mock_server::*;
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::{MockServerConfig, MockServerMetrics};
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use pact_plugin_driver::plugin_manager::shutdown_plugins;

/// This trait is implemented by types which allow us to start a mock server.
pub trait StartMockServer {
    /// Start a mock server running in a background thread.
    fn start_mock_server(&self) -> ValidatingMockServer;
}

impl StartMockServer for RequestResponsePact {
  fn start_mock_server(&self) -> ValidatingMockServer {
    ValidatingMockServer::start(self.boxed())
  }
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
    mock_server: Arc<Mutex<mock_server::MockServer>>,
    // Signal received when the server thread is done executing
    done_rx: std::sync::mpsc::Receiver<()>,
}

impl ValidatingMockServer {
  /// Create a new mock server which handles requests as described in the
  /// pact, and runs in a background thread
  pub fn start(pact: Box<dyn Pact + Send>) -> ValidatingMockServer {
    debug!("Starting mock server from pact {:?}", pact);
    // Spawn new runtime in thread to prevent reactor execution context conflict
    let (pact_tx, pact_rx) = std::sync::mpsc::channel::<Box<dyn Pact + Send>>();
    pact_tx.send(pact).expect("INTERNAL ERROR: Could not pass pact into mock server thread");
    let (mock_server, done_rx) = std::thread::spawn(|| {
      let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("new runtime");

      let (mock_server, server_future) = runtime.block_on(async move {
        mock_server::MockServer::new("".into(), pact_rx.recv().unwrap(), ([0, 0, 0, 0], 0 as u16).into(),
          MockServerConfig::default())
          .await
          .unwrap()
      });

      // Start the actual thread the runtime will run on
      let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
      let tname = format!(
        "test({})-pact-mock-server",
        thread::current().name().unwrap_or("<unknown>")
      );
      std::thread::Builder::new()
        .name(tname)
        .spawn(move || {
          runtime.block_on(server_future);
          let _ = done_tx.send(());
        })
        .expect("thread spawn");

      (mock_server, done_rx)
    })
    .join()
    .unwrap();

    let (description, url_str) = {
      let ms = mock_server.lock().unwrap();
      let pact = ms.pact.lock().unwrap();
      let description = format!(
        "{}/{}", pact.consumer().name, pact.provider().name
      );
      (description, ms.url())
    };
    ValidatingMockServer {
      description,
      url: url_str.parse().expect("invalid mock server URL"),
      mock_server,
      done_rx,
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

    /// Returns the current status of the mock server
    pub fn status(&self) -> Vec<MatchResult> {
      self.mock_server.lock().unwrap().mismatches()
    }

    /// Helper function called by our `drop` implementation. This basically exists
    /// so that it can return `Err(message)` whenever needed without making the
    /// flow control in `drop` ultra-complex.
    fn drop_helper(&mut self) -> Result<(), String> {
        // Kill the server
        let mut ms = self.mock_server.lock().unwrap();
        ms.shutdown()?;

        if ::std::thread::panicking() {
            return Ok(());
        }

        // Wait for the server thread to finish
        self.done_rx
            .recv_timeout(std::time::Duration::from_secs(3))
            .expect("mock server thread should not panic");

        shutdown_plugins();

        // Look up any mismatches which occurred.
        let mismatches = ms.mismatches();

        if mismatches.is_empty() {
            // Success! Write out the generated pact file.
            ms.write_pact(
              &Some(
                env::var("PACT_OUTPUT_DIR").unwrap_or_else(|_| "target/pacts".to_owned())),
              env::var("PACT_OVERWRITE").unwrap_or_else(|_| "false".to_owned()) == "true")
            .map_err(|err| format!("error writing pact: {}", err))?;
            Ok(())
        } else {
            // Failure. Format our errors.
            let mut msg = format!("mock server {} failed verification:\n", self.description,);
            for mismatch in mismatches {
                match mismatch {
                    MatchResult::RequestMatch(..) => {
                        unreachable!("list of mismatches contains a match");
                    }
                    MatchResult::RequestMismatch(request, mismatches) => {
                        let _ = writeln!(&mut msg, "- request {}:", request);
                        for m in mismatches {
                            let _ = writeln!(&mut msg, "  - {}", m.description());
                        }
                    }
                    MatchResult::RequestNotFound(request) => {
                        let _ = writeln!(&mut msg, "- received unexpected request:");
                        let _ = writeln!(&mut msg, "{:#?}", request);
                    }
                    MatchResult::MissingRequest(request) => {
                        let _ = writeln!(
                            &mut msg,
                            "- request {} expected, but never occurred", request,
                        );
                        let _ = writeln!(&mut msg, "{:#?}", request);
                    }
                }
            }
            Err(msg)
        }
    }

  /// Returns the metrics collected by the mock server
  pub fn metrics(&self) -> MockServerMetrics {
    self.mock_server.lock().unwrap().metrics.clone()
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
