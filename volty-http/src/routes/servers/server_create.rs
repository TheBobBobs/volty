use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::{channels::channel::Channel, servers::server::Server};

use crate::{error::HttpError, Http};

/// # Server Data
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
pub struct CreateServer {
    /// Server name
    #[validate(length(min = 1, max = 32))]
    pub name: String,
    /// Server description
    #[validate(length(min = 0, max = 1024))]
    pub description: Option<String>,
    /// Whether this server is age-restricted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
}

impl CreateServer {
    pub fn new(name: impl std::fmt::Display) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            nsfw: None,
        }
    }

    pub fn description(mut self, description: impl std::fmt::Display) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn nsfw(mut self) -> Self {
        self.nsfw = Some(true);
        self
    }
}

/// # Create Server Response
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateServerResponse {
    /// Server object
    pub server: Server,
    /// Default channels
    pub channels: Vec<Channel>,
}

impl Http {
    pub async fn create_server(
        &self,
        server: impl Into<CreateServer>,
    ) -> Result<CreateServerResponse, HttpError> {
        let server: CreateServer = server.into();
        server.validate()?;
        let request = self.request(Method::POST, "servers/create")?.json(&server);
        self.send_request(request).await
    }
}
