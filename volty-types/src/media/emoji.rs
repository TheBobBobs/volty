use serde::{Deserialize, Serialize};

/// Utility function to check if a boolean value is false
pub fn if_false(t: &bool) -> bool {
    !t
}

/// Information about what owns this emoji
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum EmojiParent {
    Server { id: String },
    Detached,
}

impl EmojiParent {
    pub fn id(&self) -> Option<&str> {
        match self {
            EmojiParent::Server { id } => Some(id),
            EmojiParent::Detached => None,
        }
    }
}

/// Representation of an Emoji on Revolt
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Emoji {
    /// Unique Id
    #[serde(rename = "_id")]
    pub id: String,
    /// What owns this emoji
    pub parent: EmojiParent,
    /// Uploader user id
    pub creator_id: String,
    /// Emoji name
    pub name: String,
    /// Whether the emoji is animated
    #[serde(skip_serializing_if = "if_false", default)]
    pub animated: bool,
    /// Whether the emoji is marked as nsfw
    #[serde(skip_serializing_if = "if_false", default)]
    pub nsfw: bool,
}
