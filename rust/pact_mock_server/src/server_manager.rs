use mock_server::MockServer;

use pact_matching::models::{Pact};
use std::collections::BTreeMap;

// Struct to represent many mock servers running in a background thread
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
        let mock_server = MockServer::spawn(id.clone(), pact, port, &mut self.runtime)?;
        let port = mock_server.addr.port();

        self.mock_servers.insert(id, Box::new(mock_server));

        Ok(port)
    }

    pub fn shutdown_mock_server_by_port(&mut self, port: u16) -> bool {
        debug!("Shutting down mock server with port {}", port);
        let result = self.mock_servers.iter()
            .find(|ms| ms.1.addr.port() == port)
            .map(|ms| ms.1.id.clone());

        if let Some(id) = result {
            if let Some(mut mock_server) = self.mock_servers.remove(&id) {
                return match mock_server.shutdown() {
                    Ok(()) => true,
                    Err(_) => false
                }
            }
        }

        false
    }

    pub fn find_server_by_port_mut<R>(&mut self, mock_server_port: u16, f: &Fn(&mut MockServer) -> R) -> Option<R> {
        match self.mock_servers.iter_mut().find(|ms| ms.1.addr.port() == mock_server_port) {
            Some(mock_server) => {
                mock_server.1.read_match_results_from_server();
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
    use futures::future::Future;

    #[test]
    fn manager_should_start_and_shutdown_mock_server() {
        let mut manager = ServerManager::new();
        let start_result = manager.start_mock_server("foobar".into(), Pact::default(), 0);

        assert!(start_result.is_ok());
        let server_port = start_result.unwrap();

        // Server should be up
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_ok());

        // Should be able to read matches without blocking
        let matches = manager.find_server_by_port_mut(server_port, &|mock_server| {
            mock_server.matches.clone()
        });
        assert_eq!(matches, Some(vec![]));

        let stopped = manager.shutdown_mock_server_by_port(server_port);
        assert!(stopped);

        // The tokio runtime is now out of tasks
        manager.runtime.shutdown_on_idle().wait().unwrap();

        // Server should be down
        assert!(TcpStream::connect(("127.0.0.1", server_port)).is_err());
    }
}
