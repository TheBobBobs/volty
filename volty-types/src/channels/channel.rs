use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{media::attachment::File, permissions::OverrideField, util::misc::if_false};

/// Representation of a channel on Revolt
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "channel_type")]
pub enum Channel {
    /// Personal "Saved Notes" channel which allows users to save messages
    SavedMessages {
        /// Unique Id
        #[serde(rename = "_id")]
        id: String,
        /// Id of the user this channel belongs to
        user: String,
    },
    /// Direct message channel between two users
    DirectMessage {
        /// Unique Id
        #[serde(rename = "_id")]
        id: String,

        /// Whether this direct message channel is currently open on both sides
        active: bool,
        /// Set of user ids participating in direct message
        recipients: HashSet<String>,
        /// Id of the last message sent in this channel
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,
    },
    /// Group channel between 1 or more participants
    Group {
        /// Unique Id
        #[serde(rename = "_id")]
        id: String,

        /// Display name of the channel
        name: String,
        /// User id of the owner of the group
        owner: String,
        /// Channel description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// Set of user ids participating in channel
        recipients: HashSet<String>,

        /// Custom icon attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
        /// Id of the last message sent in this channel
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        /// Permissions assigned to members of this group
        /// (does not apply to the owner of the group)
        #[serde(skip_serializing_if = "Option::is_none")]
        permissions: Option<i64>,

        /// Whether this group is marked as not safe for work
        #[serde(skip_serializing_if = "if_false", default)]
        nsfw: bool,
    },
    /// Text channel belonging to a server
    TextChannel {
        /// Unique Id
        #[serde(rename = "_id")]
        id: String,
        /// Id of the server this channel belongs to
        #[serde(rename = "server")]
        server_id: String,

        /// Display name of the channel
        name: String,
        /// Channel description
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// Custom icon attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,
        /// Id of the last message sent in this channel
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,

        /// Default permissions assigned to users in this channel
        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<OverrideField>,
        /// Permissions assigned based on role to this channel
        #[serde(
            default = "HashMap::<String, OverrideField>::new",
            skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
        )]
        role_permissions: HashMap<String, OverrideField>,

        /// Whether this channel is marked as not safe for work
        #[serde(skip_serializing_if = "if_false", default)]
        nsfw: bool,
    },
    /// Voice channel belonging to a server
    VoiceChannel {
        /// Unique Id
        #[serde(rename = "_id")]
        id: String,
        /// Id of the server this channel belongs to
        #[serde(rename = "server")]
        server_id: String,

        /// Display name of the channel
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Channel description
        description: Option<String>,
        /// Custom icon attachment
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<File>,

        /// Default permissions assigned to users in this channel
        #[serde(skip_serializing_if = "Option::is_none")]
        default_permissions: Option<OverrideField>,
        /// Permissions assigned based on role to this channel
        #[serde(
            default = "HashMap::<String, OverrideField>::new",
            skip_serializing_if = "HashMap::<String, OverrideField>::is_empty"
        )]
        role_permissions: HashMap<String, OverrideField>,

        /// Whether this channel is marked as not safe for work
        #[serde(skip_serializing_if = "if_false", default)]
        nsfw: bool,
    },
}

impl Channel {
    pub fn id(&self) -> &str {
        match self {
            Self::SavedMessages { id, .. }
            | Self::DirectMessage { id, .. }
            | Self::Group { id, .. }
            | Self::TextChannel { id, .. }
            | Self::VoiceChannel { id, .. } => id,
        }
    }

    pub fn server_id(&self) -> Option<&str> {
        match self {
            Channel::SavedMessages { .. }
            | Channel::DirectMessage { .. }
            | Channel::Group { .. } => None,
            Channel::TextChannel { server_id, .. } | Channel::VoiceChannel { server_id, .. } => {
                Some(server_id)
            }
        }
    }
}

/// Partial values of [Channel]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PartialChannel {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nsfw: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role_permissions: Option<HashMap<String, OverrideField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_permissions: Option<OverrideField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_message_id: Option<String>,
}

impl PartialChannel {
    pub fn apply(self, channel: &mut Channel) {
        match channel {
            Channel::SavedMessages { .. } => {}
            Channel::DirectMessage {
                active: c_active,
                last_message_id,
                ..
            } => {
                if let Some(active) = self.active {
                    *c_active = active;
                }
                if self.last_message_id.is_some() {
                    *last_message_id = self.last_message_id;
                }
            }
            Channel::Group {
                name: c_name,
                owner: c_owner,
                description,
                icon,
                last_message_id,
                permissions,
                nsfw: c_nsfw,
                ..
            } => {
                if let Some(name) = self.name {
                    *c_name = name;
                }
                if let Some(owner) = self.owner {
                    *c_owner = owner;
                }
                if self.description.is_some() {
                    *description = self.description;
                }
                if self.icon.is_some() {
                    *icon = self.icon;
                }
                if self.last_message_id.is_some() {
                    *last_message_id = self.last_message_id;
                }
                if self.permissions.is_some() {
                    *permissions = self.permissions;
                }
                if let Some(nsfw) = self.nsfw {
                    *c_nsfw = nsfw;
                }
            }
            Channel::TextChannel {
                name: c_name,
                description,
                icon,
                last_message_id,
                default_permissions,
                role_permissions: c_role_permissions,
                nsfw: c_nsfw,
                ..
            } => {
                if let Some(name) = self.name {
                    *c_name = name;
                }
                if self.description.is_some() {
                    *description = self.description;
                }
                if self.icon.is_some() {
                    *icon = self.icon;
                }
                if self.last_message_id.is_some() {
                    *last_message_id = self.last_message_id;
                }
                if self.default_permissions.is_some() {
                    *default_permissions = self.default_permissions;
                }
                if let Some(role_permissions) = self.role_permissions {
                    *c_role_permissions = role_permissions;
                }
                if let Some(nsfw) = self.nsfw {
                    *c_nsfw = nsfw;
                }
            }
            Channel::VoiceChannel {
                name: c_name,
                description,
                icon,
                default_permissions,
                role_permissions: c_role_permissions,
                nsfw: c_nsfw,
                ..
            } => {
                if let Some(name) = self.name {
                    *c_name = name;
                }
                if self.description.is_some() {
                    *description = self.description;
                }
                if self.icon.is_some() {
                    *icon = self.icon;
                }
                if self.default_permissions.is_some() {
                    *default_permissions = self.default_permissions;
                }
                if let Some(role_permissions) = self.role_permissions {
                    *c_role_permissions = role_permissions;
                }
                if let Some(nsfw) = self.nsfw {
                    *c_nsfw = nsfw;
                }
            }
        }
    }
}

/// Optional fields on channel object
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldsChannel {
    Description,
    Icon,
    DefaultPermissions,
}

impl FieldsChannel {
    pub fn remove(&self, channel: &mut Channel) {
        match self {
            FieldsChannel::Description => match channel {
                Channel::Group { description, .. }
                | Channel::TextChannel { description, .. }
                | Channel::VoiceChannel { description, .. } => {
                    *description = None;
                }
                Channel::SavedMessages { .. } | Channel::DirectMessage { .. } => {}
            },
            FieldsChannel::Icon => match channel {
                Channel::Group { icon, .. }
                | Channel::TextChannel { icon, .. }
                | Channel::VoiceChannel { icon, .. } => {
                    *icon = None;
                }
                Channel::SavedMessages { .. } | Channel::DirectMessage { .. } => {}
            },
            FieldsChannel::DefaultPermissions => match channel {
                Channel::TextChannel {
                    default_permissions,
                    ..
                }
                | Channel::VoiceChannel {
                    default_permissions,
                    ..
                } => {
                    *default_permissions = None;
                }
                Channel::SavedMessages { .. }
                | Channel::DirectMessage { .. }
                | Channel::Group { .. } => {}
            },
        }
    }
}
