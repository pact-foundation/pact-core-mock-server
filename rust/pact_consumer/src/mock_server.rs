//! Support for mock HTTP servers that verify pacts.

use std::{
  env,
  fmt::Write as FmtWrite,
  io::{self, prelude::*},
  thread,
};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use log::{debug, warn};
use url::Url;

use pact_mock_server::*;
use pact_mock_server::matching::MatchResult;
use pact_mock_server::mock_server::{MockServerConfig, MockServerMetrics};
use pact_models::pact::Pact;
use pact_models::sync_pact::RequestResponsePact;
use std::path::PathBuf;
use uuid::Uuid;
use pact_matching::metrics::{MetricEvent, send_metrics};

/// This trait is implemented by types which allow us to start a mock server.
#[async_trait]
pub trait StartMockServer {
  /// Start a mock server running in a background thread.
  fn start_mock_server(&self) -> ValidatingMockServer;

  /// Start a mock server running in a task (requires a Tokio runtime to be already setup)
  async fn start_mock_server_async(&self) -> ValidatingMockServer;
}

#[async_trait]
impl StartMockServer for RequestResponsePact {
  fn start_mock_server(&self) -> ValidatingMockServer {
    ValidatingMockServer::start(self.boxed(), None)
  }

  async fn start_mock_server_async(&self) -> ValidatingMockServer {
    ValidatingMockServer::start_async(self.boxed(), None).await
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
    // Output directory to write pact files
    output_dir: Option<PathBuf>,
}

impl ValidatingMockServer {
  /// Create a new mock server which handles requests as described in the
  /// pact, and runs in a background thread
  ///
  /// Panics:
  /// Will panic if the provided Pact can not be sent to the background thread.
  pub fn start(pact: Box<dyn Pact + Send + Sync>, output_dir: Option<PathBuf>) -> ValidatingMockServer {
    debug!("Starting mock server from pact {:?}", pact);
    // Spawn new runtime in thread to prevent reactor execution context conflict
    let (pact_tx, pact_rx) = std::sync::mpsc::channel::<Box<dyn Pact + Send + Sync>>();
    pact_tx.send(pact).expect("INTERNAL ERROR: Could not pass pact into mock server thread");
    let (mock_server, done_rx) = std::thread::spawn(|| {
      let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("new runtime");

      let (mock_server, server_future) = runtime.block_on(async move {
        mock_server::MockServer::new(
          Uuid::new_v4().to_string(),
          pact_rx.recv().unwrap(),
          ([0, 0, 0, 0], 0).into(),
          MockServerConfig::default()
        )
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
      output_dir
    }
  }

  /// Create a new mock server which handles requests as described in the
  /// pact, and runs in a background task in the current Tokio runtime.
  ///
  /// Panics:
  /// Will panic if unable to get the URL to the spawned mock server
  pub async fn start_async(pact: Box<dyn Pact + Send + Sync>, output_dir: Option<PathBuf>) -> ValidatingMockServer {
    debug!("Starting mock server from pact {:?}", pact);

    let (mock_server, server_future) = mock_server::MockServer::new(
      Uuid::new_v4().to_string(),
      pact,
      ([0, 0, 0, 0], 0 as u16).into(),
      MockServerConfig::default()
    )
      .await
      .unwrap();

    let (done_tx, done_rx) = std::sync::mpsc::channel::<()>();
    tokio::spawn(async move {
      dbg!(server_future.await);
      let _ = done_tx.send(());
    });

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
      output_dir
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
        if let Err(_) = self.done_rx.recv_timeout(std::time::Duration::from_secs(3)) {
          warn!("Timed out waiting for mock server to finish");
        }

      let interactions = {
        let pact = ms.pact.lock().unwrap();
        pact.interactions().len()
      };
      send_metrics(MetricEvent::ConsumerTestRun {
        interactions,
        test_framework: "pact_consumer".to_string(),
        app_name: "pact_consumer".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string()
      });

        // Look up any mismatches which occurred.
        let mismatches = ms.mismatches();

        if mismatches.is_empty() {
          // Success! Write out the generated pact file.
          let output_dir = self.output_dir.as_ref().map(|dir| dir.to_string_lossy().to_string())
            .unwrap_or_else(|| {
              let val = env::var("PACT_OUTPUT_DIR");
              debug!("env:PACT_OUTPUT_DIR = {:?}", val);
              val.unwrap_or_else(|_| "target/pacts".to_owned())
            });
          let overwrite = env::var("PACT_OVERWRITE");
          debug!("env:PACT_OVERWRITE = {:?}", overwrite);
          ms.write_pact(
            &Some(output_dir),
            overwrite.unwrap_or_else(|_| "false".to_owned()) == "true")
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
