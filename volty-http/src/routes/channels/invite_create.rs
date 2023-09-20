use reqwest::Method;
use volty_types::channels::channel_invite::Invite;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn create_invite(
        &self,
        channel_id: impl std::fmt::Display,
    ) -> Result<Invite, HttpError> {
        let path = format!("channels/{channel_id}/invites");
        let request = self.request(Method::POST, &path)?;
        self.send_request(request).await
    }
}
