use tokio::io;
use failure::Fail;
use crypto::crypto_box::CryptoError;
use tezos_messages::p2p::binary_message::BinaryChunkError;

#[derive(Debug, Fail)]
pub enum SocketError {
    #[fail(display = "io error {}", _0)]
    Io(io::Error),
    #[fail(display = "encoding error")]
    EncodingError,
    #[fail(display = "decoding error")]
    DecodingError,
    #[fail(display = "wrong proof of work")]
    WrongPow,
    #[fail(display = "encryption error {}", _0)]
    Encryption(CryptoError),
    #[fail(display = "decryption error {}", _0)]
    Decryption(CryptoError),
    #[fail(display = "chunk error {}", _0)]
    Chunk(BinaryChunkError),
}
