use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::{
    users::user::{FieldsUser, Presence, User, UserStatus},
    util::regex::RE_DISPLAY_NAME,
};

use crate::{error::HttpError, Http};

/// # Profile Data
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct UserProfileData {
    /// Text to set as user profile description
    #[validate(length(min = 0, max = 2000))]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    /// Attachment Id for background
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(length(min = 1, max = 128))]
    background: Option<String>,
}

/// # User Data
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct UserEdit {
    /// New display name
    #[validate(length(min = 2, max = 32), regex = "RE_DISPLAY_NAME")]
    display_name: Option<String>,
    /// Attachment Id for avatar
    #[validate(length(min = 1, max = 128))]
    avatar: Option<String>,

    /// New user status
    #[validate]
    status: Option<UserStatus>,
    /// New user profile data
    ///
    /// This is applied as a partial.
    #[validate]
    profile: Option<UserProfileData>,

    /// Bitfield of user badges
    #[serde(skip_serializing_if = "Option::is_none")]
    badges: Option<i32>,
    /// Enum of user flags
    #[serde(skip_serializing_if = "Option::is_none")]
    flags: Option<i32>,

    /// Fields to remove from user object
    #[validate(length(min = 1))]
    remove: Option<Vec<FieldsUser>>,
}

impl UserEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn display_name(mut self, display_name: impl std::fmt::Display) -> Self {
        self.display_name = Some(display_name.to_string());
        self
    }

    pub fn avatar(mut self, avatar: impl std::fmt::Display) -> Self {
        self.avatar = Some(avatar.to_string());
        self
    }

    pub fn status(mut self, status: impl Into<UserStatus>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn status_text(mut self, text: impl std::fmt::Display) -> Self {
        if let Some(status) = &mut self.status {
            status.text = Some(text.to_string());
        } else {
            self.status = Some(UserStatus {
                text: Some(text.to_string()),
                presence: None,
            });
        }
        self
    }

    pub fn status_presence(mut self, presence: impl Into<Presence>) -> Self {
        if let Some(status) = &mut self.status {
            status.presence = Some(presence.into());
        } else {
            self.status = Some(UserStatus {
                text: None,
                presence: Some(presence.into()),
            });
        }
        self
    }

    pub fn profile(mut self, profile: impl Into<UserProfileData>) -> Self {
        self.profile = Some(profile.into());
        self
    }

    pub fn profile_content(mut self, content: impl std::fmt::Display) -> Self {
        if let Some(profile) = &mut self.profile {
            profile.content = Some(content.to_string());
        } else {
            self.profile = Some(UserProfileData {
                content: Some(content.to_string()),
                background: None,
            });
        }
        self
    }

    pub fn profile_background(mut self, background: impl std::fmt::Display) -> Self {
        if let Some(profile) = &mut self.profile {
            profile.background = Some(background.to_string());
        } else {
            self.profile = Some(UserProfileData {
                content: None,
                background: Some(background.to_string()),
            });
        }
        self
    }

    pub fn badges(mut self, badges: impl Into<i32>) -> Self {
        self.badges = Some(badges.into());
        self
    }

    pub fn flags(mut self, flags: impl Into<i32>) -> Self {
        self.flags = Some(flags.into());
        self
    }

    pub fn remove(mut self, field: impl Into<FieldsUser>) -> Self {
        if let Some(remove) = &mut self.remove {
            remove.push(field.into());
        } else {
            self.remove = Some(vec![field.into()])
        }
        self
    }
}

impl Http {
    pub async fn edit_user(
        &self,
        user_id: impl std::fmt::Display,
        data: impl Into<UserEdit>,
    ) -> Result<User, HttpError> {
        let data: UserEdit = data.into();
        data.validate()?;
        let path = format!("users/{user_id}");
        let request = self.request(Method::PATCH, &path)?.json(&data);
        self.send_request(request).await
    }
}
