use std::io;

use thiserror::Error;
use std::result;

pub type Result<T> = result::Result<T, ProtocolError>;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    Io {
        #[from]
        source: io::Error
    },
    /* LineCodecProblems */
    #[error("Unsupported encoding {0}")]
    UnsupportedEncoding(String),
    #[error("Maximum line length exceeded")]
    MaxLineLengthExceeded,
    /* MessageParsing */
    #[error("empty message")]
    EmptyMessage,
    #[error("invalid command")]
    InvalidCommand,
    /* Command Errors */
    #[error("not enough arguments for command {0} when parsing ")]
    NotEnoughArguments(String),
    #[error("unknown command {0}")]
    UnknownCommand(String),
    #[error("command parsing error")]
    ParseError,

    /* MessageError */
    #[error("ping timeout")]
    PingTimeout,
    #[error("send error")]
    SendError,
}