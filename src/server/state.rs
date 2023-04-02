use std::collections::{HashMap, HashSet};
use log::debug;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot::{channel, Receiver, Sender};
use uuid::Uuid;
use proto::message::Message;
use crate::client::handle::ClientHandle;
use crate::server::state::ServerStateCommand::{NickCheck, Register, SetNick};

pub enum ServerStateCommand {
    NickCheck {
        nick: String,
        tx: Sender<bool>
    },
    SetNick {
        nick: String,
        tx: Sender<bool>
    },
    Register {
        nick: String,
        un: String,
        peer_address: String,
        real: String,
        client_tx: UnboundedSender<Message>,
        tx: Sender<Option<Uuid>>
    },
    DropClient {
        nick: String,
        uuid: Uuid,
    }
}
type ServerCommand<T> = (ServerStateCommand, Receiver<T>);

impl ServerStateCommand {

    pub fn set_nick(nick: &str) -> ServerCommand<bool> {
        let (tx, rx) = channel();
        (SetNick {
            nick: nick.to_owned(),
            tx
        }, rx)
    }

    pub fn nick_check(nick: &str) -> ServerCommand<bool> {
        let (tx, rx) = channel();
        (NickCheck {
            nick: nick.to_owned(),
            tx
        }, rx)
    }

    pub fn register(nick: &str, un: &str, peer: &str, realname: &str, client_tx: UnboundedSender<Message>) -> ServerCommand<Option<Uuid>> {
        let (tx, rx) = channel();
        (Register {
            nick: nick.to_string(),
            un: un.to_string(),
            peer_address: peer.to_string(),
            real: realname.to_string(),
            tx,
            client_tx
        }, rx)
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
    pub(crate) rx: UnboundedReceiver<ServerStateCommand>,
    clients: HashMap<Uuid, ClientHandle>,
    nicks: HashSet<String>,
}

impl ServerState {
    pub fn new(rx: UnboundedReceiver<ServerStateCommand>) -> Self {
        ServerState {
            rx,
            clients: HashMap::new(),
            nicks: HashSet::new(),
        }
    }

    pub fn contains_nick(&mut self, nick: &String) -> bool {
        self.nicks.contains(nick)
    }

    pub fn set_nick(&mut self, nick: String) -> bool {
        self.nicks.insert(nick)
    }

    pub fn register(&mut self, nick: String, un: String, peer: String, real: String, tx: UnboundedSender<Message>) -> Option<Uuid> {
        let handle = ClientHandle::new(nick, un, peer, real, tx);
        let uuid = Uuid::new_v4();
        self.clients.insert(uuid, handle);
        Some(uuid)
    }

    pub fn drop_client(&mut self, nick: String, uuid: Uuid) {
        if nick.is_empty() || uuid.is_nil() {
            return;
        }
        debug!("client with nick: {} uuid: {} is being dropped", nick, uuid);
        self.nicks.remove(&nick);
        self.clients.remove(&uuid);
    }
}