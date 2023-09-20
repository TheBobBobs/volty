pub use async_trait::async_trait;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::select;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite;
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
    if let tungstenite::Message::Text(text) = message {
        match serde_json::from_str::<ServerMessage>(&text) {
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

pub struct WebSocket {
    url: String,

    tx: WsTX,
    rx: WsRX,
    last_message: Instant,
}

impl WebSocket {
    pub async fn connect(ws_url: &str, token: &str) -> Self {
        let url = format!("{}/?format=json&token={}", &ws_url, &token);
        let (tx, rx) = retrying_connect(&url).await;
        Self {
            url,
            tx,
            rx,
            last_message: Instant::now(),
        }
    }

    async fn reconnect(&mut self) {
        let (tx, rx) = retrying_connect(&self.url).await;
        self.tx = tx;
        self.rx = rx;
        self.last_message = Instant::now();
    }

    async fn check_error(&mut self, error: tungstenite::Error) -> Result<(), tungstenite::Error> {
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

    pub async fn next(&mut self) -> ServerMessage {
        loop {
            if self.last_message.elapsed().as_millis() >= HEARTBEAT * 2 {
                self.reconnect().await;
            }

            let millis = HEARTBEAT - self.last_message.elapsed().as_millis().min(HEARTBEAT);
            let duration = Duration::from_millis(millis as u64);
            let heartbeat = sleep(duration);

            // TODO might skip messages when select! cancels
            let next = self.rx.next();

            select! {
                _ = heartbeat => {
                    if let Err(e) = self.send_ping().await {
                        self.check_error(e).await.unwrap();
                    }
                }
                message = next => {
                    if let Some(msg) = self.handle_message(message).await {
                        log::debug!("Received event: {:?}", &msg);
                        return msg;
                    }
                }
            };
        }
    }

    async fn handle_message(
        &mut self,
        message: Option<Result<tungstenite::Message, tungstenite::Error>>,
    ) -> Option<ServerMessage> {
        self.last_message = Instant::now();
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

    pub async fn send(&mut self, message: &ClientMessage) -> Result<(), tungstenite::Error> {
        log::debug!("Sending message: {:?}", message);
        let text = serde_json::to_string(message).expect("ClientMessage failed to serialize");
        let item = tungstenite::Message::Text(text);
        let mut result = self.tx.send(item.clone()).await;
        self.last_message = Instant::now();
        while let Err(e) = result {
            self.check_error(e).await?;
            result = self.tx.send(item.clone()).await;
            self.last_message = Instant::now();
        }
        result
    }

    async fn send_ping(&mut self) -> Result<(), tungstenite::Error> {
        log::info!("Sending ping");
        let message = ClientMessage::Ping {
            data: Ping::Number(0),
            responded: None,
        };
        self.send(&message).await
    }
}
