use tokio::net::TcpStream;
use tezos_messages::p2p::encoding::{
    peer::PeerMessageResponse,
};
use slog::Logger;
use super::{
    error::SocketError,
    decipher_state::DecipherState,
    read_message_state::ReadMessageState,
};

/// Reference to shared chain state
pub struct BootstrapState {
    inner: Option<InnerState>,
    reader: ReadMessageState<PeerMessageResponse>,
    decipher: DecipherState,
}

enum InnerState {
    // -> GetCurrentBranch
    // <- GetCurrentBranch
    // <- CurrentBranch
    // -> CurrentBranch
    ExchangeCurrentBranch,
    // -> GetBlockHeaders
    // <- BlockHeader
    // AskBlockHeaders,
}

impl BootstrapState {
    pub fn new(decipher: DecipherState) -> Self {
        BootstrapState {
            inner: Some(InnerState::ExchangeCurrentBranch),
            reader: ReadMessageState::new(),
            decipher: decipher,
        }
    }

    pub async fn run(&mut self, logger: &Logger, stream: &mut TcpStream) -> Result<(), SocketError> {
        let &mut BootstrapState { ref mut inner, ref mut reader, ref mut decipher } = self;
        let _ = inner;

        let _message = loop {
            if let Some(message) = reader.read_message(logger, stream, decipher).await? {
                break message;
            }
        };

        Ok(())
    }
}
