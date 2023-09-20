use reqwest::Method;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn delete_message(
        &self,
        channel_id: impl std::fmt::Display,
        message_id: impl std::fmt::Display,
    ) -> Result<(), HttpError> {
        let path = format!("channels/{channel_id}/messages/{message_id}");
        let request = self.request(Method::DELETE, &path)?;
        self.send_request(request).await
    }
}
