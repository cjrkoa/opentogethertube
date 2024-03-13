use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};

use futures_util::{SinkExt, StreamExt};
use ott_balancer_protocol::client::*;
use rand::{distributions::Alphanumeric, Rng};
use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

use crate::{TestRunner, WebsocketSender};

pub struct Client {
    addr: SocketAddr,
    pub(crate) stream: Option<WebSocketStream<TcpStream>>,
}

impl Client {
    pub fn new(ctx: &TestRunner) -> anyhow::Result<Self> {
        Ok(Self {
            addr: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), ctx.port()),
            stream: None,
        })
    }

    /// Connect to the balancer, targeting the given room.
    pub async fn connect(&mut self, room: impl AsRef<str>) {
        assert!(!self.connected(), "already connected");

        let mut attempts = 0;
        loop {
            match self.try_connect(room.as_ref()).await {
                Ok(stream) => {
                    self.stream = Some(stream);
                    break;
                }
                Err(e) => {
                    attempts += 1;
                    eprintln!("failed to connect, retrying [{}]: {}", attempts, e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    if attempts > 10 {
                        panic!("client failed to connect after 10 attempts");
                    }
                }
            }
        }
    }

    async fn try_connect(
        &mut self,
        room: impl AsRef<str>,
    ) -> anyhow::Result<WebSocketStream<TcpStream>> {
        let stream = tokio::net::TcpStream::connect(self.addr).await?;
        let (stream, _) = tokio_tungstenite::client_async(
            format!(
                "ws://localhost:{}/api/room/{}",
                self.addr.port(),
                room.as_ref()
            ),
            stream,
        )
        .await?;

        Ok(stream)
    }

    /// Send the auth message to the balancer.
    pub async fn auth(&mut self) {
        let auth = ClientMessage::Auth(ClientMessageAuth {
            token: rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(40)
                .map(char::from)
                .collect(),
        });

        self.send(auth).await;
    }

    /// Connect to the balancer, targeting the given room, and send the auth message.
    ///
    /// Equivalent to calling [`connect`] and [`auth`] in sequence.
    pub async fn join(&mut self, room: impl AsRef<str>) {
        self.connect(room).await;
        self.auth().await;
    }

    pub fn connected(&self) -> bool {
        if let Some(stream) = &self.stream {
            stream.get_ref().peer_addr().is_ok()
        } else {
            false
        }
    }

    pub async fn disconnect(&mut self) {
        assert!(self.connected(), "not connected");

        let mut stream = self.stream.take().unwrap();
        let _ = stream.close(None).await;
    }

    pub async fn wait_for_disconnect(&mut self) {
        if !self.connected() {
            return;
        }

        let mut stream = self.stream.take().unwrap();
        while stream.next().await.is_some() {}
    }

    /// Receive a message from the balancer. This will block until a message is received, or 200ms has passed.
    ///
    /// If it times out, the client will *not* be disconnected, and it will return `Err`. If the connection is closed,
    /// it will return `Err`. If the message is a close message, the client will be disconnected, and it will return `Ok`.
    pub async fn recv(&mut self) -> anyhow::Result<Message> {
        if let Some(stream) = self.stream.as_mut() {
            match tokio::time::timeout(Duration::from_millis(200), stream.next()).await {
                Ok(Some(Ok(msg))) => {
                    if msg.is_close() {
                        self.disconnect().await;
                    }
                    Ok(msg)
                }
                Ok(Some(Err(e))) => Err(anyhow::anyhow!(e)),
                Ok(None) => {
                    self.disconnect().await;
                    Err(anyhow::anyhow!("connection closed"))
                }
                Err(_) => {
                    self.disconnect().await;
                    Err(anyhow::anyhow!("timed out"))
                }
            }
        } else {
            Err(anyhow::anyhow!("not connected"))
        }
    }
}

#[async_trait::async_trait]
impl WebsocketSender for Client {
    async fn send_raw(&mut self, msg: Message) {
        assert!(self.connected(), "not connected");

        if let Some(stream) = self.stream.as_mut() {
            stream.send(msg).await.expect("failed to send message");
        }
    }
}
