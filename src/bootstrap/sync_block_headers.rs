use std::io::Write;
use slog::Logger;
use serde::{Serialize, Deserialize};
use tezos_messages::{
    cached_data,
    p2p::{
        encoding::{
            peer::{PeerMessageResponse, PeerMessage},
            current_branch::CurrentBranch,
            block_header::{BlockHeader, GetBlockHeadersMessage},
        },
        binary_message::{BinaryMessage, cache::BinaryDataCache},
    },
};
use tezos_encoding::{has_encoding, encoding::{HasEncoding, Encoding, Field}};
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

        let len = self.remote_branch.history().len();
        let mut last = self.remote_branch.history()[len - 2].clone();
        let mut chain = Chain {
            headers: Vec::<BlockHeader>::new(),
            body: Default::default(),
        };
        loop {
            connection.write(&GetBlockHeadersMessage::new(vec![last.clone()]).into()).await?;
            let r = connection.read().await?;
            match &r.messages()[0] {
                &PeerMessage::BlockHeader(ref h) => {
                    if h.block_header().predecessor().eq(&last) {
                        continue;
                    }
                    chain.headers.push(h.block_header().clone());
                    if h.block_header().level() == 0 {
                        break;
                    }
                    last = h.block_header().predecessor().clone();
                },
                _ => (),
            }
        }
        let data = chain.as_bytes().unwrap();
        let mut file = std::fs::File::create("target/data.dump").unwrap();
        file.write_all(data.as_ref()).unwrap();
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Chain {
    headers: Vec<BlockHeader>,

    #[serde(skip_serializing)]
    body: BinaryDataCache,
}

cached_data!(Chain, body);
has_encoding!(Chain, CHAIN_ENCODING, {
        Encoding::Obj(vec![
            Field::new("headers", Encoding::dynamic(Encoding::list(BlockHeader::encoding().clone()))),
        ])
});
