use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::channels::message::Message;

use crate::{error::HttpError, Http};

use super::message_send::SendableEmbed;

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct MessageEdit {
    /// New message content
    #[validate(length(min = 1, max = 2000))]
    content: Option<String>,
    /// Embeds to include in the message
    #[validate(length(min = 0, max = 10))]
    embeds: Option<Vec<SendableEmbed>>,
}

impl MessageEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl std::fmt::Display) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn embed(mut self, embed: impl Into<SendableEmbed>) -> Self {
        self.embeds = Some(vec![embed.into()]);
        self
    }

    pub fn embeds<E: Into<SendableEmbed>>(mut self, embeds: impl IntoIterator<Item = E>) -> Self {
        self.embeds = Some(embeds.into_iter().map(|e| e.into()).collect());
        self
    }
}

impl From<String> for MessageEdit {
    fn from(value: String) -> Self {
        Self::new().content(value)
    }
}

impl From<&str> for MessageEdit {
    fn from(value: &str) -> Self {
        Self::new().content(value.to_string())
    }
}

impl Http {
    pub async fn edit_message(
        &self,
        channel_id: impl std::fmt::Display,
        message_id: impl std::fmt::Display,
        data: impl Into<MessageEdit>,
    ) -> Result<Message, HttpError> {
        let data: MessageEdit = data.into();
        data.validate()?;
        let path = format!("channels/{channel_id}/messages/{message_id}");
        let request = self.request(Method::PATCH, &path)?.json(&data);
        self.send_request(request).await
    }
}
