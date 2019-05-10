use server;
use MatchResult;

use pact_matching::models::{Pact, Interaction, Request, OptionalBody, PactSpecification};
use pact_matching::Mismatch;
use pact_matching::s;
use std::ffi::CString;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::io::{self, Read, Write};
use serde_json::json;
use futures::future::Future;
use futures::stream::Stream;

/// Struct to represent a mock server
#[derive(Debug)]
pub struct MockServer {
    /// Mock server unique ID
    pub id: String,
    /// Address the mock server is running on
    pub addr: std::net::SocketAddr,
    /// Receiver of match results
    matches_rx: std::sync::mpsc::Receiver<MatchResult>,
    /// List of all match results for requests this mock server has received
    pub matches: Vec<MatchResult>,
    /// List of resources that need to be cleaned up when the mock server completes
    pub resources: Vec<CString>,
    /// Pact that this mock server is based on
    pub pact: Pact
}

impl MockServer {
    /// Creates a new mock server with the given ID, pact and socket address
    pub fn new(id: String, pact: Pact, addr: std::net::SocketAddr,
        matches_rx: std::sync::mpsc::Receiver<MatchResult>
    ) -> MockServer {
        MockServer {
            id: id.clone(),
            addr: addr,
            matches_rx: matches_rx,
            matches: vec![],
            resources: vec![],
            pact: pact
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

    fn read_matches(&mut self) {
        self.matches.extend(self.matches_rx.iter());
    }

    /// Returns all the mismatches that have occurred with this mock server
    pub fn mismatches(&self) -> Vec<MatchResult> {
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
            id.clone(),
            pact.clone(),
            port as u16,
            shutdown_rx.map_err(|_| ()),
            matches_tx
        ).map_err(|err| format!("Could not start server: {}", err))?;

        self.runtime.spawn(server);
        self.mock_servers.insert(id.clone(), Box::new(
            MockServer::new(id, pact, socket_addr, matches_rx)
        ));

        Ok(socket_addr.port())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_server_read_matches_should_read_nothing_if_server_not_started() {
        let mut mock_server = {
            let (_, matches_rx) = std::sync::mpsc::channel();
            MockServer::new("foobar".into(), Pact::default(), ([0, 0, 0, 0], 0).into(), matches_rx)
        };

        mock_server.read_matches();
        assert_eq!(mock_server.matches.len(), 0);
    }

    #[test]
    fn can_start_mock_server() {
        let mut manager = ServerManager::new();
        let result = manager.start_mock_server("foobar".into(), Pact::default(), 0);
        assert!(result.is_ok())
    }
}