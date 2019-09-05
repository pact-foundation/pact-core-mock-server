//!
//! This module defines the external interface for controlling one particular
//! instance of a mock server.
//!

use hyper_server;
use matching::MatchResult;

use pact_matching::models::{Pact, Interaction, PactSpecification};
use std::ffi::CString;
use std::path::PathBuf;
use std::io;
use std::sync::{Arc, Mutex};
use serde_json::json;
use futures::future::Future;

lazy_static! {
    static ref PACT_FILE_MUTEX: Mutex<()> = Mutex::new(());
}

/// Struct to represent the "foreground" part of mock server
#[derive(Debug)]
pub struct MockServer {
    /// Mock server unique ID
    pub id: String,
    /// Address the mock server is running on
    pub addr: std::net::SocketAddr,
    /// List of resources that need to be cleaned up when the mock server completes
    pub resources: Vec<CString>,
    /// Pact that this mock server is based on
    pub pact: Pact,
    /// Receiver of match results
    matches: Arc<Mutex<Vec<MatchResult>>>,
    /// Shutdown signal
    shutdown_tx: Option<futures::sync::oneshot::Sender<()>>
}

impl MockServer {
    /// Create a new mock server, consisting of its state (self) and its executable server future.
    pub fn new(id: String, pact: Pact, addr: std::net::SocketAddr) -> Result<(MockServer, impl Future<Item = (), Error = ()>), String> {
        let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel();
        let matches = Arc::new(Mutex::new(vec![]));

        let (future, socket_addr) = hyper_server::create_and_bind(
            pact.clone(),
            addr,
            shutdown_rx.map_err(|_| ()),
            matches.clone()
        ).map_err(|err| format!("Could not start server: {}", err))?;

        let mock_server = MockServer {
            id: id.clone(),
            addr: socket_addr,
            resources: vec![],
            pact: pact,
            matches: matches,
            shutdown_tx: Some(shutdown_tx)
        };

        Ok((mock_server, future))
    }

    /// Send the shutdown signal to the server
    pub fn shutdown(&mut self) -> Result<(), String> {
        match self.shutdown_tx.take() {
            Some(sender) => {
                match sender.send(()) {
                    Ok(()) => Ok(()),
                    Err(_) => Err("Problem sending shutdown signal to mock server".into())
                }
            },
            _ => Err("Mock server already shut down".into())
        }
    }

    /// Converts this mock server to a `Value` struct
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "id" : json!(self.id.clone()),
            "port" : json!(self.addr.port() as u64),
            "provider" : json!(self.pact.provider.name.clone()),
            "status" : json!(if self.mismatches().is_empty() { "ok" } else { "error" })
        })
    }

    /// Returns all collected matches
    pub fn matches(&self) -> Vec<MatchResult> {
        self.matches.lock().unwrap().clone()
    }

    /// Returns all the mismatches that have occurred with this mock server
    pub fn mismatches(&self) -> Vec<MatchResult> {
        let matches = self.matches();
        let mismatches = matches.iter()
            .filter(|m| !m.matched())
            .map(|m| m.clone());
        let interactions: Vec<&Interaction> = matches.iter().map(|m| {
            match *m {
                MatchResult::RequestMatch(ref interaction) => Some(interaction),
                MatchResult::RequestMismatch(ref interaction, _) => Some(interaction),
                MatchResult::RequestNotFound(_) => None,
                MatchResult::MissingRequest(_) => None
            }
        }).filter(|o| o.is_some()).map(|o| o.unwrap()).collect();
        let missing = self.pact.interactions.iter()
            .filter(|i| !interactions.contains(i))
            .map(|i| MatchResult::MissingRequest(i.clone()));
        mismatches.chain(missing).collect()
    }

    /// Mock server writes its pact out to the provided directory
    pub fn write_pact(&self, output_path: &Option<String>) -> io::Result<()> {
        let pact_file_name = self.pact.default_file_name();
        let filename = match *output_path {
            Some(ref path) => {
                let mut path = PathBuf::from(path);
                path.push(pact_file_name);
                path
            },
            None => PathBuf::from(pact_file_name)
        };

        info!("Writing pact out to '{}'", filename.display());

        // Lock so that no two threads can read/write pact file at the same time.
        // TODO: Could use a fs-based lock in case multiple processes are doing
        // this concurrently?
        let _file_lock = PACT_FILE_MUTEX.lock().unwrap();

        match self.pact.write_pact(filename.as_path(), PactSpecification::V3) {
            Ok(_) => Ok(()),
            Err(err) => {
                warn!("Failed to write pact to file - {}", err);
                Err(err)
            }
        }
    }

    /// Returns the URL of the mock server
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.addr.port())
    }
}

impl Clone for MockServer {
    /// Make a clone all of the MockServer fields.
    /// Note that the clone of the original server cannot be shut down directly.
    fn clone(&self) -> MockServer {
        MockServer {
            id: self.id.clone(),
            addr: self.addr,
            resources: vec![],
            pact: self.pact.clone(),
            matches: self.matches.clone(),
            shutdown_tx: None
        }
    }
}
