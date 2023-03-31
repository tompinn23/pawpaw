use tokio::sync::mpsc::UnboundedSender;
use trust_dns_resolver::TokioAsyncResolver;
use proto::prefix::Prefix;
use crate::server::state::{ServerState, ServerStateCommand};

pub mod socket;
pub mod transport;

mod state;
mod cached;

enum ServerPhase {
    Starting,
    Running,
    Stopping
}

pub struct Server {
    /* Optional too allow a mutable take once */
    state: Option<ServerState>,
    prefix: Prefix,
    resolver: TokioAsyncResolver,
    listeners: Vec<Listener>,
    tx: UnboundedSender<ServerStateCommand>,
    phase: ServerPhase
}