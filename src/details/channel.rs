use crate::details::modes::ChannelMode;
use uuid::Uuid;

pub enum ChannelError {}

#[derive(Debug)]
pub struct Channel {
    clients: Vec<Uuid>,
    opers: Vec<Uuid>,
    mode: ChannelMode,
}

impl Channel {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            clients: vec![uuid],
            opers: vec![uuid],
            mode: ChannelMode::default(),
        }
    }
}
