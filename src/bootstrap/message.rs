use tezos_messages::p2p::encoding::prelude::*;

#[derive(Debug)]
pub enum Request<'a> {
    GetCurrentBranch(&'a GetCurrentBranchMessage),
    GetCurrentHead(&'a GetCurrentHeadMessage),
    GetBlockHeaders(&'a GetBlockHeadersMessage),
    GetOperations(&'a GetOperationsMessage),
    GetProtocols(&'a GetProtocolsMessage),
    GetOperationHashesForBlocks(&'a GetOperationHashesForBlocksMessage),
    GetOperationsForBlocks(&'a GetOperationsForBlocksMessage),
}

impl<'a> Request<'a> {
    pub fn filter(m: &'a PeerMessageResponse) -> impl Iterator<Item = Request<'a>> {
        m.messages().iter().filter_map(|m| match m {
            &PeerMessage::GetCurrentBranch(ref m) => Some(Request::GetCurrentBranch(m)),
            &PeerMessage::GetCurrentHead(ref m) => Some(Request::GetCurrentHead(m)),
            &PeerMessage::GetBlockHeaders(ref m) => Some(Request::GetBlockHeaders(m)),
            &PeerMessage::GetOperations(ref m) => Some(Request::GetOperations(m)),
            &PeerMessage::GetProtocols(ref m) => Some(Request::GetProtocols(m)),
            &PeerMessage::GetOperationHashesForBlocks(ref m) => Some(Request::GetOperationHashesForBlocks(m)),
            &PeerMessage::GetOperationsForBlocks(ref m) => Some(Request::GetOperationsForBlocks(m)),
            _ => None,
        })
    }
}

#[derive(Debug)]
pub enum Response<'a> {
    CurrentBranch(&'a CurrentBranchMessage),
    CurrentHead(&'a CurrentHeadMessage),
    BlockHeader(&'a BlockHeaderMessage),
    Operation(&'a OperationMessage),
    Protocol(&'a ProtocolMessage),
    OperationHashesForBlock(&'a OperationHashesForBlocksMessage),
    OperationsForBlocks(&'a OperationsForBlocksMessage),
}

impl<'a> Response<'a> {
    pub fn filter(m: &'a PeerMessageResponse) -> impl Iterator<Item = Response<'a>> {
        m.messages().iter().filter_map(|m| match m {
            &PeerMessage::CurrentBranch(ref m) => Some(Response::CurrentBranch(m)),
            &PeerMessage::CurrentHead(ref m) => Some(Response::CurrentHead(m)),
            &PeerMessage::BlockHeader(ref m) => Some(Response::BlockHeader(m)),
            &PeerMessage::Operation(ref m) => Some(Response::Operation(m)),
            &PeerMessage::Protocol(ref m) => Some(Response::Protocol(m)),
            &PeerMessage::OperationHashesForBlock(ref m) => Some(Response::OperationHashesForBlock(m)),
            &PeerMessage::OperationsForBlocks(ref m) => Some(Response::OperationsForBlocks(m)),
            _ => None,
        })
    }
}
