use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use futures::future::FusedFuture;
use futures::stream::{FusedStream, SplitSink, SplitStream};
use futures::{ready, Sink, Stream, StreamExt};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use tokio_util::codec::Framed;
use uuid::Uuid;
use proto::codec::message::{MessageCodec, MessageCodecError};
use proto::command::Command;
use proto::message::{Message, MessageContents, MessageError};
use proto::reply::Reply;
use crate::server::Server;
use crate::server::socket::Socket;
use crate::server::transport::Transport;

#[derive(Debug)]
pub struct ClientStream {
    stream: SplitStream<Transport<Socket<TcpStream>>>,
    outgoing: Option<Outgoing>,
}

impl ClientStream {
    pub async fn collect(mut self) -> Result<Vec<Message>, MessageError> {
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
    type Item = Result<Message, MessageError>;

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
    #[error("message codec error")]
    MessageCodecError {
        #[from]
        source: MessageCodecError
    }
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
    ) -> Poll<Result<(), MessageError>> {
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
    type Output = Result<(), MessageError>;

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

pub struct ClientData {
    pub nick: String,
    pub uuid: Uuid,
}

impl ClientData {
    pub fn new() -> ClientData {
        Self {
            nick: String::new(),
            uuid: Uuid::nil(),
        }
    }
}

pub struct Client {
    server: Arc<Server>,
    addr: SocketAddr,
    data: ClientData,
    sender: UnboundedSender<Message>,
    stream: ClientStream
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
        let (tx_out, rx_out) = mpsc::unbounded_channel();
        let framed = Framed::new(sock, MessageCodec::new("utf-8")?);
        let conn = Transport::new(framed, tx_out.clone());
        let (outgoing, incoming) = conn.split();
        Ok(Client {
            server,
            addr: addr.expect("Failed to find peer address"),
            sender: tx_out.clone(),
            data: ClientData::new(),
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

    pub async fn handle_message(&mut self, message: Message) {
        if let MessageContents::Command(cmd) = message.contents {
            match cmd {
                Command::NICK(nick,hops) => {
                    self.handle_nick_message(nick, hops);
                }
                _ => todo!()
            }
        }
    }

    pub async fn poll(&mut self) {
        if let Some(option) = self.stream.next().await {
            match option {
                Ok(msg) => {
                    self.handle_message(msg);
                }
                Err(v) => eprintln!("{}", v),
            }
        }
    }


    pub async fn handle_nick_message(&mut self, nick: String, hops: Option<i32>) -> Result<(), Reply> {
        if self.server.nick_exists(&nick) {
            Err(Reply::ErrNickCollision(nick))
        }
        Ok(())
    }
}