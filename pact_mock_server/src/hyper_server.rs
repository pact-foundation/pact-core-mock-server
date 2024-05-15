//! Mock server implementation using Hyper

use std::net::SocketAddr;
use std::sync::Arc;
use hyper_util::rt::TokioIo;

use pact_models::v4::pact::V4Pact;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinSet;
use tracing::{debug, error};

use crate::mock_server::{MockServerConfig, MockServerEvent};
use crate::mock_server::MockServerEvent::ConnectionFailed;

/// Create and bind the server, spawning the server loop into the runtime and returning the bound
/// address, the send end of the shutdown channel and the receive end of the event channel
pub(crate) async fn create_and_bind(
  server_id: &str,
  pact: V4Pact,
  addr: SocketAddr,
  config: MockServerConfig
) -> anyhow::Result<(SocketAddr, oneshot::Sender<()>, mpsc::Receiver<MockServerEvent>)> {
  let listener = TcpListener::bind(addr).await?;
  let local_addr = listener.local_addr()?;

  let mut join_set = JoinSet::new();
  let (shutdown_send, mut shutdown_recv) = oneshot::channel::<()>();
  let (event_send, mut event_recv) = mpsc::channel::<MockServerEvent>(256);

  tokio::spawn(async move {
    loop {
      select! {
        connection = listener.accept() => {
          match connection {
            Ok((stream, remote_address)) => {
              debug!("Received connection from remote {}", remote_address);
              let io = TokioIo::new(stream);
              join_set.spawn(async move {

              });
            },
            Err(e) => {
              error!("failed to accept connection: {e}");
              if let Err(err) = event_send.send(ConnectionFailed(e.to_string())).await {
                error!("Failed to send ConnectionFailed event: {}", err);
              }
            }
          }
        }

        _ = &mut shutdown_recv => {
          debug!("Received shutdown signal, waiting for existing connections to complete");
          while let Some(_) = join_set.join_next().await {};
          debug!("Existing connections complete, exiting main loop");
          break;
        }
      }
    }
  });

  Ok((local_addr, shutdown_send, event_recv))
}
