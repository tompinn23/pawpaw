use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::oneshot::{channel, Receiver, Sender};
use uuid::Uuid;
use crate::server::state::ServerStateCommand::NickCheck;

pub enum ServerStateCommand {
    NickCheck {
        nick: String,
        tx: Sender<bool>
    }
}

impl ServerStateCommand {
    pub fn nick_check(nick: &str) -> (Self, Receiver<bool>) {
        let (tx, rx) = channel();
        (NickCheck {
            nick: nick.to_owned(),
            tx
        }, rx)
    }
}

pub struct ServerState {
    rx: UnboundedReceiver<ServerStateCommand>,
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
}