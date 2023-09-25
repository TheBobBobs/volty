use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::servers::server_ban::ServerBan;

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct MemberBan {
    /// Ban reason
    #[validate(length(min = 1, max = 1024))]
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl MemberBan {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reason(mut self, reason: impl std::fmt::Display) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

impl Http {
    pub async fn ban_member(
        &self,
        server_id: impl std::fmt::Display,
        user_id: impl std::fmt::Display,
        data: impl Into<MemberBan>,
    ) -> Result<ServerBan, HttpError> {
        let data = data.into();
        data.validate()?;
        let path = format!("servers/{server_id}/bans/{user_id}");
        let request = self.request(Method::PUT, &path)?.json(&data);
        self.send_request(request).await
    }
}
