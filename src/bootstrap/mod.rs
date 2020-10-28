use super::{
    error::SocketError,
    decipher_state::DecipherState,
    read_message_state::ReadMessageState,
};

pub type ChainId = [u8; 4];

pub mod genesis;

mod state;
pub use self::state::BootstrapState;
