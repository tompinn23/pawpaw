#[derive(Debug)]
pub struct ChannelMode {
    private: bool,
    secret: bool,
    invite_only: bool,
    topic_oper_only: bool,
    no_outside_messages: bool,
    limit: i32,
    //TODO: Replace this with some kind of ban mask type.
    ban_mask: Vec<String>,
    key: Option<String>,
}

#[derive(Debug)]
pub struct UserMode {
    invisible: bool,
    srv_notices: bool,
    wallops: bool,
    oper: bool,
}

impl ChannelMode {
    pub fn default() -> Self {
        ChannelMode {
            private: false,
            secret: false,
            invite_only: false,
            topic_oper_only: true,
            no_outside_messages: true,
            limit: 0,
            ban_mask: Vec::new(),
            key: None,
        }
    }
}
