use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::time::Duration;
use futures::{stream::Stream, sink::Sink};
use tokio_util::codec::Framed;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time;
use tokio::time::{Interval, Sleep};
use proto::codec::message::MessageCodec;
use proto::command::Command;
use proto::message::{Message, MessageContents, MessageError};
use pin_project::pin_project;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PingerError {
    #[error("ping timeout reached")]
    PingTimeout,
}


#[derive(Debug)]
#[pin_project]
struct Pinger {
    tx: UnboundedSender<Message>,
    enabled: bool,
    ping_timeout: Duration,
    #[pin]
    ping_deadline: Option<Sleep>,
    #[pin]
    ping_interval: Interval,
}

impl Pinger {
    pub fn new(tx: UnboundedSender<Message>) -> Pinger {
        let ping_time = Duration::from_secs(120);
        let ping_timeout = Duration::from_secs(30);
        let mut ret = Self {
            tx,
            enabled: true,
            ping_timeout,
            ping_deadline: None,
            ping_interval: time::interval(ping_time)
        };
        ret.ping_interval.reset();
        ret
    }

    fn handle_message(self: Pin<&mut Self>, message: &Message) -> Result<(), MessageError> {
        match &message.contents {
            MessageContents::Command(command) => match command {
                Command::PING(ref data, _) => {
                    self.send_pong(data)?;
                }
                Command::PONG(_, None) | Command::PONG(_, Some(_)) => {
                    self.project().ping_deadline.set(None);
                }
                _ => (),
            },
            _ => (),
        }
        Ok(())
    }

    fn send_pong(self: Pin<&mut Self>, data: &str) -> Result<(), MessageError> {
        self.project()
            .tx
            .send(Command::Pong(data.to_owned(), None).into())
            .map_err(|source| MessageError::SendError{ source })?;
        Ok(())
    }

    fn send_ping(self: Pin<&mut Self>) -> Result<(), MessageError> {
        //FIXME: Send Proper server address.
        let data = format!("{}", "127.0.0.1");
        let mut this = self.project();
        this.tx
            .send(Command::Ping(data.clone(), None).into())
            .map_err(|source| MessageError::SendError{ source })?;
        if this.ping_deadline.is_none() {
            let ping_deadline = time::sleep(*this.ping_timeout);
            this.ping_deadline.set(Some(ping_deadline));
        }
        Ok(())
    }
}

impl Future for Pinger {
    type Output = Result<(), MessageError>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ping_deadline) = self.as_mut().project().ping_deadline.as_pin_mut() {
            match ping_deadline.poll(cx) {
                Poll::Ready(()) => return Poll::Ready(Err(MessageError::PingTimeout)),
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
    pub fn new(inner: Framed<T, MessageCodec>, tx: UnboundedSender<Message>) -> Transport<T> {
        let pinger = Some(Pinger::new(tx));
        Transport {
            inner,
            pinger,
        }
    }

    pub fn into_inner(self) -> Framed<T, MessageCodec> {
        self.inner
    }
}

impl<T> Stream for Transport<T>
    where
        T: Unpin + AsyncRead + AsyncWrite,
{
    type Item = Result<Message, MessageError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            match pinger.poll(cx) {
                Poll::Ready(result) => result?,
                Poll::Pending => (),
            }
        }
        let result = ready!(self.as_mut().project().inner.poll_next(cx));
        let message = match result {
            None => return Poll::Ready(None),
            Some(message) => message?,
        };

        if let Some(pinger) = self.as_mut().project().pinger.as_pin_mut() {
            pinger.handle_message(&message)?;
        }
        Poll::Ready(Some(Ok(message)))
    }
}

impl<T> Sink<Message> for Transport<T>
    where
        T: Unpin + AsyncRead + AsyncWrite,
{
    type Error = MessageError;
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