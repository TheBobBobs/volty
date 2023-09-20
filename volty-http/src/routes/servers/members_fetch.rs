use reqwest::Method;
use serde::{Deserialize, Serialize};
use volty_types::{servers::server_member::Member, users::user::User};

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FetchMembersResponse {
    pub members: Vec<Member>,
    pub users: Vec<User>,
}

impl Http {
    pub async fn fetch_members(
        &self,
        server_id: impl std::fmt::Display,
    ) -> Result<FetchMembersResponse, HttpError> {
        let path = format!("servers/{server_id}/members");
        let request = self.request(Method::GET, &path)?;
        self.send_request(request).await
    }
}
