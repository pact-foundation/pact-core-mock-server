use server;
use MatchResult;

use pact_matching::models::{Pact, Interaction, PactSpecification};
use std::ffi::CString;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::io;
use serde_json::json;
use futures::future::Future;

/// Struct to represent a mock server
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
    shutdown_tx: futures::sync::oneshot::Sender<()>
}

impl MockServer {
    /// Creates a new mock server with the given ID, pact and socket address
    pub fn new(id: String, pact: Pact, addr: std::net::SocketAddr,
        matches_rx: std::sync::mpsc::Receiver<MatchResult>,
        shutdown_tx: futures::sync::oneshot::Sender<()>
    ) -> MockServer {
        MockServer {
            id: id.clone(),
            addr: addr,
            matches: vec![],
            resources: vec![],
            pact: pact,
            matches_rx: matches_rx,
            shutdown_tx: shutdown_tx
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

    fn read_matches_from_server(&mut self) {
        self.matches.extend(self.matches_rx.iter());
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

pub struct ServerManager {
    runtime: tokio::runtime::Runtime,
    mock_servers: BTreeMap<String, Box<MockServer>>
}

impl ServerManager {
    pub fn new() -> ServerManager {
        ServerManager {
            runtime: tokio::runtime::Builder::new()
                .blocking_threads(1)
                .build()
                .unwrap(),
            mock_servers: BTreeMap::new()
        }
    }

    pub fn start_mock_server(&mut self, id: String, pact: Pact, port: u16) -> Result<u16, String> {
        let (shutdown_tx, shutdown_rx) = futures::sync::oneshot::channel();
        let (matches_tx, matches_rx) = std::sync::mpsc::channel();

        let (server, socket_addr) = server::create_and_bind(
            pact.clone(),
            port as u16,
            shutdown_rx.map_err(|_| ()),
            matches_tx
        ).map_err(|err| format!("Could not start server: {}", err))?;

        self.runtime.spawn(server);
        self.mock_servers.insert(id.clone(), Box::new(
            MockServer::new(id, pact, socket_addr, matches_rx, shutdown_tx)
        ));

        Ok(socket_addr.port())
    }

    pub fn shutdown_mock_server_by_port(&mut self, port: u16) -> bool {
        debug!("Shutting down mock server with port {}", port);
        let result = self.mock_servers.iter()
            .find(|ms| ms.1.addr.port() == port)
            .map(|ms| ms.1.id.clone());

        if let Some(id) = result {
            if let Some(mock_server) = self.mock_servers.remove(&id) {
                mock_server.shutdown_tx.send(()).unwrap();
                return true
            }
        }

        false
    }

    pub fn find_server_by_port_mut<R>(&mut self, mock_server_port: u16, f: &Fn(&mut MockServer) -> R) -> Option<R> {
        match self.mock_servers.iter_mut().find(|ms| ms.1.addr.port() == mock_server_port) {
            Some(mock_server) => {
                mock_server.1.read_matches_from_server();
                Some(f(mock_server.1))
            },
            None => None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;

    #[test]
    fn mock_server_read_matches_should_read_matches_when_sender_is_closed() {
        let match_result = MatchResult::RequestMatch(Interaction::default());

        let mut mock_server = {
            let (shutdown_tx, _) = futures::sync::oneshot::channel();
            let (matches_tx, matches_rx) = std::sync::mpsc::channel();

            matches_tx.send(match_result.clone()).unwrap();

            MockServer::new("foobar".into(), Pact::default(), ([0, 0, 0, 0], 0).into(), matches_rx, shutdown_tx)
        };

        mock_server.read_matches_from_server();
        assert_eq!(mock_server.matches, vec![match_result]);
    }

    #[test]
    fn manager_should_start_and_shutdown_mock_server() {
        let mut manager = ServerManager::new();
        let start_result = manager.start_mock_server("foobar".into(), Pact::default(), 0);

        assert!(start_result.is_ok());
        let server_port = start_result.unwrap();

        // Server should be up
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_ok());

        let stopped = manager.shutdown_mock_server_by_port(server_port);
        assert!(stopped);

        // The tokio runtime is now out of tasks
        manager.runtime.shutdown_on_idle().wait().unwrap();

        // Server should be down
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_err());
    }
}