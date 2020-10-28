use std::{mem, convert::TryFrom};
use tezos_messages::p2p::encoding::{
    peer::PeerMessageResponse,
    current_branch::{CurrentBranchMessage, CurrentBranch, GetCurrentBranchMessage},
};
use super::{SocketError, TrustedConnection, ChainId, genesis, message::{Request, Response}};

/// Reference to shared chain state
pub struct BootstrapState {
    state: FullState,
    connection: TrustedConnection<PeerMessageResponse>,
}

enum FullState {
    Initial(ChainId),
    // -> GetCurrentBranch
    AskedRemoteBranch(ChainId),
    // <- CurrentBranch
    ReceivedRemoteBranch(SyncBlockHeaders),
    // -> GetBlockHeaders
    // <- BlockHeader
    // next state is again `ReceivedRemoteBranch` or `FullState::Finish`
    // TODO: result of bootstrap
    Finish,
    // if peer requested CurrentBranch with unknown chain id
    UnknownChain,
    Awaiting,
}

impl BootstrapState {
    pub fn new(connection: TrustedConnection<PeerMessageResponse>, chain_id: ChainId) -> Self {
        BootstrapState {
            state: FullState::Initial(chain_id),
            connection: connection,
        }
    }

    pub async fn run(self) -> Result<(), SocketError> {
        let mut s = self;
        loop {
            let current_state = mem::replace(&mut s.state, FullState::Awaiting);
            match current_state {
                FullState::Finish => break Ok(()),
                FullState::UnknownChain => break Ok(()),
                current_state => {
                    let _ = mem::replace(&mut s.state, current_state);
                    s.run_inner().await?;
                },
            }
        }
    }

    async fn run_inner(&mut self) -> Result<(), SocketError> {
        let current_state = mem::replace(&mut self.state, FullState::Awaiting);
        let new_state = match current_state {
            FullState::Initial(chain_id) => {
                // ask remote branch
                let request = GetCurrentBranchMessage::new(chain_id.to_vec());
                self.connection.write(&request.into()).await?;
                FullState::AskedRemoteBranch(chain_id)
            },
            FullState::AskedRemoteBranch(chain_id) => {
                let message = self.connection.read().await?;
                let to_write = self.handle_peer_request(&message, chain_id);
                if !to_write.is_empty() {
                    self.connection.write_batch(to_write.as_ref()).await?;
                }
                match self.handle_peer_response(&message, chain_id) {
                    None => FullState::AskedRemoteBranch(chain_id),
                    Some(None) => FullState::UnknownChain,
                    Some(Some(peer_current_branch)) => {
                        FullState::ReceivedRemoteBranch(SyncBlockHeaders {
                            chain_id: chain_id,
                            remote_branch: peer_current_branch,
                        })
                    },
                }
            },
            FullState::ReceivedRemoteBranch(mut s) => {
                s.run().await?;
                FullState::Finish
            },
            FullState::Finish => FullState::Finish,
            FullState::UnknownChain => FullState::UnknownChain,
            FullState::Awaiting => FullState::Awaiting,
        };
        self.state = new_state;

        Ok(())
    }

    fn handle_peer_response(
        &mut self,
        message: &PeerMessageResponse,
        chain_id: ChainId,
    ) -> Option<Option<CurrentBranch>> {
        for response in Response::filter(&message) {
            match response {
                Response::CurrentBranch(m) => {
                    if ChainId::try_from(m.chain_id().clone()).unwrap() == chain_id {
                        return Some(Some(m.current_branch().clone()));
                    } else {
                        return Some(None);
                    }
                },
                _ => (),
            }
        }

        None
    }

    fn handle_peer_request(
        &mut self,
        message: &PeerMessageResponse,
        chain_id: ChainId,
    ) -> Vec<PeerMessageResponse> {
        let mut write = Vec::new();
        for request in Request::filter(message) {
            match request {
                Request::GetCurrentBranch(m) => {
                    if ChainId::try_from(m.chain_id.clone()).unwrap() == chain_id {
                        let genesis_block_header = genesis::block_header();
                        let current_branch = CurrentBranch::new(genesis_block_header, Vec::new());
                        let response = CurrentBranchMessage::new(chain_id.to_vec(), current_branch);
                        write.push(response.into())
                    }
                },
                // ignore
                _ => (),
            }
        }
        write
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
