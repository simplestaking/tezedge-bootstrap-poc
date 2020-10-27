use tokio::net::TcpStream;
use tezos_messages::p2p::encoding::{
    peer::{PeerMessageResponse, PeerMessage},
    current_branch::{CurrentBranchMessage, CurrentBranch},
};
use slog::Logger;
use super::{
    SocketError, DecipherState, ReadMessageState,
    genesis,
};

/// Reference to shared chain state
pub struct BootstrapState {
    inner: Option<InnerState>,
    reader: ReadMessageState<PeerMessageResponse>,
    decipher: DecipherState,
}

#[allow(dead_code)]
enum InnerState {
    // -> GetCurrentBranch
    // <- GetCurrentBranch
    // <- CurrentBranch
    // -> CurrentBranch
    ExchangeCurrentBranch,
    // -> GetBlockHeaders
    // <- BlockHeader
    AskBlockHeaders,
    // if peer request CurrentBranch with unknown chain id
    UnknownChain,
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

        let message = reader.read_message(logger, stream, decipher).await?;

        for message in message.messages() {
            match message {
                &PeerMessage::GetCurrentBranch(_) => {
                    let genesis_block_header = genesis::block_header();
                    let current_branch = CurrentBranch::new(genesis_block_header, Vec::new());
                    let current_branch_message = CurrentBranchMessage::new(genesis::CHAIN_ID.to_vec(), current_branch);
                    decipher.write_message::<_, PeerMessageResponse>(stream, &[current_branch_message.into()]).await?;
                },
                _ => (),
            }
        }

        Ok(())
    }
}
