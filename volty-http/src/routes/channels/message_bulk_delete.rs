use std::time::{Duration, SystemTime};

use reqwest::Method;
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use validator::Validate;

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct BulkDelete {
    #[validate(length(min = 1, max = 100))]
    pub ids: Vec<String>,
}

impl Http {
    pub async fn delete_messages(
        &self,
        channel_id: impl std::fmt::Display,
        data: impl Into<BulkDelete>,
    ) -> Result<(), HttpError> {
        let mut data: BulkDelete = data.into();
        let oldest = SystemTime::now() - Duration::from_secs(7 * 24 * 60 * 60);
        data.ids.retain(|i| {
            let Ok(id) = Ulid::from_string(&i.to_string()) else {
                return false;
            };
            id.datetime() > oldest
        });
        dbg!(&data);
        data.validate()?;
        let path = format!("channels/{channel_id}/messages/bulk");
        let request = self.request(Method::DELETE, &path)?.json(&data);
        self.send_request(request).await
    }
}
