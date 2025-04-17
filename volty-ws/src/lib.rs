pub use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::ops::Deref;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::{self, Bytes};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use volty_types::ws::{client::ClientMessage, common::Ping, server::ServerMessage};

mod cache;
pub use cache::{Cache, UpdateCache};

mod handler;
pub use handler::RawHandler;

type WsRX = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
type WsTX = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>;

async fn retrying_connect(url: &str) -> (WsTX, WsRX) {
    loop {
        match connect_async(url).await {
            Ok((stream, _)) => {
                return stream.split();
            }
            Err(e) => {
                log::error!("Retrying: {:?}", e);
                sleep(Duration::from_secs(5)).await;
            }
        };
    }
}

fn parse_msg(message: tungstenite::Message) -> Option<ServerMessage> {
    if let tungstenite::Message::Binary(bytes) = message {
        match rmp_serde::from_slice::<ServerMessage>(&bytes) {
            Ok(msg) => Some(msg),
            Err(e) => {
                log::error!("Parse: {:?}", e);
                None
            }
        }
    } else {
        None
    }
}

const HEARTBEAT: u128 = 30_000;

#[derive(Clone)]
pub struct WebSocket {
    inner: Arc<InnerWebSocket>,
}

impl Deref for WebSocket {
    type Target = InnerWebSocket;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub struct InnerWebSocket {
    url: String,

    tx: Mutex<WsTX>,
    rx: Mutex<WsRX>,
    last_message: Mutex<Instant>,
}

impl WebSocket {
    pub async fn connect(token: impl std::fmt::Display) -> Self {
        const DEFAULT_WS_URL: &str = "wss://ws.revolt.chat";
        Self::connect_with_url(DEFAULT_WS_URL, token).await
    }

    pub async fn connect_with_url(
        ws_url: impl std::fmt::Display,
        token: impl std::fmt::Display,
    ) -> Self {
        let url = format!("{}/?format=msgpack&token={}", &ws_url, &token);
        let (tx, rx) = retrying_connect(&url).await;
        let inner = InnerWebSocket {
            url,
            tx: Mutex::new(tx),
            rx: Mutex::new(rx),
            last_message: Mutex::new(Instant::now()),
        };
        Self {
            inner: Arc::new(inner),
        }
    }

    async fn update_last_message(&self) {
        let mut last = self.last_message.lock().await;
        *last = Instant::now();
    }

    async fn reconnect(&self) {
        let (tx, rx) = retrying_connect(&self.url).await;
        let mut t = self.tx.lock().await;
        *t = tx;
        let mut r = self.rx.lock().await;
        *r = rx;
        self.update_last_message().await;
    }

    async fn check_error(&self, error: tungstenite::Error) -> Result<(), tungstenite::Error> {
        log::error!("Check: {:?}", &error);
        use tungstenite::Error;
        match error {
            Error::ConnectionClosed
            | Error::AlreadyClosed
            | Error::Io(_)
            | Error::Tls(_)
            | Error::Protocol(_) => {
                sleep(Duration::from_secs(5)).await;
                self.reconnect().await;
                Ok(())
            }
            // TODO
            e => panic!("{:?}", e),
        }
    }

    pub async fn next(&self) -> ServerMessage {
        loop {
            let last_ms = self.last_message.lock().await.elapsed().as_millis();
            if last_ms >= HEARTBEAT * 2 {
                self.reconnect().await;
            }

            let last_ms = self.last_message.lock().await.elapsed().as_millis();
            let millis = HEARTBEAT - last_ms.min(HEARTBEAT);
            let duration = Duration::from_millis(millis as u64);
            let heartbeat = sleep(duration);

            // TODO might skip messages when select! cancels
            let mut rx = self.rx.lock().await;
            let next = rx.next();

            select! {
                _ = heartbeat => {
                    drop(rx);
                    if let Err(e) = self.send_ping().await {
                        self.check_error(e).await.unwrap();
                    }
                }
                message = next => {
                    drop(rx);
                    if let Some(msg) = self.handle_message(message).await {
                        log::debug!("Received event: {:?}", &msg);
                        return msg;
                    }
                }
            };
        }
    }

    async fn handle_message(
        &self,
        message: Option<Result<tungstenite::Message, tungstenite::Error>>,
    ) -> Option<ServerMessage> {
        self.update_last_message().await;
        match message {
            Some(result) => match result {
                Ok(message) => parse_msg(message),
                Err(e) => {
                    self.check_error(e).await.unwrap();
                    None
                }
            },
            None => {
                self.reconnect().await;
                None
            }
        }
    }

    pub async fn send(&self, message: &ClientMessage) -> Result<(), tungstenite::Error> {
        log::debug!("Sending message: {:?}", message);
        let bytes = Bytes::from(
            rmp_serde::to_vec_named(message).expect("ClientMessage failed to serialize"),
        );
        let item = tungstenite::Message::Binary(bytes);
        let mut result = self.tx.lock().await.send(item.clone()).await;
        self.update_last_message().await;
        while let Err(e) = result {
            self.check_error(e).await?;
            result = self.tx.lock().await.send(item.clone()).await;
            self.update_last_message().await;
        }
        result
    }

    pub async fn send_ping(&self) -> Result<(), tungstenite::Error> {
        let message = ClientMessage::Ping {
            data: Ping::Number(0),
            responded: None,
        };
        self.send(&message).await
    }

    pub async fn send_typing(
        &self,
        channel_id: impl std::fmt::Display,
    ) -> Result<(), tungstenite::Error> {
        self.send(&ClientMessage::BeginTyping {
            channel: channel_id.to_string(),
        })
        .await
    }

    pub async fn send_end_typing(
        &self,
        channel_id: impl std::fmt::Display,
    ) -> Result<(), tungstenite::Error> {
        self.send(&ClientMessage::EndTyping {
            channel: channel_id.to_string(),
        })
        .await
    }
}
