use tezos_messages::p2p::encoding::block_header::BlockHeader;
use super::ChainId;

pub fn block_header() -> BlockHeader {
    use tezos_messages::p2p::encoding::block_header::BlockHeaderBuilder;

    let genesis_hash = hex::decode("8fcf233671b6a04fcf679d2a381c2544ea6c1ea29ba6157776ed8424bbcacb48").unwrap();
    let genesis_time = 1574946133;
    let operation_list_list_hash = hex::decode("0e5751c026e543b2e8ab2eb06099daa1d1e5df47778f7787faab45cdf12fe3a8").unwrap();
    let context_hash = hex::decode("fc73e5a7e0733387f86ad1b704121b81835c9a510a529ed96ecb9c34b732231b").unwrap();
    BlockHeaderBuilder::default()
        .level(0)
        .proto(0)
        .predecessor(genesis_hash)
        .timestamp(genesis_time)
        .validation_pass(0)
        .operations_hash(operation_list_list_hash)
        .fitness(vec![])
        .context(context_hash)
        .protocol_data(vec![])
        .build().unwrap()
}

pub const CHAIN_ID: ChainId = 0x9caecab9u32.to_be_bytes();
