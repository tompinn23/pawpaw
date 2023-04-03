use crate::proto::Command;
use crate::proto::MessageCodec;
use crate::proto::ProtocolError;
use crate::proto::{Message, MessageContents};
use crate::server::Server;
use futures::{sink::Sink, stream::Stream};
use pin_project::pin_project;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{ready, Context, Poll};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;
use tokio::time::{Interval, Sleep};
use tokio_util::codec::Framed;

#[derive(Clone, Debug)]
pub struct Sender {
    server: Arc<Server>,
    sender: UnboundedSender<Message>,
}

impl Sender {
    pub fn send<M: Into<Message>>(&self, msg: M) -> Result<(), ProtocolError> {
        let mut m = msg.into();
        m.set_prefix(self.server.prefix());
        self.sender.send(m).map_err(|_| ProtocolError::SendError)?;
        Ok(())
    }

    pub fn new(server: Arc<Server>, sender: UnboundedSender<Message>) -> Self {
        Self { server, sender }
    }

    pub fn tx(&self) -> UnboundedSender<Message> {
        self.sender.clone()
    }

    fn server(&self) -> &Arc<Server> {
        &self.server
    }
}

#[derive(Debug, Error)]
pub enum PingerError {
    #[error("ping timeout reached")]
    PingTimeout,
}

#[derive(Debug)]
#[pin_project]
struct Pinger {
    tx: Sender,
    enabled: bool,
    ping_timeout: Duration,
    #[pin]
    ping_deadline: Option<Sleep>,
    #[pin]
    ping_interval: Interval,
}

impl Pinger {
    pub fn new(tx: Sender) -> Pinger {
        let ping_time = Duration::from_secs(120);
        let ping_timeout = Duration::from_secs(30);
        let mut ret = Self {
            tx,
            enabled: true,
            ping_timeout,
            ping_deadline: None,
            ping_interval: time::interval(ping_time),
        };
        ret.ping_interval.reset();
        ret
    }

    fn handle_message(self: Pin<&mut Self>, message: &Message) -> Result<bool, ProtocolError> {
        if let MessageContents::Command(command) = &message.contents {
            match command {
                Command::PING(ref data, _) => {
                    self.send_pong(data)?;
                    return Ok(true);
                }
                Command::PONG(_, None) | Command::PONG(_, Some(_)) => {
                    self.project().ping_deadline.set(None);
                    return Ok(true);
                }
                _ => (),
            }
        }
        Ok(false)
    }

    fn send_pong(self: Pin<&mut Self>, data: &str) -> Result<(), ProtocolError> {
        self.project()
            .tx
            .send(Command::Pong(data.to_owned(), None))
            .map_err(|_| ProtocolError::SendError)?;
        Ok(())
    }

    fn send_ping(self: Pin<&mut Self>) -> Result<(), ProtocolError> {
        let mut this = self.project();
        this.tx
            .send(Command::Ping(this.tx.server().name(), None))
            .map_err(|_| ProtocolError::SendError)?;
        if this.ping_deadline.is_none() {
            let ping_deadline = time::sleep(*this.ping_timeout);
            this.ping_deadline.set(Some(ping_deadline));
        }
        Ok(())
    }
}

impl Future for Pinger {
    type Output = Result<(), ProtocolError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ping_deadline) = self.as_mut().project().ping_deadline.as_pin_mut() {
            match ping_deadline.poll(cx) {
                Poll::Ready(()) => return Poll::Ready(Err(ProtocolError::PingTimeout)),
                Poll::Pending => (),
            }
        }
        if let Poll::Ready(_) = self.as_mut().project().ping_interval.poll_tick(cx) {
            if *self.as_mut().project().enabled {
                self.as_mut().send_ping()?;
            }
        }
        Poll::Pending
    }
}

#[derive(Debug)]
#[pin_project]
pub struct Transport<T> {
    #[pin]
    inner: Framed<T, MessageCodec>,
    #[pin]
    pinger: Option<Pinger>,
}

impl<T> Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    pub fn new(inner: Framed<T, MessageCodec>, tx: Sender) -> Transport<T> {
        let pinger = Some(Pinger::new(tx));
        Transport { inner, pinger }
    }

    pub fn into_inner(self) -> Framed<T, MessageCodec> {
        self.inner
    }
}

impl<T> Stream for Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Item = Result<Message, ProtocolError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            match pinger.poll(cx) {
                Poll::Ready(result) => result?,
                Poll::Pending => (),
            }
        }
        let result: Option<Result<Result<Message, ProtocolError>, ProtocolError>> =
            ready!(self.as_mut().project().inner.poll_next(cx));
        let message: Message = match result {
            None => return Poll::Ready(None),
            Some(message) => match message {
                Ok(msg) => match msg {
                    Ok(v) => v,
                    Err(v) => return Poll::Ready(Some(Err(v))),
                },
                Err(v) => return Poll::Ready(Some(Err(v))),
            },
        };

        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            if pinger.handle_message(&message)? {
                return Poll::Pending;
            }
        }
        Poll::Ready(Some(Ok(message)))
    }
}

impl<T> Sink<Message> for Transport<T>
where
    T: Unpin + AsyncRead + AsyncWrite,
{
    type Error = ProtocolError;
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_ready(cx))?;
        Poll::Ready(Ok(()))
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.project().inner.start_send(item)?;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_flush(cx))?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(self.project().inner.poll_close(cx))?;
        Poll::Ready(Ok(()))
    }
}
