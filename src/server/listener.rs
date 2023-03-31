use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use thiserror::Error;
#[cfg(feature = "native-tls")]
use tokio_native_tls::TlsAcceptor;
#[cfg(feature = "rustls")]
use tokio_rustls::TlsAcceptor;

use log::{debug, info};

use crate::server::socket::Socket;


#[derive(Debug, Error)]
pub enum ListenerError {
    #[error("connection error")]
    ConnectionError {
        #[from]
        source: io::Error,
    },
    #[cfg(feature = "native-tls")]
    #[error("tls error")]
    TlsError {
        #[from]
        source: tokio_native_tls::native_tls::Error
    },
    #[cfg(feature = "rustls")]
    #[error("tls error")]
    TlsError {
        #[from]
        source: tokio_rustls::rustls::Error
    }
}

#[derive(Debug)]
pub enum Listener {
    #[cfg(feature = "native-tls")]
    Tls(String, TcpListener, TlsAcceptor),
    #[cfg(feature = "rustls")]
    Tls(TcpListener, TlsAcceptor),
    Plain(String, TcpListener)
}

impl Listener {
    pub async fn new(name: String, addr: SocketAddr) -> Result<Listener, io::Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Listener::Plain(name, listener))
    }

    #[cfg(feature = "native-tls")]
    pub async fn new_tls(name: String, addr: SocketAddr, tls: TlsAcceptor) -> Result<Listener, io::Error> {
        let listener = TcpListener::bind(addr).await?;
        Ok(Listener::Tls(name,listener, tls))
    }

    #[cfg(rustls)]
    pub async fn new_tls(addr: SocketAddr, tls: TlsAcceptor) -> Result<Listener, io::Error> {
        unimplemented!();
        let listener = TcpListener::bind(addr).await?;
        Ok(Listener::Tls(listener, tls))
    }

    pub async fn accept(&self) -> Result<Socket<TcpStream>, ListenerError> {
        match self {
            Listener::Plain(ref name, ref acceptor) => {
                let (socket, addr) = acceptor.accept().await.map_err(|source| ListenerError::ConnectionError { source })?;
                debug!("ACCEPTOR(plain): {} accepted a connection from: {}", name, addr);
                Ok(Socket::Plain(socket))
            }
            #[cfg(feature = "native-tls")]
            Listener::Tls(ref name, ref listen, ref accept) => {
                let (socket, addr) = listen.accept().await.map_err(|source| ListenerError::ConnectionError { source })?;
                let stream = accept.accept(socket).await.map_err(|source| ListenerError::TlsError { source })?;
                debug!("ACCEPTOR(native-tls): {} accepted a connection from: {}", name, addr);
                Ok(Socket::Tls(stream))
            }
            #[cfg(feature = "rustls")]
            Listener::Tls(ref name, ref listen, ref accept) => {
                let (socket, addr) = listen.accept().await.map_err(|source| ListenerError::TlsError { source })?;
                debug!("ACCEPTOR(rustls): {} accepted a connection from: {}", name, addr);
                unimplemented!();
            }
        }
    }
}