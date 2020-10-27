use std::{mem, convert::TryFrom};
use tokio::{
    net::TcpStream,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tezos_messages::p2p::{
    encoding::{
        connection::ConnectionMessage, version::NetworkVersion, metadata::MetadataMessage,
        ack::AckMessage,
    },
    binary_message::{BinaryMessage, BinaryChunk},
};
use tezos_conversation::Identity;
use slog::Logger;
use super::{error::SocketError, decipher_state::DecipherState};

pub enum HandshakeState {
    Connection,
    Metadata(DecipherState),
    Acknowledge(DecipherState),
    Finish(DecipherState, AckMessage),
    Awaiting,
}

impl HandshakeState {
    pub async fn run(
        &mut self,
        logger: &Logger,
        stream: &mut TcpStream,
    ) -> Result<(), SocketError> {
        let current_state = mem::replace(self, HandshakeState::Awaiting);
        let new_state = match current_state {
            HandshakeState::Connection => {
                let decipher = outgoing_connection(stream).await?;

                slog::info!(logger, "exchanged connection messages");
                HandshakeState::Metadata(decipher)
            },
            HandshakeState::Metadata(mut decipher) => {
                let m = MetadataMessage::new(false, false);
                decipher.write_message(stream, &[m]).await?;
                let _ = decipher.read_chunk(stream).await?;

                slog::info!(logger, "exchanged metadata messages");
                HandshakeState::Acknowledge(decipher)
            },
            HandshakeState::Acknowledge(mut decipher) => {
                decipher.write_message(stream, &[AckMessage::Ack]).await?;
                let data = decipher.read_chunk(stream).await?;
                let ack = AckMessage::from_bytes(data).map_err(|_| SocketError::DecodingError)?;

                slog::info!(logger, "exchanged acknowledge messages");
                HandshakeState::Finish(decipher, ack)
            },
            HandshakeState::Finish(decipher, ack) => HandshakeState::Finish(decipher, ack),
            HandshakeState::Awaiting => HandshakeState::Awaiting,
        };
        let _ = mem::replace(self, new_state);
        Ok(())
    }
}

async fn outgoing_connection(stream: &mut TcpStream) -> Result<DecipherState, SocketError> {
    let identity = Identity::from_path("identity.json".to_string()).unwrap();

    let chain_name = "TEZOS_ALPHANET_CARTHAGE_2019-11-28T13:02:13Z".to_string();
    let version = NetworkVersion::new(chain_name, 0, 1);
    let connection_message = ConnectionMessage {
        port: 0,
        versions: vec![version],
        public_key: identity.public_key(),
        proof_of_work_stamp: identity.proof_of_work(),
        message_nonce: vec![0; 24],
    };
    let chunk = connection_message
        .as_bytes()
        .map_err(|_| SocketError::EncodingError)?;
    let initiator_chunk = BinaryChunk::from_content(chunk.as_ref()).unwrap();
    stream
        .write_all(initiator_chunk.raw())
        .await
        .map_err(SocketError::Io)?;

    let mut size_buf = [0; 2];
    stream
        .read_exact(size_buf.as_mut())
        .await
        .map_err(SocketError::Io)?;
    let size = u16::from_be_bytes(size_buf) as usize;
    let mut chunk = vec![0; size + 2];
    chunk[..2].clone_from_slice(size_buf.as_ref());
    stream
        .read_exact(&mut chunk[2..])
        .await
        .map_err(SocketError::Io)?;
    let responder_chunk = BinaryChunk::try_from(chunk).unwrap();

    let decipher = identity
        .decipher(initiator_chunk.raw(), responder_chunk.raw())
        .ok()
        .unwrap();
    Ok(DecipherState::new(decipher, true))
}
