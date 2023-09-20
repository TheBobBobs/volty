use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use optional_struct::OptionalStruct;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{media::attachment::File, permissions::OverrideField, util::misc::if_false};

/// Representation of a server role
#[derive(Clone, Debug, Default, Deserialize, Serialize, OptionalStruct)]
#[optional_derive(Clone, Debug, Default, Deserialize, Serialize)]
#[optional_name = "PartialRole"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Role {
    /// Role name
    pub name: String,
    /// Permissions available to this role
    pub permissions: OverrideField,
    /// Colour used for this role
    ///
    /// This can be any valid CSS colour
    #[serde(skip_serializing_if = "Option::is_none")]
    pub colour: Option<String>,
    /// Whether this role should be shown separately on the member sidebar
    #[serde(skip_serializing_if = "if_false", default)]
    pub hoist: bool,
    /// Ranking of this role
    #[serde(default)]
    pub rank: i64,
}

/// Optional fields on role object
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldsRole {
    Colour,
}

impl FieldsRole {
    pub fn remove(&self, role: &mut Role) {
        match self {
            FieldsRole::Colour => role.colour = None,
        }
    }
}

/// Channel category
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct Category {
    /// Unique ID for this category
    #[validate(length(min = 1, max = 32))]
    pub id: String,
    /// Title for this category
    #[validate(length(min = 1, max = 32))]
    pub title: String,
    /// Channels in this category
    pub channels: Vec<String>,
}

/// System message channel assignments
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemMessageChannels {
    /// ID of channel to send user join messages in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_joined: Option<String>,
    /// ID of channel to send user left messages in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_left: Option<String>,
    /// ID of channel to send user kicked messages in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_kicked: Option<String>,
    /// ID of channel to send user banned messages in
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_banned: Option<String>,
}

/// Server flag enum
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
#[repr(i32)]
pub enum ServerFlags {
    Verified = 1,
    Official = 2,
}

/// Representation of a server on Revolt
#[derive(Clone, Debug, Default, Deserialize, Serialize, OptionalStruct)]
#[optional_derive(Clone, Debug, Default, Deserialize, Serialize)]
#[optional_name = "PartialServer"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Server {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,
    /// User id of the owner
    pub owner: String,

    /// Name of the server
    pub name: String,
    /// Description for the server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Channels within this server
    // ! FIXME: this may be redundant
    pub channels: Vec<String>,
    /// Categories for this server
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<Vec<Category>>,
    /// Configuration for sending system event messages
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_messages: Option<SystemMessageChannels>,

    /// Roles for this server
    #[serde(
        default = "HashMap::<String, Role>::new",
        skip_serializing_if = "HashMap::<String, Role>::is_empty"
    )]
    pub roles: HashMap<String, Role>,
    /// Default set of server and channel permissions
    pub default_permissions: i64,

    /// Icon attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<File>,
    /// Banner attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<File>,

    /// Enum of server flags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<i32>,

    /// Whether this server is flagged as not safe for work
    #[serde(skip_serializing_if = "if_false", default)]
    pub nsfw: bool,
    /// Whether to enable analytics
    #[serde(skip_serializing_if = "if_false", default)]
    pub analytics: bool,
    /// Whether this server should be publicly discoverable
    #[serde(skip_serializing_if = "if_false", default)]
    pub discoverable: bool,
}

impl Server {
    pub fn role_by_id_or_name(&self, id_or_name: impl AsRef<str>) -> Option<(&str, &Role)> {
        if let Some((role_id, role)) = self.roles.get_key_value(id_or_name.as_ref()) {
            return Some((role_id, role));
        };
        self.roles
            .iter()
            .find_map(|(i, r)| (r.name == id_or_name.as_ref()).then_some((i.as_str(), r)))
    }
}

/// Optional fields on server object
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldsServer {
    Description,
    Categories,
    SystemMessages,
    Icon,
    Banner,
}

impl FieldsServer {
    pub fn remove(&self, server: &mut Server) {
        match self {
            FieldsServer::Description => server.description = None,
            FieldsServer::Categories => server.categories = None,
            FieldsServer::SystemMessages => server.system_messages = None,
            FieldsServer::Icon => server.icon = None,
            FieldsServer::Banner => server.banner = None,
        }
    }
}
