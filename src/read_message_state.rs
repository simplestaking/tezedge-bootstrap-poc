use std::{mem, fmt};
use tokio::net::TcpStream;
use slog::Logger;
use tezos_messages::p2p::binary_message::BinaryMessage;
use tezos_encoding::binary_reader::BinaryReaderError;
use super::{
    error::SocketError,
    decipher_state::DecipherState,
};

pub enum ReadMessageState<M>
where
    M: BinaryMessage,
{
    Empty,
    Unknown {
        buffer: Vec<u8>,
    },
    Buffering {
        remaining: usize,
        buffer: Vec<u8>,
    },
    HasMessage(M),
    Awaiting,
}

impl<M> ReadMessageState<M>
where
    M: BinaryMessage + fmt::Debug,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        ReadMessageState::Unknown {
            buffer: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub async fn read_message(&mut self, logger: &Logger, stream: &mut TcpStream, decipher: &mut DecipherState) -> Result<Option<M>, SocketError> {
        let current_state = mem::replace(self, ReadMessageState::Empty);
        match current_state {
            ReadMessageState::HasMessage(message) => {
                slog::debug!(logger, "message: {:x?}", message);
                Ok(Some(message))
            },
            current_state => {
                let _ = mem::replace(self, current_state);
                self.run(stream, decipher).await?;
                Ok(None)
            }
        }
    }

    async fn run(&mut self, stream: &mut TcpStream, decipher: &mut DecipherState) -> Result<(), SocketError> {
        let current_state = mem::replace(self, ReadMessageState::Awaiting);
        let new_state = match current_state {
            ReadMessageState::Empty => {
                let data = decipher.read_chunk(stream).await?;
                // do no know if it is enough, the new state is Unknown
                ReadMessageState::Unknown {
                    buffer: data,
                }
            },
            ReadMessageState::Unknown { buffer } => {
                // check if it is enough, the new state is Buffering or HasMessage
                match M::from_bytes(&buffer) {
                    Ok(message) => ReadMessageState::HasMessage(message),
                    Err(BinaryReaderError::Underflow { bytes }) => {
                        ReadMessageState::Buffering {
                            remaining: bytes,
                            buffer: buffer,
                        }
                    },
                    Err(_) => return Err(SocketError::DecodingError)
                }        
            },
            ReadMessageState::Buffering { remaining, mut buffer } => {
                // add one more chunk
                let chunk = decipher.read_chunk(stream).await?;
                buffer.extend_from_slice(chunk.as_ref());
                // if buffered what is remaining, move to Unknown, otherwise continue Buffering
                if chunk.len() >= remaining {
                    ReadMessageState::Unknown { buffer }
                } else {
                    ReadMessageState::Buffering {
                        remaining: remaining - chunk.len(),
                        buffer: buffer,
                    }
                }
            },
            ReadMessageState::HasMessage(message) => ReadMessageState::HasMessage(message),
            ReadMessageState::Awaiting => ReadMessageState::Awaiting,
        };
        let _ = mem::replace(self, new_state);
        Ok(())
    }
}
