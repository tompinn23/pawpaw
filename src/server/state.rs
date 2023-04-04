use crate::client::handle::ClientHandle;
use crate::details::{Channel, ChannelError};
use crate::proto::{Message, Reply};
use crate::server::state::ServerStateCommand::{JoinChannel, NickCheck, Register, SetNick};
use crate::server::{transport, Server};
use dashmap::{DashMap, DashSet};
use log::debug;
use std::collections::{HashMap, HashSet};
use std::iter::zip;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{channel, Receiver, Sender};
use uuid::Uuid;

pub enum ServerStateCommand {
    NickCheck {
        nick: String,
        tx: Sender<bool>,
    },
    SetNick {
        nick: String,
        tx: Sender<bool>,
    },
    Register {
        nick: String,
        un: String,
        peer_address: String,
        real: String,
        client_tx: UnboundedSender<Message>,
        tx: Sender<Option<Uuid>>,
    },
    JoinChannel {
        uuid: Uuid,
        chans: Vec<String>,
        keys: Option<Vec<String>>,
        tx: Sender<Result<(), ChannelError>>,
    },
    DropClient {
        nick: String,
        uuid: Uuid,
    },
}
type ServerCommand<T> = (ServerStateCommand, Receiver<T>);

impl ServerStateCommand {
    pub fn set_nick(nick: &str) -> ServerCommand<bool> {
        let (tx, rx) = channel();
        (
            SetNick {
                nick: nick.to_owned(),
                tx,
            },
            rx,
        )
    }

    pub fn nick_check(nick: &str) -> ServerCommand<bool> {
        let (tx, rx) = channel();
        (
            NickCheck {
                nick: nick.to_owned(),
                tx,
            },
            rx,
        )
    }

    pub fn join_channel(
        uuid: &Uuid,
        chans: Vec<String>,
        keys: Option<Vec<String>>,
    ) -> ServerCommand<Result<(), ChannelError>> {
        let (tx, rx) = channel();
        (
            JoinChannel {
                uuid: *uuid,
                chans,
                keys,
                tx,
            },
            rx,
        )
    }
    pub fn register(
        nick: &str,
        un: &str,
        peer: &str,
        realname: &str,
        client_tx: UnboundedSender<Message>,
    ) -> ServerCommand<Option<Uuid>> {
        let (tx, rx) = channel();
        (
            Register {
                nick: nick.to_string(),
                un: un.to_string(),
                peer_address: peer.to_string(),
                real: realname.to_string(),
                tx,
                client_tx,
            },
            rx,
        )
    }

    pub fn drop_client(nick: &str, uuid: &Uuid) -> ServerStateCommand {
        ServerStateCommand::DropClient {
            nick: nick.to_owned(),
            uuid: *uuid,
        }
    }
}

#[derive(Debug)]
pub struct ServerState {
    //pub(crate) rx: UnboundedReceiver<ServerStateCommand>,
    clients: DashMap<Uuid, ClientHandle>,
    nicks: DashSet<String>,
    channels: DashMap<String, Channel>,
}

impl ServerState {
    pub fn new() -> Self {
        ServerState {
            clients: DashMap::new(),
            nicks: DashSet::new(),
            channels: DashMap::new(),
        }
    }

    pub fn contains_nick(&self, nick: &String) -> bool {
        self.nicks.contains(nick)
    }

    pub fn set_nick(&self, nick: String) -> bool {
        self.nicks.insert(nick)
    }

    pub fn join_channel(
        &self,
        uuid: Uuid,
        chans: Vec<String>,
        keys: Option<Vec<String>>,
    ) -> Result<Reply, ChannelError> {
        for channel in chans {
            if let Some(channel) = self.channels.get_mut(&channel) {
                return Ok(Reply::ErrAlreadyRegistered);
            } else {
                let val = Channel::new(uuid);
                self.channels.insert(channel, val);
                return Ok(Reply::ErrAlreadyRegistered);
            }
        }
        Ok(Reply::ErrAlreadyRegistered)
    }

    pub fn register(
        &self,
        nick: String,
        un: String,
        peer: String,
        real: String,
        tx: transport::Sender,
    ) -> Option<Uuid> {
        let handle = ClientHandle::new(nick, un, peer, real, tx);
        let uuid = Uuid::new_v4();
        self.clients.insert(uuid, handle);
        Some(uuid)
    }

    pub fn get_channel_users(&self, server: &Arc<Server>, channel: &str) -> Vec<Reply> {
        let preamble = server.prefix.to_string().len() + " xxx :".len();
        if let Some(channel) = self.channels.get(channel) {
            let channel = channel.value();
            let clients = channel.get_clients();
            while let Some(client) = channel.get_clients().next() {}
        }
        vec![Reply::EndOfNames(channel.to_string())]
    }

    pub fn drop_client(&self, nick: String, uuid: Uuid) {
        if nick.is_empty() || uuid.is_nil() {
            return;
        }
        debug!("client with nick: {} uuid: {} is being dropped", nick, uuid);
        self.nicks.remove(&nick);
        self.clients.remove(&uuid);
    }
}
