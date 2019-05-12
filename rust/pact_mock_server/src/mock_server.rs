use hyper_server;
use matching::MatchResult;

use pact_matching::models::{Pact, Interaction, PactSpecification};
use std::ffi::CString;
use std::path::PathBuf;
use std::io;
use std::sync::Mutex;
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
    /// List of all match results for requests this mock server has received
    pub matches: Vec<MatchResult>,
    /// List of resources that need to be cleaned up when the mock server completes
    pub resources: Vec<CString>,
    /// Pact that this mock server is based on
    pub pact: Pact,
    /// Receiver of match results
    matches_rx: std::sync::mpsc::Receiver<MatchResult>,
    /// Shutdown signal
    shutdown_tx: Option<futures::sync::oneshot::Sender<()>>
}

impl MockServer {
    /// Creates a new mock server with the given ID, pact and socket address,
    /// and spawns it on a tokio runtime
    pub fn spawn(id: String, pact: Pact, port: u16,
        runtime: &mut tokio::runtime::Runtime
    ) -> Result<MockServer, String> {
        let (mock_server, future) = MockServer::create_and_bind(id, pact, port)?;
        runtime.spawn(future);
        Ok(mock_server)
    }

    /// Creates a new mock server with the given ID, pact and socket address,
    /// and spawns it on a tokio current_thread runtime (convenient for testing)
    pub fn spawn_current_thread(id: String, pact: Pact, port: u16,
        runtime: &mut tokio::runtime::current_thread::Runtime
    ) -> Result<MockServer, String> {
        let (mock_server, future) = MockServer::create_and_bind(id, pact, port)?;
        runtime.spawn(future);
        Ok(mock_server)
    }

    fn create_and_bind(id: String, pact: Pact, port: u16) -> Result<(MockServer, impl Future<Item = (), Error = ()>), String> {
        let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel();
        let (matches_tx, matches_rx) = std::sync::mpsc::channel();

        let (future, socket_addr) = hyper_server::create_and_bind(
            pact.clone(),
            port as u16,
            shutdown_rx.map_err(|_| ()),
            matches_tx
        ).map_err(|err| format!("Could not start server: {}", err))?;

        let mock_server = MockServer {
            id: id.clone(),
            addr: socket_addr,
            matches: vec![],
            resources: vec![],
            pact: pact,
            matches_rx: matches_rx,
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

    /// Read pending matches from the running server. Will not block.
    pub fn read_match_results_from_server(&mut self) {
        loop {
            match self.matches_rx.try_recv() {
                Ok(match_result) => self.matches.push(match_result),
                Err(_) => break
            }
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

    /// Returns all the mismatches that have occurred with this mock server
    pub fn mismatches(&self) -> Vec<MatchResult> {
        //self.matches.extend(self.matches_rx.iter());

        let mismatches = self.matches.iter()
            .filter(|m| !m.matched())
            .map(|m| m.clone());
        let interactions: Vec<&Interaction> = self.matches.iter().map(|m| {
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
        let _write_lock = PACT_FILE_MUTEX.lock().unwrap();

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_server_read_matches_should_read_matches_when_sender_is_closed() {
        let match_result = MatchResult::RequestMatch(Interaction::default());

        let mut mock_server = {
            let (shutdown_tx, _) = futures::sync::oneshot::channel();
            let (matches_tx, matches_rx) = std::sync::mpsc::channel();

            matches_tx.send(match_result.clone()).unwrap();

            MockServer {
                id: "foobar".into(),
                addr: ([0, 0, 0, 0], 0).into(),
                matches: vec![],
                resources: vec![],
                pact: Pact::default(),
                matches_rx: matches_rx,
                shutdown_tx: Some(shutdown_tx)
            }
        };

        mock_server.read_match_results_from_server();
        assert_eq!(mock_server.matches, vec![match_result]);
    }
}
