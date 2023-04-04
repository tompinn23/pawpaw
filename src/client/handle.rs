use crate::proto::message::Message;
use crate::server::transport::Sender;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ClientHandle {
    nick: String,
    un: String,
    hostname: String,
    real: String,
    conn_channels: i32,
    tx: Sender,
}

impl ClientHandle {
    pub fn new(nick: String, un: String, hostname: String, real: String, tx: Sender) -> Self {
        ClientHandle {
            nick,
            un,
            hostname,
            real,
            conn_channels: 0,
            tx,
        }
    }
}
