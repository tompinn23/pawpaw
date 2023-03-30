use std::{cmp, io};
use bytes::{Buf, BufMut, BytesMut};
use encoding::{label::encoding_from_whatwg_label, DecoderTrap, EncoderTrap, EncodingRef, ByteWriter};
use tokio_util::codec::{Decoder, Encoder};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LineCodecError {
    #[error("The encoding {0} is not supported")]
    UnsupportedEncoding(String),
    #[error("LineCodec IO error")]
    Io {
        #[from ]
        source: io::Error
    },
    #[error("Maximum line length exceeded")]
    MaxLineLengthExceeded
}

pub struct LineCodec {
    encoding: EncodingRef,
    next_index: usize,
    max_length: usize
}

impl LineCodec {
    pub fn new(label: &str, max_length: usize) -> Result<LineCodec, LineCodecError> {
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

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let read_to = cmp::min(self.max_length.saturating_add(1), src.len());
        let len = match memchr::memmem::find(&src[self.next_index..read_to], b"\r\n") {
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
        let buf = src.split_to(len);
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

struct LineBytesMut<'a>(&'a mut BytesMut);

impl<'a> ByteWriter for LineBytesMut<'a> {
    fn write_byte(&mut self, b: u8) {
        self.0.put_u8(b);
    }

    fn write_bytes(&mut self, v: &[u8]) {
        self.0.put_slice(v);
    }
}

impl Encoder<String> for LineCodec {
    type Error = LineCodecError;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut bytes = LineBytesMut(dst);
        self.encoding.encode_to(item.as_str(), EncoderTrap::Replace, &mut bytes).or;
        dst.put_slice( b"\r\n");
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use bytes::BufMut;
    use super::*;

    #[test]
    fn create_utf8() {
        let dec = LineCodec::new("utf-8", 512);
        assert!(dec.is_ok())
    }

    #[test]
    fn utf8_decode() {
        let mut input = BytesMut::new();
        input.put_slice(b"Hello World\r\n");
        let mut dec = LineCodec::new("utf-8", 512).expect("Failed to construct utf-8 line decoder");
        let out = dec.decode(&mut input);
        assert!(out.is_ok());
        let out = out.unwrap();
        assert_eq!(out, Some("Hello World".to_string()));

    }

    #[test]
    fn create_cp437() {
        let dec = LineCodec::new("cp437", 512);
        assert!(dec.is_ok())
    }
}