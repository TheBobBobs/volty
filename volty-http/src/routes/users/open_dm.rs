use reqwest::Method;
use volty_types::channels::channel::Channel;

use crate::{error::HttpError, Http};

impl Http {
    pub async fn open_dm(&self, user_id: &str) -> Result<Channel, HttpError> {
        let path = format!("users/{user_id}/dm");
        let request = self.request(Method::GET, &path)?;
        self.send_request(request).await
    }
}
