use reqwest::Method;
use serde::Deserialize;
use volty_types::{channels::channel::Channel, servers::server::Server};

use crate::{error::HttpError, Http};

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum InviteJoinResponse {
    Server {
        /// Channels in the server
        channels: Vec<Channel>,
        /// Server we are joining
        server: Server,
    },
}

impl Http {
    pub async fn join_invite(
        &self,
        code: impl std::fmt::Display,
    ) -> Result<InviteJoinResponse, HttpError> {
        let path = format!("invites/{code}");
        let request = self.request(Method::POST, &path)?;
        self.send_request(request).await
    }
}
