use std::sync::Arc;

use volty::prelude::*;

pub struct Bot {
    pub http: Http,
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
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().unwrap();

    let token = std::env::var("BOT_TOKEN").expect("Missing Env Variable: BOT_TOKEN");
    let http = Http::new(&token, true);
    let mut ws = WebSocket::connect(&token).await;
    let cache = Cache::new();

    let bot = Bot {
        http,
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
