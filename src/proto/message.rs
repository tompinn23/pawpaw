use std::fmt;
use std::fmt::Formatter;

use std::str::FromStr;
use thiserror::Error;

use super::command::Command;
use super::error::ProtocolError;
use super::prefix::Prefix;
use super::reply::Reply;

#[non_exhaustive]
#[derive(Clone, PartialEq, Debug)]
pub enum MessageContents {
    Command(Command),
    Reply(Reply),
}

impl<'a> From<&'a MessageContents> for String {
    fn from(value: &'a MessageContents) -> Self {
        match value {
            MessageContents::Command(cmd) => cmd.to_string(),
            MessageContents::Reply(rep) => rep.to_string(),
        }
    }
}

impl fmt::Display for MessageContents {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out: String = self.into();
        write!(f, "{}", out)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Message {
    pub prefix: Option<Prefix>,
    pub contents: MessageContents,
}

impl Message {
    pub fn new(
        prefix: Option<&str>,
        command: &str,
        args: Vec<&str>,
    ) -> Result<Message, ProtocolError> {
        Ok(Message {
            prefix: prefix.map(|p| p.into()),
            contents: MessageContents::Command(Command::new(command, args)?),
        })
    }

    pub fn set_prefix(&mut self, prefix: Prefix) {
        self.prefix = Some(prefix);
    }

    pub fn prefix_from_str(&mut self, prefix: &str) {
        self.prefix = Some(Prefix::from(prefix));
    }
}

impl From<Command> for Message {
    fn from(value: Command) -> Self {
        Message {
            prefix: None,
            contents: MessageContents::Command(value),
        }
    }
}

impl From<Reply> for Message {
    fn from(value: Reply) -> Self {
        Message {
            prefix: None,
            contents: MessageContents::Reply(value),
        }
    }
}

impl<'a> From<&'a Message> for String {
    fn from(value: &'a Message) -> Self {
        let mut buf = String::new();
        if let Some(prefix) = &value.prefix {
            buf.push_str(&prefix.to_string());
            buf.push(' ');
        }
        buf.push_str(&value.contents.to_string());
        buf
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let out: String = self.into();
        write!(f, "{}", out)
    }
}

#[derive(Error, Debug)]
pub enum MessageParseError {
    #[error("empty message")]
    EmptyMessage,
    #[error("invalid command")]
    InvalidCommand,
    #[error("parse failed: {0}")]
    ParseFailed(String),
}

impl FromStr for Message {
    type Err = ProtocolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(ProtocolError::EmptyMessage);
        }

        let mut state = s;
        let prefix = if state.starts_with(':') {
            let prefix = state.find(' ').map(|i| &state[1..i]);
            state = state.find(' ').map_or("", |i| &state[i + 1..]);
            prefix
        } else {
            None
        };
        let suffix = if state.contains(" :") {
            let suffix = state.find(" :").map(|i| &state[i + 2..]);
            state = state.find(" :").map_or("", |i| &state[..i + 1]);
            suffix
        } else {
            None
        };

        let command = match state.find(' ').map(|i| &state[..i]) {
            Some(cmd) => {
                state = state.find(' ').map_or("", |i| &state[i + 1..]);
                cmd
            }
            // If there's no arguments but the "command" starts with colon, it's not a command.
            None if state.starts_with(':') => {
                return Err(ProtocolError::InvalidCommand);
            }
            // If there's no arguments following the command, the rest of the state is the command.
            None => {
                let cmd = state;
                state = "";
                cmd
            }
        };

        let mut args: Vec<_> = state.splitn(14, ' ').filter(|s| !s.is_empty()).collect();
        if let Some(suffix) = suffix {
            args.push(suffix);
        }

        Message::new(prefix, command, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_message() {
        let str = ":localhost NOTICE * :*** Hello World.";
        let message = Message {
            prefix: Some(Prefix::ServerOrNick("localhost".to_string())),
            contents: MessageContents::Command(Command::NOTICE(
                "*".to_string(),
                "*** Hello World.".to_string(),
            )),
        };
        assert_eq!(str.parse::<Message>().unwrap(), message);
    }
}
