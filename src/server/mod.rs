use crate::config::{Config, ListenConfig};
use crate::proto::Prefix;
use crate::server::state::{ServerState, ServerStateCommand};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use std::io;
use std::sync::Arc;
use thiserror::Error;
use tokio::fs::read;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tokio::task::JoinHandle;
use trust_dns_resolver::TokioAsyncResolver;

use crate::client::{Client, ClientError};
use crate::proto::Message;
use crate::proto::Reply;
#[cfg(feature = "native-tls")]
use tokio_native_tls::{native_tls::Identity, TlsAcceptor};
use uuid::Uuid;

use crate::server::listener::Listener;

pub mod socket;
pub mod transport;

mod client;
mod listener;
mod state;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("listener {0} is missing tls certificate")]
    TLSMissingCert(String),
    #[error("listener {0} is missing tls key")]
    TLSMissingKey(String),
    #[error("server IO error")]
    Io {
        #[from]
        source: io::Error,
    },
    #[cfg(feature = "native-tls")]
    #[error("server TLS error")]
    NativeTLSError {
        #[from]
        source: tokio_native_tls::native_tls::Error,
    },
    #[cfg(feature = "rustls")]
    #[error("server TLS error")]
    RusTLSError {
        #[from]
        source: tokio_rustls::rustls::Error,
    },
    #[error("sever control send error")]
    ControlSendError,
    #[error("server control recv error {source}")]
    ControlRecvError {
        #[from]
        source: tokio::sync::oneshot::error::RecvError,
    },
    #[error("the uuid doesn't have a client")]
    InvalidUUID,
}

impl ServerError {
    pub fn to_reply(&self, cmd: &str, subcmds: Option<Vec<String>>) -> Reply {
        if cfg!(debug_assertions) {
            Reply::ErrGeneric(cmd.to_owned(), subcmds, self.to_string())
        } else {
            //TODO: Add some kind of reference to server logging.
            Reply::ErrGeneric(
                cmd.to_owned(),
                subcmds,
                "Internal Server Error, contact an administrator for assistance.".to_owned(),
            )
        }
    }
}

#[derive(Debug)]
enum ServerPhase {
    Starting,
    Running,
    Stopping,
}

#[derive(Debug)]
pub struct Server {
    /* Optional too allow a mutable take once */
    state: Arc<ServerState>,
    prefix: Prefix,
    hostname: String,
    motd: Vec<String>,
    resolver: TokioAsyncResolver,
    listeners: Vec<Listener>,
    tx: UnboundedSender<ServerStateCommand>,
    phase: ServerPhase,
}

impl Server {
    pub async fn new(config: Config) -> Result<Server, ServerError> {
        let resolver =
            TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create DNS resolver.");
        let (tx, rx) = unbounded_channel();
        let mut server = Self {
            resolver,
            listeners: Vec::new(),
            motd: config.motd.split("\n").map(|x| x.to_string()).collect(),
            hostname: config.hostname.clone(),
            prefix: Prefix::ServerOrNick(config.hostname.clone()),
            state: Arc::new(ServerState::new(Prefix::ServerOrNick(config.hostname))),
            tx,
            phase: ServerPhase::Starting,
        };
        for (name, listener) in config.listeners {
            if listener.tls {
                #[cfg(any(feature = "native-tls", feature = "rustls"))]
                server.add_tls_listener(name, listener).await?;
                #[cfg(all(not(feature = "native-tls"), not(feature = "rustls")))]
                panic!("Tried to add TLS listener with no TLS library enabled.");
            } else {
                server.add_listener(name, listener).await?;
            }
        }
        Ok(server)
    }

    pub fn prefix(&self) -> Prefix {
        self.prefix.clone()
    }
    pub fn resolver(&self) -> TokioAsyncResolver {
        self.resolver.clone()
    }

    pub fn get_motd(&self) -> &Vec<String> {
        &self.motd
    }

    pub fn name(&self) -> String {
        self.hostname.clone()
    }

    pub fn state(&self) -> Arc<ServerState> {
        self.state.clone()
    }

    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    pub async fn add_tls_listener(
        &mut self,
        name: String,
        listener: ListenConfig,
    ) -> Result<(), ServerError> {
        let cert = if let Some(cert) = listener.tls_cert {
            read(cert)
                .await
                .map_err(|source| ServerError::Io { source })?
        } else {
            return Err(ServerError::TLSMissingCert(name));
        };
        let key = if let Some(key) = listener.tls_key {
            read(key)
                .await
                .map_err(|source| ServerError::Io { source })?
        } else {
            return Err(ServerError::TLSMissingKey(name));
        };
        #[cfg(feature = "native-tls")]
        let ident = Identity::from_pkcs8(&cert, &key)
            .map_err(|source| ServerError::NativeTLSError { source })?;
        let acceptor =
            TlsAcceptor::from(tokio_native_tls::native_tls::TlsAcceptor::builder(ident).build()?);
        let listener = Listener::new_tls(name, listener.address, acceptor)
            .await
            .map_err(|source| ServerError::Io { source })?;
        self.listeners.push(listener);
        return Ok(());
        #[cfg(feature = "rustls")]
        todo!();
    }

    pub async fn add_listener(
        &mut self,
        name: String,
        listener: ListenConfig,
    ) -> Result<(), ServerError> {
        let listener = Listener::new(name, listener.address).await?;
        self.listeners.push(listener);
        Ok(())
    }

    pub async fn accept(self: &Arc<Self>) -> Result<Client, ClientError> {
        let mut futs: FuturesUnordered<_> = self.listeners.iter().map(|l| l.accept()).collect();
        let conn = loop {
            if let Some(c) = futs.next().await {
                match c {
                    Ok(val) => break val,
                    Err(e) => eprintln!("{}", e),
                }
            }
        };
        Client::new(conn, self.clone())
    }

    pub async fn server_loop(&mut self) -> JoinHandle<Result<(), ServerError>> {
        // let mut state = self
        //     .state
        //     .take()
        //     .expect("Failed to obtain mutable server state");
        tokio::spawn(async move {
            // while let Some(evt) = state.rx.recv().await {
            //     match evt {
            //         ServerStateCommand::SetNick { nick, tx } => {
            //             if state.contains_nick(&nick) {
            //                 tx.send(false).map_err(|_| ServerError::ControlSendError)?;
            //             } else {
            //                 state.set_nick(nick);
            //                 tx.send(true).map_err(|_| ServerError::ControlSendError)?;
            //             }
            //         }
            //         ServerStateCommand::Register {
            //             nick,
            //             un,
            //             peer_address,
            //             real,
            //             client_tx,
            //             tx,
            //         } => {
            //             tx.send(state.register(nick, un, peer_address, real, client_tx))
            //                 .map_err(|_| ServerError::ControlSendError)?;
            //         }
            //         ServerStateCommand::JoinChannel {
            //             uuid,
            //             chans,
            //             keys,
            //             tx,
            //         } => tx.send(state.join_channel(uuid, chans, keys)),
            //         ServerStateCommand::DropClient { nick, uuid } => {
            //             state.drop_client(nick, uuid);
            //         }
            //         _ => {}
            //     }
            // }
            Ok(())
        })
    }

    pub async fn nick_exists(&self, nick: &str) -> Result<bool, ServerError> {
        let (cmd, rx) = ServerStateCommand::nick_check(nick);
        self.tx
            .send(cmd)
            .map_err(|_| ServerError::ControlSendError)?;
        rx.await
            .map_err(|source| ServerError::ControlRecvError { source })
    }

    pub async fn set_nick(&self, nick: &str) -> Result<bool, ServerError> {
        let (cmd, rx) = ServerStateCommand::set_nick(nick);
        self.tx
            .send(cmd)
            .map_err(|_| ServerError::ControlSendError)?;
        rx.await
            .map_err(|source| ServerError::ControlRecvError { source })
    }

    pub async fn register(
        &self,
        nick: &str,
        un: &str,
        peer: &str,
        realname: &str,
        client_tx: UnboundedSender<Message>,
    ) -> Result<Option<Uuid>, ServerError> {
        let (cmd, rx) = ServerStateCommand::register(nick, un, peer, realname, client_tx);
        self.tx
            .send(cmd)
            .map_err(|_| ServerError::ControlSendError)?;
        rx.await
            .map_err(|source| ServerError::ControlRecvError { source })
    }

    pub async fn join_channel(
        &self,
        uuid: &Uuid,
        chans: Vec<String>,
        keys: Option<Vec<String>>,
    ) -> Result<(), ServerError> {
        let (cmd, rx) = ServerStateCommand::join_channel(uuid, chans, keys);
        self.tx
            .send(cmd)
            .map_err(|_| ServerError::ControlSendError)?;
        match rx.await {
            Ok(_) => Ok(()),
            Err(e) => Ok(()),
        }
    }

    pub fn drop_client(&self, nick: &str, uuid: &Uuid) {
        self.state().drop_client(nick.to_string() ,uuid.clone());
    }
}
