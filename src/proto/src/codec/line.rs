use std::{cmp, io};
use std::io::BufRead;
use bytes::{Buf, BufMut, BytesMut};
use encoding::{label::encoding_from_whatwg_label, EncoderTrap, EncodingRef};
use tokio_util::codec::{Decoder, Encoder};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LineCodecError {
    #[error("The encoding {} is not supported")]
    UnsupportedEncoding(String),
    #[error("LineCodec IO error")]
    Io {
        #[from ]
        source: io::Error
    }
}

pub struct LineCodec {
    encoding: EncodingRef,
    next_index: usize,
    max_length: usize
}

impl LineCodec {
    pub fn new(label: &str, max_length: i32) -> Result<LineCodec, LineCodecError> {
        encoding_from_whatwg_label(label)
            .map(|enc| LineCodec {
                encoding: enc,
                next_index: 0,
                max_length
            }).ok_or_else(|| LineCodecError::UnsupportedEncoding(label.to_string()))
    }

    pub fn name(&self) -> &str { self.encoding.name() }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = LineCodecError;

    fn decode(&mut self, src: &mut BytesMuf) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let read_to = cmp::min(self.max_length.saturating_add(1), src.len());
        let mut len = match memchr::memmem::find(&src[self.next_index..read_to], b"\r\n") {
            Some(n) => n,
            None if src.len() > self.max_length => {
                src.clear();
                self.next_index = 0;
                return Err(LineCodecError::MaxLineLengthExceeded)
            }
            None => {
                self.next_index = src.len();
                return Ok(None)
            }
        };
        let mut buf = src.split_to(len);
        src.advance(2);
        match buf.last(){
            None => return Ok(Some(String::new())),
            _ => {},
        }
        self.next_index = 0;
        match self.encoding.decode(&buf.freeze(), DecoderTrap::Replace) {
            Ok(data) => {
                Ok(Some(data))
            }
            Err(data) => {
                return Err(LineCodecError::Io {
                    source: io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Failed to decode {} as {}.", data, self.encoding.name())
                    )
                });
            }
        }
    }
}

impl Encoder<String> for LineCodec {

}