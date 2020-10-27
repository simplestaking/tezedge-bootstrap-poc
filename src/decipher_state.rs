use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tezos_messages::p2p::binary_message::{BinaryChunk, BinaryMessage};
use tezos_conversation::{Decipher, NonceAddition};
use super::error::SocketError;

pub const CONTENT_LENGTH_MAX: usize =
    tezos_messages::p2p::binary_message::CONTENT_LENGTH_MAX - crypto::crypto_box::BOX_ZERO_BYTES;

pub struct DecipherState {
    decipher: Decipher,
    initiator: bool,
    initiators_counter: u64,
    responders_counter: u64,
}

impl DecipherState {
    pub fn new(decipher: Decipher, initiator: bool) -> Self {
        DecipherState {
            decipher: decipher,
            initiator: initiator,
            initiators_counter: 0,
            responders_counter: 0,
        }
    }

    pub fn encrypt(&mut self, data: &[u8]) -> Result<BinaryChunk, SocketError> {
        let (chunk_number, counter) = if self.initiator {
            (
                NonceAddition::Initiator(self.initiators_counter),
                &mut self.initiators_counter,
            )
        } else {
            (
                NonceAddition::Responder(self.responders_counter),
                &mut self.responders_counter,
            )
        };
        self.decipher
            .encrypt(data, chunk_number)
            .map_err(SocketError::Encryption)
            .and_then(|v| BinaryChunk::from_content(v.as_ref()).map_err(SocketError::Chunk))
            .map(|c| {
                *counter += 1;
                c
            })
    }

    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>, SocketError> {
        let (chunk_number, counter) = if self.initiator {
            (
                NonceAddition::Responder(self.responders_counter),
                &mut self.responders_counter,
            )
        } else {
            (
                NonceAddition::Initiator(self.initiators_counter),
                &mut self.initiators_counter,
            )
        };
        self.decipher
            .decrypt(data, chunk_number)
            .map_err(SocketError::Decryption)
            .map(|c| {
                *counter += 1;
                c
            })
    }

    pub async fn write_message<T, M>(
        &mut self,
        stream: &mut T,
        messages: &[M],
    ) -> Result<(), SocketError>
    where
        T: Unpin + AsyncWriteExt,
        M: BinaryMessage,
    {
        let mut chunks = Vec::new();
        for message in messages {
            let bytes = message.as_bytes().map_err(|_| SocketError::EncodingError)?;
            for plain in bytes.chunks(CONTENT_LENGTH_MAX) {
                let chunk = self.encrypt(plain.as_ref())?;
                chunks.extend_from_slice(chunk.raw());
            }
        }
        stream
            .write_all(chunks.as_ref())
            .await
            .map_err(SocketError::Io)?;
        Ok(())
    }

    pub async fn read_chunk<T>(&mut self, stream: &mut T) -> Result<Vec<u8>, SocketError>
    where
        T: Unpin + AsyncReadExt,
    {
        let mut size_buf = [0; 2];
        stream
            .read_exact(size_buf.as_mut())
            .await
            .map_err(SocketError::Io)?;
        let size = u16::from_be_bytes(size_buf) as usize;
        let mut chunk = [0; 0x10000];
        stream
            .read_exact(&mut chunk[..size])
            .await
            .map_err(SocketError::Io)?;

        self.decrypt(&chunk[..size])
    }
}
