use serde::{Deserialize, Serialize};

/// Push Notification
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PushNotification {
    /// Known author name
    pub author: String,
    /// URL to author avatar
    pub icon: String,
    /// URL to first matching attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Message content or system message information
    pub body: String,
    /// Unique tag, usually the channel ID
    pub tag: String,
    /// Timestamp at which this notification was created
    pub timestamp: u64,
    /// URL to open when clicking notification
    pub url: String,
}
