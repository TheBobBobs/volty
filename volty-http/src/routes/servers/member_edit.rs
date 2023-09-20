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
    nickname: Option<String>,
    /// Attachment Id to set for avatar
    avatar: Option<String>,
    /// Array of role ids
    roles: Option<Vec<String>>,
    /// Timestamp this member is timed out until
    timeout: Option<Timestamp>,
    /// Fields to remove from channel object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsMember>>,
}

impl MemberEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn nickname(mut self, nickname: impl Into<String>) -> Self {
        self.nickname = Some(nickname.into());
        self
    }

    pub fn avatar(mut self, avatar: impl Into<String>) -> Self {
        self.avatar = Some(avatar.into());
        self
    }

    pub fn roles(mut self, roles: impl Into<Vec<String>>) -> Self {
        self.roles = Some(roles.into());
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
