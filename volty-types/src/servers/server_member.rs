use std::collections::HashSet;

use iso8601_timestamp::Timestamp;
use optional_struct::OptionalStruct;
use serde::{Deserialize, Serialize};

use crate::media::attachment::File;

use super::server::Server;

/// Composite primary key consisting of server and user id
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MemberCompositeKey {
    /// Server Id
    pub server: String,
    /// User Id
    pub user: String,
}

/// Representation of a member of a server on Revolt
#[derive(Clone, Debug, Deserialize, Serialize, OptionalStruct)]
#[optional_derive(Clone, Debug, Default, Deserialize, Serialize)]
#[optional_name = "PartialMember"]
#[opt_skip_serializing_none]
#[opt_some_priority]
pub struct Member {
    /// Unique member id
    #[serde(rename = "_id")]
    pub id: MemberCompositeKey,

    /// Time at which this user joined the server
    pub joined_at: Timestamp,

    /// Member's nickname
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    /// Avatar attachment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<File>,

    /// Member's roles
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub roles: HashSet<String>,
    /// Timestamp this member is timed out until
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<Timestamp>,
}

impl Member {
    pub fn new(server_id: String, user_id: String) -> Self {
        let id = MemberCompositeKey {
            server: server_id,
            user: user_id,
        };
        Self {
            id,
            joined_at: Timestamp::now_utc(),
            nickname: None,
            avatar: None,
            roles: HashSet::new(),
            timeout: None,
        }
    }

    pub fn in_timeout(&self) -> bool {
        if let Some(timeout) = self.timeout {
            *timeout > *Timestamp::now_utc()
        } else {
            false
        }
    }

    pub fn effective_rank(&self, server: &Server) -> i64 {
        if self.id.user == server.owner {
            return i64::MIN;
        }
        self.rank(server)
    }

    pub fn rank(&self, server: &Server) -> i64 {
        self.roles
            .iter()
            .filter_map(|r| server.roles.get(r).map(|r| r.rank))
            .min()
            .unwrap_or(i64::MAX)
    }
}

/// Optional fields on server member object
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldsMember {
    Nickname,
    Avatar,
    Roles,
    Timeout,
}

impl FieldsMember {
    pub fn remove(&self, member: &mut Member) {
        match self {
            FieldsMember::Nickname => member.nickname = None,
            FieldsMember::Avatar => member.avatar = None,
            FieldsMember::Roles => member.roles.clear(),
            FieldsMember::Timeout => member.timeout = None,
        }
    }
}

/// Member removal intention
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum RemovalIntention {
    Leave,
    Kick,
    Ban,
}
