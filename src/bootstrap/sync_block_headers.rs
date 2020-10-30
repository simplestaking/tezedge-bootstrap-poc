use slog::Logger;
use tezos_messages::p2p::{
    encoding::{
        peer::{PeerMessageResponse, PeerMessage},
        current_branch::CurrentBranch,
        block_header::{BlockHeader, GetBlockHeadersMessage},
    },
};
use super::{SocketError, TrustedConnection};

pub struct SyncBlockHeaders {
    remote_branch: CurrentBranch,
}

impl SyncBlockHeaders {
    pub fn new(remote_branch: CurrentBranch) -> Self {
        SyncBlockHeaders {
            remote_branch: remote_branch,
        }
    }

    pub async fn run(
        &mut self,
        connection: &mut TrustedConnection<PeerMessageResponse>,
        _logger: &Logger,
    ) -> Result<(), SocketError> {
        // TODO:
        // let head: &BlockHeader = self.remote_branch.current_head();

        let mut last = self.remote_branch.history().last().unwrap().clone();
        let mut chain = Vec::<BlockHeader>::new();
        loop {
            connection.write(&GetBlockHeadersMessage::new(vec![last.clone()]).into()).await?;
            let r = connection.read().await?;
            match &r.messages()[0] {
                &PeerMessage::BlockHeader(ref h) => {
                    chain.push(h.block_header().clone());
                    last = h.block_header().predecessor().clone();
                },
                _ => (),
            }
        }
    }
}
