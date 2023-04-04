use crate::server::transport::Sender;
use dashmap::DashSet;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct ServerClient {
    nickname: RwLock<String>,
    user_name: String,
    server_name: String,
    hostname: String,
    sender: Sender,
    connected_channels: DashSet<String>,
}

impl ServerClient {
    pub fn new(
        nick: String,
        user_name: String,
        server_name: String,
        hostname: String,
        sender: Sender,
    ) -> Self {
        Self {
            nickname: RwLock::new(nick),
            user_name,
            server_name,
            hostname,
            sender,
            connected_channels: DashSet::new(),
        }
    }

    pub async fn set_nickname(&self, nick: String) {
        let mut n = self.nickname.write().await;
        *n = nick;
    }
    pub fn join_channel(&self, channel: &String) -> bool {
        if self.connected_channels.contains(channel) {
            return false;
        }
        self.connected_channels.insert(channel.clone());
        true
    }

    pub fn remove_channel(&self, channel: &String) -> bool {
        self.connected_channels.remove(channel);
        true
    }
}
