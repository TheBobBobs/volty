use optional_struct::OptionalStruct;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{media::attachment::File, util::misc::if_false};

/// User's relationship with another user (or themselves)
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize)]
pub enum RelationshipStatus {
    #[default]
    None,
    User,
    Friend,
    Outgoing,
    Incoming,
    Blocked,
    BlockedOther,
}

/// Relationship entry indicating current status with other user
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Relationship {
    #[serde(rename = "_id")]
    pub id: String,
    pub status: RelationshipStatus,
}

/// Presence status
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Presence {
    /// User is online
    Online,
    /// User is not currently available
    Idle,
    /// User is focusing / will only receive mentions
    Focus,
    /// User is busy / will not receive any notifications
    Busy,
    /// User appears to be offline
    Invisible,
}

/// User's active status
#[derive(Clone, Debug, Default, Eq, PartialEq, Deserialize, Serialize, Validate)]
pub struct UserStatus {
    /// Custom status text
    #[validate(length(min = 1, max = 128))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Current presence option
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
}

/// User's profile
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserProfile {
    /// Text content on user's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Background visible on user's profile
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<File>,
}

/// User badge bitfield
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[repr(i32)]
pub enum Badges {
    /// Revolt Developer
    Developer = 1,
    /// Helped translate Revolt
    Translator = 2,
    /// Monetarily supported Revolt
    Supporter = 4,
    /// Responsibly disclosed a security issue
    ResponsibleDisclosure = 8,
    /// Revolt Founder
    Founder = 16,
    /// Platform moderator
    PlatformModeration = 32,
    /// Active monetary supporter
    ActiveSupporter = 64,
    /// 🦊🦝
    Paw = 128,
    /// Joined as one of the first 1000 users in 2021
    EarlyAdopter = 256,
    /// Amogus
    ReservedRelevantJokeBadge1 = 512,
    /// Low resolution troll face
    ReservedRelevantJokeBadge2 = 1024,
}

/// User flag enum
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[repr(i32)]
pub enum Flags {
    /// User has been suspended from the platform
    Suspended = 1,
    /// User has deleted their account
    Deleted = 2,
    /// User was banned off the platform
    Banned = 4,
}

/// Bot information for if the user is a bot
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BotInformation {
    /// Id of the owner of this bot
    pub owner: String,
}

/// Representiation of a User on Revolt.
#[derive(Clone, Debug, Default, Deserialize, Serialize, OptionalStruct)]
#[optional_derive(Clone, Debug, Default, Deserialize, Serialize)]
#[optional_name = "PartialUser"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct User {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,
    /// Username
    pub username: String,
    /// Discriminator
    pub discriminator: String,
    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Avatar attachment
    pub avatar: Option<File>,
    /// Relationships with other users
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relations: Option<Vec<Relationship>>,

    /// Bitfield of user badges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<i32>,
    /// User's current status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    /// User's profile page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfile>,

    /// Enum of user flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,
    /// Whether this user is privileged
    #[serde(skip_serializing_if = "if_false", default)]
    pub privileged: bool,
    /// Bot information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bot: Option<BotInformation>,

    // ? Entries below should never be pushed to the database
    /// Current session user's relationship with this user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<RelationshipStatus>,
    /// Whether this user is currently online
    #[serde(skip_serializing_if = "Option::is_none")]
    pub online: Option<bool>,
}

/// Optional fields on user object
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldsUser {
    Avatar,
    StatusText,
    StatusPresence,
    ProfileContent,
    ProfileBackground,
    DisplayName,
}

impl FieldsUser {
    pub fn remove(&self, user: &mut User) {
        match self {
            FieldsUser::Avatar => user.avatar = None,
            FieldsUser::StatusText => {
                if let Some(status) = &mut user.status {
                    status.text = None;
                }
            }
            FieldsUser::StatusPresence => {
                if let Some(status) = &mut user.status {
                    status.presence = None;
                }
            }
            FieldsUser::ProfileContent => {
                if let Some(profile) = &mut user.profile {
                    profile.content = None;
                }
            }
            FieldsUser::ProfileBackground => {
                if let Some(profile) = &mut user.profile {
                    profile.background = None;
                }
            }
            FieldsUser::DisplayName => {
                user.display_name = None;
            }
        }
    }
}

/// Enumeration providing a hint to the type of user we are handling
pub enum UserHint {
    /// Could be either a user or a bot
    Any,
    /// Only match bots
    Bot,
    /// Only match users
    User,
}
