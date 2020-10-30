use std::collections::BTreeMap;
use tezos_messages::p2p::{
    binary_message::BinaryMessage,
    encoding::block_header::BlockHeader,
};
use crypto::{blake2b, hash::Hash};

pub enum Level {
    Precise(u32),
    Guess(u32),
}

pub enum Block {
    Hash(Hash),
    Header(BlockHeader),
}

pub struct HeadersChain {
    blocks: Vec<BlockHeader>,
}

pub struct BlockChain {
    sequence: Vec<HeadersChain>,
}
