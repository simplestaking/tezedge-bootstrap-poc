use std::{mem, convert::TryInto};
use tokio::net::TcpStream;
use tezos_messages::p2p::encoding::{
    peer::{PeerMessageResponse, PeerMessage},
    current_branch::{CurrentBranchMessage, CurrentBranch, GetCurrentBranchMessage},
};
use slog::Logger;
use super::{
    SocketError, DecipherState, ReadMessageState,
    ChainId,
    genesis,
};

/// Reference to shared chain state
pub struct BootstrapState {
    inner: InnerState,
    reader: ReadMessageState<PeerMessageResponse>,
    decipher: DecipherState,
}

enum InnerState {
    Initial(ChainId),
    // -> GetCurrentBranch
    // <- GetCurrentBranch
    ExchangedCurrentBranchRequest(ChainId),
    // -> CurrentBranch
    // <- CurrentBranch
    ExchangedCurrentBranchResponse(SyncBlockHeaders),
    // -> GetBlockHeaders
    // <- BlockHeader
    Finish,

    // if peer request CurrentBranch with unknown chain id
    UnknownChain,

    Awaiting,
}

impl BootstrapState {
    pub fn new(decipher: DecipherState, chain_id: ChainId) -> Self {
        BootstrapState {
            inner: InnerState::Initial(chain_id),
            reader: ReadMessageState::new(),
            decipher: decipher,
        }
    }

    pub async fn run(self, logger: &Logger, stream: &mut TcpStream) -> Result<(), SocketError> {
        let mut s = self;
        loop {
            let current_state = mem::replace(&mut s.inner, InnerState::Awaiting);
            match current_state {
                InnerState::Finish => break Ok(()),
                InnerState::UnknownChain => break Ok(()),
                current_state => {
                    let _ = mem::replace(&mut s.inner, current_state);
                    s.run_inner(logger, stream).await?;
                },
            }
        }
    }

    async fn run_inner(&mut self, logger: &Logger, stream: &mut TcpStream) -> Result<(), SocketError> {
        let &mut BootstrapState { ref mut inner, ref mut reader, ref mut decipher } = self;
        let current_state = mem::replace(inner, InnerState::Awaiting);
        let new_state = match current_state {
            InnerState::Initial(chain_id) => {
                let peer_chain_id = Self::exchange_request(reader, decipher, logger, stream, &chain_id).await?;
                if peer_chain_id == chain_id {
                    InnerState::ExchangedCurrentBranchRequest(chain_id)
                } else {
                    InnerState::UnknownChain
                }
            },
            InnerState::ExchangedCurrentBranchRequest(chain_id) => {
                let peer_current_branch = Self::exchange_response(reader, decipher, logger, stream, &chain_id).await?;
                InnerState::ExchangedCurrentBranchResponse(SyncBlockHeaders {
                    chain_id: chain_id,
                    remote_branch: peer_current_branch,
                })
            },
            InnerState::ExchangedCurrentBranchResponse(mut s) => {
                s.run().await?;
                InnerState::Finish
            },
            InnerState::Finish => InnerState::Finish,
            InnerState::UnknownChain => InnerState::UnknownChain,
            InnerState::Awaiting => InnerState::Awaiting,
        };
        *inner = new_state;

        Ok(())
    }

    async fn exchange_request(
        reader: &mut ReadMessageState<PeerMessageResponse>,
        decipher: &mut DecipherState,
        logger: &Logger,
        stream: &mut TcpStream,
        chain_id: &ChainId,
    ) -> Result<ChainId, SocketError> {
        let request = GetCurrentBranchMessage::new(chain_id.to_vec());
        decipher.write_message::<_, PeerMessageResponse>(stream, [request.into()].as_ref()).await?;
        let request = reader.read_message(logger, stream, decipher).await?;
        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            &PeerMessage::GetCurrentBranch(ref m) => Ok(m.chain_id.clone().try_into().unwrap()),
            _ => panic!(),
        }
    }

    async fn exchange_response(
        reader: &mut ReadMessageState<PeerMessageResponse>,
        decipher: &mut DecipherState,
        logger: &Logger,
        stream: &mut TcpStream,
        chain_id: &ChainId,
    ) -> Result<CurrentBranch, SocketError> {
        let genesis_block_header = genesis::block_header();
        let current_branch = CurrentBranch::new(genesis_block_header, Vec::new());
        let response = CurrentBranchMessage::new(chain_id.to_vec(), current_branch);
        decipher.write_message::<_, PeerMessageResponse>(stream, [response.into()].as_ref()).await?;

        let request = reader.read_message(logger, stream, decipher).await?;
        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            &PeerMessage::CurrentBranch(ref m) => Ok(m.current_branch().clone()),
            _ => panic!(),
        }
    }
}

struct SyncBlockHeaders {
    chain_id: ChainId,
    remote_branch: CurrentBranch,
}

impl SyncBlockHeaders {
    pub async fn run(&mut self) -> Result<(), SocketError> {
        // TODO:
        let _ = (&mut self.chain_id, &mut self.remote_branch);
        Ok(())
    }
}
