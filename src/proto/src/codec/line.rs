use encoding::{label::encoding_from_whatwg_label, EncoderTrap, EncodingRef};

pub enum LineCodecError {

}

pub struct LineCodec {
    //encoding: EncodingRef,
    next_index: usize,
    max_length: usize
}

impl LineCodec {
    pub fn new(label: &str) -> Result<LineCodec, LineCodecError> {
        encoding_from_whatwg_label(label)
    }
}