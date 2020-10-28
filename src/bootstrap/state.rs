use std::{mem, convert::TryInto};
use tezos_messages::p2p::encoding::{
    peer::{PeerMessageResponse, PeerMessage},
    current_branch::{CurrentBranchMessage, CurrentBranch, GetCurrentBranchMessage},
};
use slog::Logger;
use super::{SocketError, TrustedConnection, ChainId, genesis};

/// Reference to shared chain state
pub struct BootstrapState {
    inner: InnerState,
    connection: TrustedConnection<PeerMessageResponse>,
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
    pub fn new(connection: TrustedConnection<PeerMessageResponse>, chain_id: ChainId) -> Self {
        BootstrapState {
            inner: InnerState::Initial(chain_id),
            connection: connection,
        }
    }

    pub async fn run(self, logger: &Logger) -> Result<(), SocketError> {
        let mut s = self;
        loop {
            let current_state = mem::replace(&mut s.inner, InnerState::Awaiting);
            match current_state {
                InnerState::Finish => break Ok(()),
                InnerState::UnknownChain => break Ok(()),
                current_state => {
                    let _ = mem::replace(&mut s.inner, current_state);
                    s.run_inner(logger).await?;
                },
            }
        }
    }

    async fn run_inner(&mut self, _logger: &Logger) -> Result<(), SocketError> {
        let current_state = mem::replace(&mut self.inner, InnerState::Awaiting);
        let new_state = match current_state {
            InnerState::Initial(chain_id) => {
                let peer_chain_id = self.exchange_request(&chain_id).await?;
                if peer_chain_id == chain_id {
                    InnerState::ExchangedCurrentBranchRequest(chain_id)
                } else {
                    InnerState::UnknownChain
                }
            },
            InnerState::ExchangedCurrentBranchRequest(chain_id) => {
                let peer_current_branch = self.exchange_response(&chain_id).await?;
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
        self.inner = new_state;

        Ok(())
    }

    async fn exchange_request(&mut self, chain_id: &ChainId) -> Result<ChainId, SocketError> {
        let request = GetCurrentBranchMessage::new(chain_id.to_vec());
        self.connection.write([request.into()].as_ref()).await?;
        let request = self.connection.read().await?;
        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            &PeerMessage::GetCurrentBranch(ref m) => Ok(m.chain_id.clone().try_into().unwrap()),
            _ => panic!(),
        }
    }

    async fn exchange_response(
        &mut self,
        chain_id: &ChainId,
    ) -> Result<CurrentBranch, SocketError> {
        let genesis_block_header = genesis::block_header();
        let current_branch = CurrentBranch::new(genesis_block_header, Vec::new());
        let response = CurrentBranchMessage::new(chain_id.to_vec(), current_branch);
        self.connection.write([response.into()].as_ref()).await?;
        let response = self.connection.read().await?;
        assert_eq!(response.messages().len(), 1);
        match &response.messages()[0] {
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
