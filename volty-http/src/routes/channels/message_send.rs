use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::{
    channels::message::{Interactions, Masquerade, Message, Reply},
    util::regex::RE_COLOUR,
};

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct SendableEmbed {
    #[validate(length(min = 1, max = 128))]
    pub icon_url: Option<String>,
    pub url: Option<String>,
    #[validate(length(min = 1, max = 100))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 2000))]
    pub description: Option<String>,
    pub media: Option<String>,
    #[validate(length(min = 1, max = 128), regex = "RE_COLOUR")]
    pub colour: Option<String>,
}

impl SendableEmbed {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn icon_url(mut self, icon_url: impl std::fmt::Display) -> Self {
        self.icon_url = Some(icon_url.to_string());
        self
    }

    pub fn url(mut self, url: impl std::fmt::Display) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn title(mut self, title: impl std::fmt::Display) -> Self {
        self.title = Some(title.to_string());
        self
    }

    pub fn description(mut self, description: impl std::fmt::Display) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn media(mut self, media: impl std::fmt::Display) -> Self {
        self.media = Some(media.to_string());
        self
    }

    pub fn colour(mut self, colour: impl std::fmt::Display) -> Self {
        self.colour = Some(colour.to_string());
        self
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct SendableMessage {
    /// Message content to send
    #[validate(length(min = 0, max = 2000))]
    pub content: Option<String>,
    /// Attachments to include in message
    #[validate(length(min = 1, max = 128))]
    pub attachments: Option<Vec<String>>,
    /// Messages to reply to
    pub replies: Option<Vec<Reply>>,
    /// Embeds to include in message
    ///
    /// Text embed content contributes to the content length cap
    #[validate(length(min = 1, max = 10))]
    pub embeds: Option<Vec<SendableEmbed>>,
    /// Masquerade to apply to this message
    #[validate]
    pub masquerade: Option<Masquerade>,
    /// Information about how this message should be interacted with
    pub interactions: Option<Interactions>,
}

impl SendableMessage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn content(mut self, content: impl std::fmt::Display) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn attachment(mut self, attachment: impl std::fmt::Display) -> Self {
        self.attachments = Some(vec![attachment.to_string()]);
        self
    }

    pub fn attachments<S: std::fmt::Display>(
        mut self,
        attachments: impl IntoIterator<Item = S>,
    ) -> Self {
        self.attachments = Some(attachments.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn reply(mut self, reply: impl Into<Reply>) -> Self {
        self.replies = Some(vec![reply.into()]);
        self
    }

    pub fn replies<R: Into<Reply>>(mut self, replies: impl IntoIterator<Item = R>) -> Self {
        self.replies = Some(replies.into_iter().map(|r| r.into()).collect());
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

    pub fn masquerade(mut self, masquerade: impl Into<Masquerade>) -> Self {
        self.masquerade = Some(masquerade.into());
        self
    }

    pub fn interactions(mut self, interactions: impl Into<Interactions>) -> Self {
        self.interactions = Some(interactions.into());
        self
    }
}

impl From<String> for SendableMessage {
    fn from(value: String) -> Self {
        Self::new().content(value)
    }
}

impl From<&str> for SendableMessage {
    fn from(value: &str) -> Self {
        Self::new().content(value.to_string())
    }
}

impl Http {
    pub async fn send_message(
        &self,
        channel_id: impl std::fmt::Display,
        message: impl Into<SendableMessage>,
    ) -> Result<Message, HttpError> {
        let data: SendableMessage = message.into();
        data.validate()?;
        let path = format!("channels/{channel_id}/messages");
        let request = self.request(Method::POST, &path)?.json(&data);
        self.send_request(request).await
    }
}
