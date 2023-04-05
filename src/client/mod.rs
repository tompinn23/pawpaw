use crate::proto::codec::message::MessageCodec;
use crate::proto::command::Command;
use crate::proto::error::ProtocolError;
use crate::proto::message::{Message, MessageContents};
use crate::proto::reply::Reply;
use crate::server::socket::Socket;
use crate::server::transport::{Sender, Transport};
use crate::server::{Server, ServerError};
use futures::future::FusedFuture;
use futures::stream::{FusedStream, SplitSink, SplitStream};
use futures::{ready, FutureExt, Sink, Stream, StreamExt};
use log::{debug, error};
use std::error::Error;
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use tokio_util::codec::Framed;
use uuid::Uuid;

pub mod handle;

#[derive(Debug)]
pub struct ClientStream {
    stream: SplitStream<Transport<Socket<TcpStream>>>,
    outgoing: Option<Outgoing>,
}

impl ClientStream {
    pub async fn collect(mut self) -> Result<Vec<Message>, ProtocolError> {
        let mut output = Vec::new();
        while let Some(msg) = self.next().await {
            match msg {
                Ok(m) => output.push(m),
                Err(e) => return Err(e),
            }
        }

        Ok(output)
    }
}

impl FusedStream for ClientStream {
    fn is_terminated(&self) -> bool {
        false
    }
}

impl Stream for ClientStream {
    type Item = Result<Message, ProtocolError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(outgoing) = self.as_mut().outgoing.as_mut() {
            match Pin::new(outgoing).poll(cx) {
                Poll::Ready(Ok(())) => {
                    // assure that we wake up again to check the incoming stream.
                    cx.waker().wake_by_ref();
                    //return Poll::Ready(None);
                }
                Poll::Ready(Err(e)) => {
                    cx.waker().wake_by_ref();
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Pending => (),
            }
        }

        match ready!(Pin::new(&mut self.as_mut().stream).poll_next(cx)) {
            Some(Ok(msg)) => {
                //self.state.handle_message(&msg)?;
                return Poll::Ready(Some(Ok(msg)));
            }
            other => Poll::Ready(other),
        }
    }
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error(transparent)]
    Protocol {
        #[from]
        source: ProtocolError,
    },
    #[error("client stream closed [disconnect?]")]
    StreamClosed,
}

#[derive(Debug)]
pub struct Outgoing {
    sink: SplitSink<Transport<Socket<TcpStream>>, Message>,
    stream: UnboundedReceiver<Message>,
    buffered: Option<Message>,
}

impl Outgoing {
    fn try_start_send(
        &mut self,
        cx: &mut Context<'_>,
        msg: Message,
    ) -> Poll<Result<(), ProtocolError>> {
        debug_assert!(self.buffered.is_none());
        match Pin::new(&mut self.sink).poll_ready(cx)? {
            Poll::Ready(()) => Poll::Ready(Pin::new(&mut self.sink).start_send(msg)),
            Poll::Pending => {
                self.buffered = Some(msg);
                Poll::Pending
            }
        }
    }
}

impl FusedFuture for Outgoing {
    fn is_terminated(&self) -> bool {
        false
    }
}

impl Future for Outgoing {
    type Output = Result<(), ProtocolError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        if let Some(msg) = this.buffered.take() {
            ready!(this.try_start_send(cx, msg))?
        }

        loop {
            match this.stream.poll_recv(cx) {
                Poll::Ready(Some(message)) => ready!(this.try_start_send(cx, message))?,
                Poll::Ready(None) => {
                    ready!(Pin::new(&mut this.sink).poll_flush(cx))?;
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => {
                    ready!(Pin::new(&mut this.sink).poll_flush(cx))?;
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}

pub struct Client {
    server: Arc<Server>,
    addr: SocketAddr,
    sender: Sender,
    stream: ClientStream,
    pub nick: String,
    pub hostname: String,
    pub username: String,
    pub realname: String,
    pub uuid: Uuid,
}

impl Client {
    pub fn new(sock: Socket<TcpStream>, server: Arc<Server>) -> Result<Self, ClientError> {
        let addr = match &sock {
            Socket::Plain(ref sock) => sock.peer_addr(),
            #[cfg(feature = "native-tls")]
            Socket::Tls(ref sock) => sock.get_ref().get_ref().get_ref().peer_addr(),
            #[cfg(feature = "rustls")]
            Socket::RusTls(ref sock) => {
                let (s, _) = sock.get_ref();
                s.peer_addr()
            }
        };
        let (tx_out, rx_out) = unbounded_channel();
        let sender = Sender::new(server.clone(), tx_out);
        let framed = Framed::new(sock, MessageCodec::new("utf-8")?);
        let conn = Transport::new(framed, sender.clone());
        let (outgoing, incoming) = conn.split();
        Ok(Client {
            server: server,
            addr: addr.expect("Failed to find peer address"),
            sender,
            nick: String::new(),
            hostname: String::new(),
            username: String::new(),
            realname: String::new(),
            uuid: Uuid::nil(),
            stream: ClientStream {
                stream: incoming,
                outgoing: Option::from(Outgoing {
                    sink: outgoing,
                    stream: rx_out,
                    buffered: None,
                }),
            },
        })
    }

    pub fn address(&self) -> IpAddr {
        self.addr.ip()
    }

    pub fn set_hostname(&mut self, hostname: String) {
        self.hostname = hostname;
    }

    pub fn send<T: Into<Message>>(&self, m: T) -> Result<(), ClientError> {
        self.sender.send(m).map_err(|e| e.into())
    }

    pub fn send_notice<S: Into<String>>(&self, message: S) -> Result<(), ClientError> {
        if self.nick.is_empty() {
            return self.send(Command::Notice("*".to_string(), message.into()));
        } else {
            return self.send(Command::Notice(self.nick.clone(), message.into()));
        }
    }

    pub fn send_motd(&self) -> Result<(), ClientError> {
        let motd = self.server.get_motd();
        self.send(Reply::MotdStart(self.server.name()))?;
        for line in motd {
            self.send(Reply::Motd(line.clone()))?;
        }
        self.send(Reply::MotdEnd)?;
        Ok(())
    }

    pub async fn handle_message(&mut self, message: Message) {
        if let MessageContents::Command(cmd) = message.contents {
            debug!("Handling message: {}", cmd);
            let reply = if self.uuid.is_nil() {
                match cmd {
                    Command::NICK(nick, hops) => self.handle_nick_message(nick, hops).await,
                    Command::USER(un, _, _, realname) => {
                        self.handle_user_message(un, realname).await
                    }
                    _ => Err(Reply::ErrNotRegistered),
                }
            } else {
                match cmd {
                    Command::NICK(_nick, _hops) => {
                        //self.handle_nick_message(nick, hops).await
                        Ok(())
                    }
                    Command::USER(..) => Err(Reply::ErrAlreadyRegistered),
                    Command::JOIN(chans, keys) => self.handle_join_message(chans, keys).await,
                    _ => Err(Reply::ErrGeneric(
                        cmd.name(),
                        None,
                        "Command not handled.".to_owned(),
                    )),
                }
            };
            match reply {
                Ok(_) => {}
                Err(v) => match self.send(v) {
                    Ok(_) => {}
                    Err(v) => error!(
                        "Err sending message: {} {}",
                        v,
                        v.source().map_or("".to_owned(), |v| v.to_string())
                    ),
                },
            }
        }
    }

    pub async fn handle_message_error(&mut self, err: ProtocolError) -> Option<ClientError> {
        debug!("err {:?}", err);
        match err {
            ProtocolError::UnknownCommand(cmd) => match self.send(Reply::ErrNoSuchCommand(cmd)) {
                Ok(_) => None,
                Err(e) => Some(e),
            },
            ProtocolError::NotEnoughArguments(cmd) => {
                match self.send(Reply::ErrNeedMoreParams(cmd)) {
                    Ok(_) => None,
                    Err(e) => Some(e),
                }
            }
            v => Some(v.into()),
        }
    }

    /*
       DO not call this repeatedly in a loop.
       Best used when u need to just send outgoing without expecting input.
    */
    pub async fn poll_nowait(&mut self) -> Result<(), ClientError> {
        if let Some(option) = self.stream.next().now_or_never() {
            if let Some(option) = option {
                match option {
                    Ok(msg) => {
                        self.handle_message(msg).await;
                    }
                    Err(v) => return Err(v.into()),
                }
            } else {
                return Err(ClientError::StreamClosed);
            }
        }
        Ok(())
    }

    pub async fn poll(&mut self) -> Result<(), ClientError> {
        let evt = self.stream.next().await;
        if let Some(option) = evt {
            match option {
                Ok(msg) => {
                    self.handle_message(msg).await;
                }
                Err(v) => {
                    if let Some(e) = self.handle_message_error(v).await {
                        return Err(e);
                    }
                }
            }
        } else {
            return Err(ClientError::StreamClosed);
        }
        Ok(())
    }

    pub async fn handle_nick_message(
        &mut self,
        nick: String,
        hops: Option<i32>,
    ) -> Result<(), Reply> {
        if self
            .server.state()
            .set_nick(nick.clone())
        {
            self.nick = nick;
        } else {
            return Err(Reply::ErrNickCollision(nick));
        }
        Ok(())
    }

    pub async fn handle_user_message(&mut self, un: String, realname: String) -> Result<(), Reply> {
        if !self.uuid.is_nil() {
            return Err(Reply::ErrAlreadyRegistered);
        }
        if let Some(uuid) = self
            .server.state()
            .register(self.nick.clone(), un.clone(), self.hostname.clone(), realname.clone(), self.sender.clone())
        {
            self.uuid = uuid;
            self.username = un;
            self.realname = realname;
            self.send_motd();
        } else {
            return Err(Reply::ErrGeneric(
                "USER".to_string(),
                None,
                "No UUID returned".to_string(),
            ));
        }
        Ok(())
    }

    pub async fn handle_join_message(
        &mut self,
        chans: Vec<String>,
        keys: Option<Vec<String>>,
    ) -> Result<(), Reply> {
        let replies = self.server.state().join_channel(self.uuid.clone(), chans, keys).await.map_err(|e| e.to_reply("JOIN", None))?;
        for rpl in replies {
            self.send(rpl);
        }
        return Ok(());
    }
}

impl Drop for Client {
    //TODO: Tell server clients going away.
    // Stick nick in timeout.
    fn drop(&mut self) {
        self.server.drop_client(&self.nick, &self.uuid)
    }
}
