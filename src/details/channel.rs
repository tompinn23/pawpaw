use std::fmt;
use std::fmt::Formatter;
use std::sync::Arc;
use crate::details::modes::ChannelMode;
use uuid::Uuid;
use crate::proto;
use crate::proto::{Prefix, Reply};
use crate::server::Server;

pub enum ChannelError {}

#[derive(Debug,Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct ChannelUser {
    uuid: Uuid,
    nick: String,
    is_oper: bool,
    chat_allowed: bool,
}

impl ChannelUser {
    pub fn new_oper(uuid: Uuid, nick: String) -> Self {
        ChannelUser {
            uuid,
            nick,
            is_oper: true,
            chat_allowed: false,
        }
    }

    pub fn new(uuid: Uuid, nick: String) -> Self {
        ChannelUser {
            uuid,
            nick,
            is_oper: false,
            chat_allowed: false,
        }
    }
}

impl fmt::Display for ChannelUser {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_oper {
            write!(f, "@")?;
        } else if self.chat_allowed {
            write!(f, "+")?;
        }
        write!(f, "{}", self.nick)
    }
}

#[derive(Debug)]
pub struct Channel {
    name: String,
    clients: Vec<ChannelUser>,
    mode: ChannelMode,
    topic: Option<String>,
}

impl Channel {
    pub fn new(name: String, uuid: Uuid, nick: String) -> Self {
        Self {
            name,
            clients: vec![ChannelUser::new_oper(uuid, nick)],
            mode: ChannelMode::default(),
            topic: None
        }
    }

    pub fn add_client(&mut self, uuid: Uuid, nick: String) {
        self.clients.push(ChannelUser::new(uuid, nick));
    }

    pub fn get_clients(&self) -> &Vec<ChannelUser> {
        &self.clients
    }

    pub fn reply_topic(&self) -> Reply {
        if let Some(topic) = &self.topic {
            Reply::Topic(self.name.clone(), topic.clone())
        } else {
            Reply::NoTopic(self.name.clone())
        }
    }

    pub fn reply_names(&self, prefix: Prefix) -> Vec<Reply> {
        let mut replies: Vec<Reply> = Vec::new();
        let prefix_length = Reply::NamReply(self.name.clone(), Vec::new()).to_string().len() + prefix.to_string().len();
        let mut formatted: String = String::new();
        let mut clients = Vec::new();
        for client in &self.clients {
            if formatted.len() + client.to_string().len() + 1 < proto::codec::message::MESSAGE_LINE_LENGTH - (prefix_length + 2) {
                formatted.push_str(&client.to_string());
                formatted.push(' ');
                clients.push(client.clone());
            } else {
                replies.push(Reply::NamReply(self.name.clone(), clients));
                clients = Vec::new();
                formatted.clear();
            }
        }
        if formatted.len() > 0 {
            replies.push(Reply::NamReply(self.name.clone(), clients));
        }
        replies.push(Reply::EndOfNames(self.name.clone()));
        replies
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn format_channel_reply() {
        let mut channel = Channel::new("test_1".to_string(), Uuid::nil(), "oper!".to_string());
        for i in 0..250 {
            channel.add_client(Uuid::nil(), format!("testy_{}", i));
        }
        for reply in channel.reply_names(Prefix::ServerOrNick("irc.tlph.one".to_string())) {
            println!("len: {}", reply.to_string().len());
            assert!(reply.to_string().len() < proto::codec::message::MESSAGE_LINE_LENGTH);
            println!("{}", reply.to_string());
        }
    }
}