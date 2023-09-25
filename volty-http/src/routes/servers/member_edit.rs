use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::{
    servers::server_member::{FieldsMember, Member},
    types::Timestamp,
};

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct MemberEdit {
    /// Member nickname
    #[validate(length(min = 1, max = 32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    nickname: Option<String>,

    /// Attachment Id to set for avatar
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar: Option<String>,

    /// Array of role ids
    #[serde(skip_serializing_if = "Option::is_none")]
    roles: Option<Vec<String>>,

    /// Timestamp this member is timed out until
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout: Option<Timestamp>,

    /// Fields to remove from channel object
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<FieldsMember>>,
}

impl MemberEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nickname(mut self, nickname: impl std::fmt::Display) -> Self {
        self.nickname = Some(nickname.to_string());
        self
    }

    pub fn avatar(mut self, avatar: impl std::fmt::Display) -> Self {
        self.avatar = Some(avatar.to_string());
        self
    }

    pub fn roles<S: std::fmt::Display>(mut self, roles: impl IntoIterator<Item = S>) -> Self {
        self.roles = Some(roles.into_iter().map(|s| s.to_string()).collect());
        self
    }

    pub fn timeout(mut self, timeout: impl Into<Timestamp>) -> Self {
        self.timeout = Some(timeout.into());
        self
    }

    pub fn remove(mut self, remove: impl Into<FieldsMember>) -> Self {
        let remove = remove.into();
        if let Some(r) = &mut self.remove {
            r.push(remove);
        } else {
            self.remove = Some(vec![remove])
        }
        self
    }
}

impl Http {
    pub async fn edit_member(
        &self,
        server_id: impl std::fmt::Display,
        user_id: impl std::fmt::Display,
        data: impl Into<MemberEdit>,
    ) -> Result<Member, HttpError> {
        let data: MemberEdit = data.into();
        data.validate()?;
        let path = format!("servers/{server_id}/members/{user_id}");
        let request = self.request(Method::PATCH, &path)?.json(&data);
        self.send_request(request).await
    }
}
