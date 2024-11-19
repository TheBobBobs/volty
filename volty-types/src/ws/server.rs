use serde::{Deserialize, Serialize};

use crate::{
    channels::{
        channel::{Channel, FieldsChannel, PartialChannel},
        message::{AppendMessage, Message, PartialMessage},
    },
    media::emoji::Emoji,
    servers::{
        server::{FieldsRole, FieldsServer, PartialRole, PartialServer, Server},
        server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember},
    },
    users::{
        user::{FieldsUser, PartialUser, RelationshipStatus, User},
        user_settings::UserSettings,
    },
    util::result::Error,
};

use super::common::Ping;

/// WebSocket Client Errors
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "error")]
pub enum WebSocketError {
    LabelMe,
    InternalError { at: String },
    InvalidSession,
    OnboardingNotFinished,
    AlreadyAuthenticated,
    MalformedData { msg: String },
}

/// Untagged Error
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorEvent {
    Error(WebSocketError),
    APIError(Error),
}

/// Protocol Events
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    /// Multiple events
    Bulk { v: Vec<ServerMessage> },

    /// Successfully authenticated
    Authenticated,

    /// Basic data to cache
    Ready {
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
        emojis: Option<Vec<Emoji>>,
    },

    /// Ping response
    Pong { data: Ping },

    /// New message
    Message(Message),

    /// Update existing message
    MessageUpdate {
        id: String,
        #[serde(rename = "channel")]
        channel_id: String,
        data: PartialMessage,
    },

    /// Append information to existing message
    MessageAppend {
        id: String,
        #[serde(rename = "channel")]
        channel_id: String,
        append: AppendMessage,
    },

    /// Delete message
    MessageDelete {
        id: String,
        #[serde(rename = "channel")]
        channel_id: String,
    },

    /// New reaction to a message
    MessageReact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove user's reaction from message
    MessageUnreact {
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    },

    /// Remove a reaction from message
    MessageRemoveReaction {
        id: String,
        channel_id: String,
        emoji_id: String,
    },

    /// Bulk delete messages
    BulkMessageDelete {
        #[serde(rename = "channel")]
        channel_id: String,
        ids: Vec<String>,
    },

    /// New channel
    ChannelCreate(Channel),

    /// Update existing channel
    ChannelUpdate {
        id: String,
        data: PartialChannel,
        clear: Vec<FieldsChannel>,
    },

    /// Delete channel
    ChannelDelete { id: String },

    /// User joins a group
    ChannelGroupJoin {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// User leaves a group
    ChannelGroupLeave {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// User started typing in a channel
    ChannelStartTyping {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// User stopped typing in a channel
    ChannelStopTyping {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// User acknowledged message in channel
    ChannelAck {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
        message_id: String,
    },

    /// New server
    ServerCreate {
        id: String,
        server: Server,
        channels: Vec<Channel>,
        emojis: Vec<Emoji>,
    },

    /// Update existing server
    ServerUpdate {
        id: String,
        data: PartialServer,
        clear: Vec<FieldsServer>,
    },

    /// Delete server
    ServerDelete { id: String },

    /// Update existing server member
    ServerMemberUpdate {
        id: MemberCompositeKey,
        data: PartialMember,
        clear: Vec<FieldsMember>,
    },

    /// User joins server
    ServerMemberJoin {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// User left server
    ServerMemberLeave {
        id: String,
        #[serde(rename = "user")]
        user_id: String,
    },

    /// Server role created or updated
    ServerRoleUpdate {
        id: String,
        role_id: String,
        data: PartialRole,
        clear: Vec<FieldsRole>,
    },

    /// Server role deleted
    ServerRoleDelete { id: String, role_id: String },

    /// Update existing user
    UserUpdate {
        id: String,
        data: PartialUser,
        clear: Vec<FieldsUser>,
    },

    /// Relationship with another user changed
    UserRelationship {
        id: String,
        user: User,
        // ! this field can be deprecated
        status: RelationshipStatus,
    },

    /// Settings updated remotely
    UserSettingsUpdate { id: String, update: UserSettings },

    /// User has been platform banned or deleted their account
    ///
    /// Clients should remove the following associated data:
    /// - Messages
    /// - DM Channels
    /// - Relationships
    /// - Server Memberships
    ///
    /// User flags are specified to explain why a wipe is occurring though not all reasons will necessarily ever appear.
    UserPlatformWipe { user_id: String, flags: i32 },

    /// New emoji
    EmojiCreate(Emoji),

    /// Delete emoji
    EmojiDelete { id: String },

    /// Auth events
    Auth,
}
