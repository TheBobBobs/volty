use reqwest::Method;
use volty_types::users::user::User;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn fetch_user(&self, user_id: impl std::fmt::Display) -> Result<User, HttpError> {
        let path = format!("users/{user_id}");
        let request = self.request(Method::GET, &path)?;
        self.send_request(request).await
    }
}
