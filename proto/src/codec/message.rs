use std::io;
use bytes::BytesMut;
use crate::codec::line::{LineCodec};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use crate::error::ProtocolError;
use crate::message::{Message, MessageParseError};

#[cfg(ircv3)]
pub const MESSAGE_LINE_LENGTH: usize = 4096 + 512;
#[cfg(not(ircv3))]
pub const MESSAGE_LINE_LENGTH: usize = 512;

#[derive(Debug)]
pub struct MessageCodec {
    inner: LineCodec
}

impl MessageCodec {
    pub fn new(label: &str) -> Result<MessageCodec, ProtocolError> {
        let inner = LineCodec::new(label, MESSAGE_LINE_LENGTH)?;
        Ok(MessageCodec{
            inner
        })
    }
}

///
/// Codec for Framed implementing Message
/// Fatal ProtocolErrors come from the linecodec.
/// Non fatal ProtocolErrors typically generate from the Message::parse
/// Non Fatal ProtocolErrors are passed up the chain to the client for handling e.g. UnknownCommand or NotEnoughArguments etc.
impl Decoder for MessageCodec {
    /// Dumb Result return for non fatal protocol errors.
    type Item = Result<Message, ProtocolError>;
    type Error = ProtocolError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src)? {
            Some(val) => {
                let v = val.parse::<Message>();
                Ok(Some(v))
            }
            None => Ok(None)
        }
    }
}
impl Encoder<Message> for MessageCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let item = item.to_string();
        self.inner.encode(item,dst)
    }
}