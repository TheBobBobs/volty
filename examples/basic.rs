use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use volty::prelude::*;

pub struct Bot {
    pub http: Http,
    pub ws: WebSocket,
    pub cache: Cache,
}

#[async_trait]
impl RawHandler for Bot {
    async fn on_ready(
        &self,
        _users: Vec<User>,
        _servers: Vec<Server>,
        _channels: Vec<Channel>,
        _members: Vec<Member>,
        _emojis: Vec<Emoji>,
    ) {
        let user = self.cache.user().await;
        println!("Ready as {}#{}", user.username, user.discriminator);
    }

    async fn on_message(&self, message: Message) {
        if message.author_id == self.cache.user_id() {
            return;
        }
        let Some(content) = message.content else {
            return;
        };
        if content == "!ping" {
            if let Err(e) = self.ws.send_typing(&message.channel_id).await {
                dbg!(e);
            }
            sleep(Duration::from_secs(2)).await;
            if let Err(e) = self.http.send_message(&message.channel_id, "pong!").await {
                dbg!(e);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().unwrap();

    let token = std::env::var("BOT_TOKEN").expect("Missing Env Variable: BOT_TOKEN");
    let http = Http::new(&token, true);
    let ws = WebSocket::connect(&token).await;
    let cache = Cache::new();

    let bot = Bot {
        http,
        ws: ws.clone(),
        cache: cache.clone(),
    };
    let handler = Arc::new(bot);

    loop {
        let event = ws.next().await;
        cache.update(event.clone()).await;
        let h = handler.clone();
        tokio::spawn(async move {
            h.on_event(event).await;
        });
    }
}
