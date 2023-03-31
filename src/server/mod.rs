use std::io;
use std::sync::Arc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use futures::sink::Sink;
use tokio::fs::read;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::oneshot::Sender;
use trust_dns_resolver::TokioAsyncResolver;
use proto::prefix::Prefix;
use crate::config::{Config, ListenConfig};
use crate::server::state::{ServerState, ServerStateCommand};

#[cfg(feature = "native-tls")]
use tokio_native_tls::{native_tls::Identity, TlsAcceptor};
use crate::client::{Client, ClientError};

use crate::server::listener::Listener;
use crate::server::socket::Socket;


pub mod socket;
pub mod transport;

mod state;
mod cached;
mod listener;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("listener {0} is missing tls certificate")]
    TLSMissingCert(String),
    #[error("listener {0} is missing tls key")]
    TLSMissingKey(String),
    #[error("server IO error")]
    Io {
        #[from]
        source: io::Error
    },
    #[cfg(feature = "native-tls")]
    #[error("server TLS error")]
    NativeTLSError {
        #[from]
        source: tokio_native_tls::native_tls::Error
    },
    #[cfg(feature = "rustls")]
    #[error("server TLS error")]
    RusTLSError {
        #[from]
        source: tokio_rustls::rustls::Error
    },
    #[error("sever control: send error")]
    ControlSendError,
    #[error("server control: recv error")]
    ControlRecvError {
        #[from]
        source: tokio::sync::oneshot::error::RecvError
    }
}


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

impl Server {
    pub async fn new(config: Config) -> Result<Server, ServerError> {
        let resolver = TokioAsyncResolver::tokio_from_system_conf().expect("Failed to create DNS resolver.");
        let (tx, rx) = unbounded_channel();
        let mut server = Self {
            resolver,
            listeners: Vec::new(),
            prefix: Prefix::Server(config.hostname),
            state: Some(ServerState::new(rx)),
            tx,
            phase: ServerPhase::Starting
        };
        for (name, listener) in config.listeners {
            if listener.tls {
                #[cfg(any(feature = "native-tls", feature = "rustls"))]
                server.add_tls_listener(name, listener).await?;
                #[cfg(not(all(feature = "native-tls", feature = "rustls")))]
                panic!("Tried to add TLS listener with no TLS library enabled.");
            } else {
                server.add_listener(name, listener).await?;
            }
        }
        Ok(server)
    }

    #[cfg(any(feature = "native-tls", feature = "rustls"))]
    pub async fn add_tls_listener(&mut self, name: String, listener: ListenConfig) -> Result<(), ServerError> {
        let cert = if let Some(cert) = listener.tls_cert {
            read(cert).await.map_err(|source| ServerError::Io{ source })?
        } else {
            return Err(ServerError::TLSMissingCert(name))
        };
        let key = if let Some(key) = listener.tls_key {
            read(key).await.map_err(|source| ServerError::Io { source })?
        } else {
            return Err(ServerError::TLSMissingKey(name))
        };
        #[cfg(feature = "native-tls")]
        let ident = Identity::from_pkcs8(&cert, &key).map_err(|source| ServerError::NativeTLSError { source })?;
        let acceptor = TlsAcceptor::from(
            tokio_native_tls::native_tls::TlsAcceptor::builder(ident).build()?
        );
        let listener = Listener::new_tls(name,listener.address, acceptor).await.map_err(|source| ServerError::Io { source })?;
        self.listeners.push(listener);
        return Ok(());
        #[cfg(feature = "rustls")]
        todo!();
    }

    pub async fn add_listener(&mut self, name: String, listener: ListenConfig) -> Result<(), ServerError> {
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
                    Err(e) => eprintln!("{}", e)
                }
            }
        };
        Client::new(conn, self.clone())
    }

    pub async fn nick_exists(&self, nick: &str) -> Result<bool, ServerError> {
        let (cmd, rx) = ServerStateCommand::nick_check(nick);
        self.tx.send(cmd).map_err(|_| ServerError::ControlSendError)?;
        rx.await.map_err(|source| ServerError::ControlRecvError { source })
    }
}