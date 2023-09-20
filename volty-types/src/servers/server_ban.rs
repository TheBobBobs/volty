use serde::{Deserialize, Serialize};

use super::server_member::MemberCompositeKey;

/// Representation of a server ban on Revolt
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerBan {
    /// Unique member id
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,
    /// Reason for ban creation
    pub reason: Option<String>,
}
