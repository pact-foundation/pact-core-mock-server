//!
//! This module defines a manager for holding multiple instances of mock servers.
//!

use crate::mock_server::MockServer;

use pact_matching::models::Pact;
use std::collections::BTreeMap;

struct ServerEntry {
    mock_server: Box<MockServer>,
    join_handle: tokio::task::JoinHandle<()>,
}

/// Struct to represent many mock servers running in a background thread
pub struct ServerManager {
    runtime: tokio::runtime::Runtime,
    mock_servers: BTreeMap<String, ServerEntry>,
}

impl ServerManager {
    /// Construct a new ServerManager for scheduling several instances of mock servers
    /// on one tokio runtime.
    pub fn new() -> ServerManager {
        ServerManager {
            runtime: tokio::runtime::Builder::new()
                .threaded_scheduler()
                .core_threads(1)
                .enable_all()
                .build()
                .unwrap(),
            mock_servers: BTreeMap::new(),
        }
    }

    /// Start a new server on the runtime
    pub fn start_mock_server_with_addr(
        &mut self,
        id: String,
        pact: Pact,
        addr: std::net::SocketAddr,
    ) -> Result<std::net::SocketAddr, String> {
        let (mock_server, future) =
            self.runtime
                .block_on(MockServer::new(id.clone(), pact, addr))?;

        let addr = mock_server.addr;

        self.mock_servers.insert(
            id,
            ServerEntry {
                mock_server: Box::new(mock_server),
                join_handle: self.runtime.spawn(future),
            },
        );
        Ok(addr)
    }

    /// Start a new server on the runtime
    pub fn start_mock_server(&mut self, id: String, pact: Pact, port: u16) -> Result<u16, String> {
        self.start_mock_server_with_addr(id, pact, ([0, 0, 0, 0], port as u16).into())
            .map(|addr| addr.port())
    }

    /// Shut down a server by its id
    pub fn shutdown_mock_server_by_id(&mut self, id: String) -> bool {
        match self.mock_servers.remove(&id) {
            Some(mut entry) => match entry.mock_server.shutdown() {
                Ok(()) => {
                    self.runtime.block_on(entry.join_handle).unwrap();
                    true
                }
                Err(_) => false,
            },
            None => false,
        }
    }

    /// Shut down a server by its local port number
    pub fn shutdown_mock_server_by_port(&mut self, port: u16) -> bool {
        log::debug!("Shutting down mock server with port {}", port);
        let result = self
            .mock_servers
            .iter()
            .find(|(_id, entry)| entry.mock_server.addr.port() == port)
            .map(|(_id, entry)| entry.mock_server.id.clone());

        if let Some(id) = result {
            if let Some(mut entry) = self.mock_servers.remove(&id) {
                return match entry.mock_server.shutdown() {
                    Ok(()) => {
                        self.runtime.block_on(entry.join_handle).unwrap();
                        true
                    }
                    Err(_) => false,
                };
            }
        }

        false
    }

    /// Find mock server by id, and map it using supplied function if found
    pub fn find_mock_server_by_id<R>(
        &self,
        id: &String,
        f: &dyn Fn(&MockServer) -> R,
    ) -> Option<R> {
        match self.mock_servers.get(id) {
            Some(entry) => Some(f(&entry.mock_server)),
            None => None,
        }
    }

    /// Find a mock server by port number and apply a mutating operation on it if successful
    pub fn find_mock_server_by_port_mut<R>(
        &mut self,
        mock_server_port: u16,
        f: &dyn Fn(&mut MockServer) -> R,
    ) -> Option<R> {
        match self
            .mock_servers
            .iter_mut()
            .find(|(_id, entry)| entry.mock_server.addr.port() == mock_server_port)
        {
            Some((_id, entry)) => Some(f(&mut entry.mock_server)),
            None => None,
        }
    }

    /// Map all the running mock servers
    pub fn map_mock_servers<R>(&self, f: &dyn Fn(&MockServer) -> R) -> Vec<R> {
        let mut results = vec![];
        for (_id_, entry) in self.mock_servers.iter() {
            results.push(f(&entry.mock_server));
        }
        return results;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpStream;
    use std::{thread, time};

    #[test]
    fn manager_should_start_and_shutdown_mock_server() {
        let mut manager = ServerManager::new();
        let start_result = manager.start_mock_server("foobar".into(), Pact::default(), 0);

        assert!(start_result.is_ok());
        let server_port = start_result.unwrap();

        // Server should be up
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_ok());

        // Should be able to read matches without blocking
        let matches =
            manager.find_mock_server_by_port_mut(server_port, &|mock_server| mock_server.matches());
        assert_eq!(matches, Some(vec![]));

        let stopped = manager.shutdown_mock_server_by_port(server_port);
        assert!(stopped);

        // The tokio runtime is now out of tasks
        drop(manager);

        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        // Server should be down
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_err());
    }
}
