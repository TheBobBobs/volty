use reqwest::Method;
use volty_types::channels::message::Message;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn fetch_message(
        &self,
        channel_id: impl std::fmt::Display,
        message_id: impl std::fmt::Display,
    ) -> Result<Message, HttpError> {
        let path = format!("channels/{channel_id}/messages/{message_id}");
        let request = self.request(Method::GET, &path)?;
        self.send_request(request).await
    }
}
