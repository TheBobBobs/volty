use reqwest::Method;
use volty_types::servers::server_member::Member;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn fetch_member(
        &self,
        server_id: impl std::fmt::Display,
        user_id: impl std::fmt::Display,
    ) -> Result<Member, HttpError> {
        let path = format!("servers/{server_id}/members/{user_id}");
        let request = self.request(Method::GET, &path)?;
        self.send_request(request).await
    }
}
