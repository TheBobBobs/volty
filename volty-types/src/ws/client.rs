use serde::{Deserialize, Serialize};

use super::common::Ping;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Authenticate { token: String },
    BeginTyping { channel: String },
    EndTyping { channel: String },
    Ping { data: Ping, responded: Option<()> },
}
