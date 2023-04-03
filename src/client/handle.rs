use crate::proto::message::Message;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug)]
pub struct ClientHandle {
    nick: String,
    un: String,
    hostname: String,
    real: String,
    tx: UnboundedSender<Message>,
}

impl ClientHandle {
    pub fn new(
        nick: String,
        un: String,
        hostname: String,
        real: String,
        tx: UnboundedSender<Message>,
    ) -> Self {
        ClientHandle {
            nick,
            un,
            hostname,
            real,
            tx,
        }
    }
}
