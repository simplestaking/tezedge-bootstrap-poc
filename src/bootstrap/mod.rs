use super::{error::SocketError, trusted_connection::TrustedConnection};

pub type ChainId = [u8; 4];

pub mod genesis;

mod message;

mod sync_block_headers;
#[allow(warnings)]
mod blockchain;

mod state;
pub use self::state::BootstrapState;
