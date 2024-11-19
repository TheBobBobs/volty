use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use validator::ValidationErrors;
use volty_types::permissions::{Permission, UserPermission};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ErrorType {
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
    TooManyAttachments {
        max: usize,
    },
    TooManyReplies {
        max: usize,
    },
    TooManyChannels {
        max: usize,
    },
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
    TooManyEmoji {
        max: usize,
    },
    TooManyRoles {
        max: usize,
    },

    // ? Bot related errors
    ReachedMaximumBots,
    IsBot,
    BotIsPrivate,

    // ? User safety related errors
    CannotReportYourself,

    // ? Permission errors
    MissingPermission {
        permission: String,
    },
    MissingUserPermission {
        permission: String,
    },
    NotElevated,
    NotPrivileged,
    CannotGiveMissingPermissions,
    NotOwner,

    // ? General errors
    DatabaseError {
        operation: String,
        collection: String,
    },
    InternalError,
    InvalidOperation,
    InvalidCredentials,
    InvalidProperty,
    InvalidSession,
    DuplicateNonce,
    NotFound,
    NoEffect,
    FailedValidation {
        error: String,
    },

    // ? Legacy errors
    VosoUnavailable,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoreError {
    /// Type of error and additional information
    #[serde(flatten)]
    pub error_type: ErrorType,

    /// Where this error occurred
    pub location: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ApiError {
    /// This error was not labeled :(
    LabelMe,

    /// Core crate error
    Core {
        #[serde(flatten)]
        error: CoreError,
    },

    // ? Onboarding related errors
    AlreadyOnboarded,

    // ? User related errors
    UsernameTaken,
    InvalidUsername,
    DiscriminatorChangeRatelimited,
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
    TooManyAttachments {
        max: usize,
    },
    TooManyReplies {
        max: usize,
    },
    TooManyChannels {
        max: usize,
    },
    TooManyEmbeds {
        max: usize,
    },
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
    TooManyEmoji {
        max: usize,
    },
    TooManyRoles {
        max: usize,
    },

    // ? Bot related errors
    ReachedMaximumBots,
    IsBot,
    BotIsPrivate,

    // ? User safety related errors
    CannotReportYourself,

    // ? Permission errors
    MissingPermission {
        permission: Permission,
    },
    MissingUserPermission {
        permission: UserPermission,
    },
    NotElevated,
    NotPrivileged,
    CannotGiveMissingPermissions,
    NotOwner,

    // ? General errors
    DatabaseError {
        operation: String,
        with: String,
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

    RetryAfter(Duration),
}

impl From<ValidationErrors> for ApiError {
    fn from(value: ValidationErrors) -> Self {
        Self::FailedValidation { error: value }
    }
}

#[derive(Clone, Debug)]
pub enum HttpError {
    Api(ApiError),
    Reqwest(Arc<reqwest::Error>),
    Serde(Arc<serde_json::Error>),
}

impl From<ApiError> for HttpError {
    fn from(value: ApiError) -> Self {
        HttpError::Api(value)
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(value: reqwest::Error) -> Self {
        HttpError::Reqwest(Arc::new(value))
    }
}

impl From<Arc<reqwest::Error>> for HttpError {
    fn from(value: Arc<reqwest::Error>) -> Self {
        HttpError::Reqwest(value)
    }
}

impl From<serde_json::Error> for HttpError {
    fn from(value: serde_json::Error) -> Self {
        HttpError::Serde(Arc::new(value))
    }
}

impl From<Arc<serde_json::Error>> for HttpError {
    fn from(value: Arc<serde_json::Error>) -> Self {
        HttpError::Serde(value)
    }
}

impl From<ValidationErrors> for HttpError {
    fn from(value: ValidationErrors) -> Self {
        HttpError::Api(value.into())
    }
}
