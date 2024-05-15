//! Mock server implementation using Hyper

use std::net::SocketAddr;

use pact_models::v4::pact::V4Pact;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

use crate::mock_server::{MockServerConfig, MockServerEvent};

/// Create and bind the server, spawning the server loop into the runtime and returning the bound
/// address, the send end of the shutdown channel and the receive end of the event channel
pub(crate) async fn create_and_bind(
  server_id: &str,
  pact: V4Pact,
  addr: SocketAddr,
  config: MockServerConfig
) -> anyhow::Result<(SocketAddr, oneshot::Sender<()>, mpsc::Receiver<MockServerEvent>)> {
  let listener = TcpListener::bind(addr).await?;
  // let mut join_set = JoinSet::new();
  let (shutdown_send, mut shutdown_recv) = oneshot::channel::<()>();
  let (event_send, mut event_recv) = mpsc::channel::<MockServerEvent>(256);

  tokio::spawn(async move {
    loop {

    }
  });

  Ok((listener.local_addr()?, shutdown_send, event_recv))
}
