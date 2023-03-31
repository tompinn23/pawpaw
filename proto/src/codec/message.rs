use std::io;
use bytes::BytesMut;
use crate::codec::line::{LineCodec, LineCodecError};
use thiserror::Error;
use tokio_util::codec::{Decoder, Encoder};
use crate::message::Message;

#[cfg(ircv3)]
pub const MESSAGE_LINE_LENGTH: usize = 4096 + 512;
#[cfg(not(ircv3))]
pub const MESSAGE_LINE_LENGTH: usize = 512;

#[derive(Error, Debug)]
pub enum MessageCodecError {
    #[error("line error")]
    LineError{
        #[from]
        source: LineCodecError
    },
    #[error("LineCodec IO error")]
    Io {
    #[from ]
    source: io::Error
    },
}

#[derive(Debug)]
pub struct MessageCodec {
    inner: LineCodec
}

impl MessageCodec {
    pub fn new(label: &str) -> Result<MessageCodec, MessageCodecError> {
        let inner = LineCodec::new(label, MESSAGE_LINE_LENGTH).map_err(|e| MessageCodecError::LineError {
            source: e,
        })?;
        Ok(MessageCodec{
            inner
        })
    }
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = MessageCodecError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        todo!()
    }
}
impl Encoder<Message> for MessageCodec {
    type Error = MessageCodecError;

    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner.encode(item.to_string(),dst).map_err(|e| MessageCodecError::LineError { source: e})
    }
}