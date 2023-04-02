use std::{cmp, fmt, io};
use std::fmt::{Error, format};
use bytes::{Buf, BufMut, BytesMut};
use encoding::{label::encoding_from_whatwg_label, DecoderTrap, EncoderTrap, EncodingRef, ByteWriter};
use tokio_util::codec::{Decoder, Encoder};
use thiserror::Error;
use crate::error::ProtocolError;

pub struct LineCodec {
    encoding: EncodingRef,
    next_index: usize,
    max_length: usize
}

impl LineCodec {
    pub fn new(label: &str, max_length: usize) -> Result<LineCodec, ProtocolError> {
        encoding_from_whatwg_label(label)
            .map(|enc| LineCodec {
                encoding: enc,
                next_index: 0,
                max_length
            }).ok_or_else(|| ProtocolError::UnsupportedEncoding(label.to_string()))
    }

    pub fn name(&self) -> &str { self.encoding.name() }
}

impl Decoder for LineCodec {
    type Item = String;
    type Error = ProtocolError;

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
                return Err(ProtocolError::MaxLineLengthExceeded)
            }
            None => {
                self.next_index = src.len();
                return Ok(None)
            }
        };
        let buf = src.split_to(len);
        src.advance(2);
        if buf.last().is_none() { return Ok(Some(String::new())); }
        self.next_index = 0;
        match self.encoding.decode(&buf.freeze(), DecoderTrap::Replace) {
            Ok(data) => {
                Ok(Some(data))
            }
            Err(data) => {
                Err(ProtocolError::Io {
                    source: io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("Failed to decode {} as {}.", data, self.encoding.name())
                    )
                })
            }
        }
    }
}

impl Encoder<String> for LineCodec {
    type Error = ProtocolError;

    fn encode(&mut self, item: String, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match self.encoding.encode(&item, EncoderTrap::Replace) {
            Ok(mut v) => {
                if v.len() > self.max_length - 2 {
                    return Err(ProtocolError::MaxLineLengthExceeded);
                }
                // Stop line at first \r\n
                if let Some(n) = memchr::memmem::find(&v, b"\r\n") {
                    v.truncate(n);
                }
                dst.put_slice(&v);
                dst.put_slice(b"\r\n");
                Ok(())
            }
            Err(e) => Err(ProtocolError::Io {
                source: io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Failed to encode {} as {}", e, self.encoding.name()))
            })
        }
    }
}

impl fmt::Debug for LineCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LineCodec")
            .field("encoder", &self.encoding.name())
            .field("next_index", &self.next_index)
            .field("max_length", &self.next_index)
            .finish()
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