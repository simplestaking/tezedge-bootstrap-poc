use std::net::SocketAddr;
use tokio::sync::oneshot;
use slog::Logger;
use super::{error::SocketError, socket_state::SocketState};

pub struct Socket {
    state: SocketState,
    shutdown_rx: oneshot::Receiver<()>,
}

pub struct Shutdown {
    tx: oneshot::Sender<()>,
}

impl Shutdown {
    pub fn shutdown(self) {
        let _ = self.tx.send(());
    }
}

impl Socket {
    pub fn outgoing(address: SocketAddr) -> (Self, Shutdown) {
        let (tx, rx) = oneshot::channel();
        (
            Socket {
                state: SocketState::outgoing(address),
                shutdown_rx: rx,
            },
            Shutdown { tx: tx },
        )
    }

    pub async fn run(&mut self, logger: &Logger) -> Result<(), SocketError> {
        loop {
            // TODO:
            let _ = &self.shutdown_rx;

            self.state.run(&logger).await.unwrap();
            if let &SocketState::Finish = &self.state {
                break Ok(());
            }
        }
    }
}
