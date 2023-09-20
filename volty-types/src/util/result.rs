use serde::{Deserialize, Serialize};
use validator::ValidationErrors;

use crate::permissions::{Permission, UserPermission};

/// Possible API Errors
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Error {
    /// This error was not labeled :(
    LabelMe,

    // ? Onboarding related errors
    AlreadyOnboarded,

    // ? User related errors
    UsernameTaken,
    InvalidUsername,
    UnknownUser,
    AlreadyFriends,
    AlreadySentRequest,
    Blocked,
    BlockedByOther,
    NotFriends,

    // ? Channel related errors
    UnknownChannel,
    UnknownAttachment,
    UnknownMessage,
    CannotEditMessage,
    CannotJoinCall,
    TooManyAttachments,
    TooManyReplies,
    EmptyMessage,
    PayloadTooLarge,
    CannotRemoveYourself,
    GroupTooLarge {
        max: usize,
    },
    AlreadyInGroup,
    NotInGroup,

    // ? Server related errors
    UnknownServer,
    InvalidRole,
    Banned,
    TooManyServers {
        max: usize,
    },
    TooManyEmoji,

    // ? Bot related errors
    ReachedMaximumBots,
    IsBot,
    BotIsPrivate,

    // ? Permission errors
    MissingPermission {
        permission: Permission,
    },
    MissingUserPermission {
        permission: UserPermission,
    },
    NotElevated,
    CannotGiveMissingPermissions,
    NotOwner,

    // ? General errors
    DatabaseError {
        operation: &'static str,
        with: &'static str,
    },
    InternalError,
    InvalidOperation,
    InvalidCredentials,
    InvalidProperty,
    InvalidSession,
    DuplicateNonce,
    VosoUnavailable,
    NotFound,
    NoEffect,
    FailedValidation {
        #[serde(skip_serializing, skip_deserializing)]
        error: ValidationErrors,
    },
}

impl Error {
    /// Create a missing permission error from a given permission
    pub fn from_permission<T>(permission: Permission) -> Result<T> {
        Err(if let Permission::ViewChannel = permission {
            Error::NotFound
        } else {
            Error::MissingPermission { permission }
        })
    }

    /// Create a missing user permission error from a given user permission
    pub fn from_user_permission<T>(permission: UserPermission) -> Result<T> {
        Err(if let UserPermission::Access = permission {
            Error::NotFound
        } else {
            Error::MissingUserPermission { permission }
        })
    }

    /// Create a failed validation error from given validation errors
    pub fn from_invalid<T>(validation_error: ValidationErrors) -> Result<T> {
        Err(Error::FailedValidation {
            error: validation_error,
        })
    }
}

/// Result type with custom Error
pub type Result<T, E = Error> = std::result::Result<T, E>;
