use std::fmt;
use tokio::net::TcpStream;
use slog::Logger;
use tezos_messages::p2p::binary_message::BinaryMessage;
use super::{error::SocketError, decipher_state::DecipherState, read_message_state::ReadMessageState};

pub struct TrustedConnection<M>
where
    M: BinaryMessage + fmt::Debug,
{
    reader: ReadMessageState<M>,
    decipher: DecipherState,
    stream: TcpStream,
    logger: Logger,
}

impl<M> TrustedConnection<M>
where
    M: BinaryMessage + fmt::Debug,
{
    pub fn new(stream: TcpStream, decipher: DecipherState, logger: &Logger) -> Self {
        TrustedConnection {
            reader: ReadMessageState::new(),
            decipher: decipher,
            stream: stream,
            logger: logger.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn transmute<Mx>(self) -> TrustedConnection<Mx>
    where
        Mx: BinaryMessage + fmt::Debug,
    {
        TrustedConnection {
            reader: ReadMessageState::new(),
            decipher: self.decipher,
            stream: self.stream,
            logger: self.logger,
        }
    }

    pub async fn read(&mut self) -> Result<M, SocketError> {
        let &mut TrustedConnection {
            ref mut reader,
            ref mut decipher,
            ref mut stream,
            ref logger,
        } = self;
        reader.read_message(logger, stream, decipher).await
    }

    pub async fn write(&mut self, messages: &[M]) -> Result<(), SocketError> {
        let &mut TrustedConnection {
            reader: _,
            ref mut decipher,
            ref mut stream,
            ref logger,
        } = self;
        slog::debug!(logger, "-> {:x?}", messages);
        decipher.write_message(stream, messages).await
    }
}
