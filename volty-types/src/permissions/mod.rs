mod permission;
mod user;

use std::collections::{HashMap, HashSet};

pub use permission::*;
pub use user::*;

use serde::{Deserialize, Serialize};

use crate::servers::{server::Server, server_member::Member};

/// Holds a permission value to manipulate.
#[derive(Debug)]
pub struct PermissionValue(pub u64);

/// Representation of a single permission override
#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub struct Override {
    /// Allow bit flags
    allow: u64,
    /// Disallow bit flags
    deny: u64,
}

/// Representation of a single permission override
/// as it appears on models and in the database
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct OverrideField {
    /// Allow bit flags
    a: i64,
    /// Disallow bit flags
    d: i64,
}

impl Override {
    /// Into allows
    pub fn allows(&self) -> u64 {
        self.allow
    }

    /// Into denies
    pub fn denies(&self) -> u64 {
        self.deny
    }
}

impl PermissionValue {
    /// Apply a given override to this value
    pub fn apply(&mut self, v: Override) {
        self.allow(v.allow);
        self.revoke(v.deny);
    }

    /// Allow given permissions
    pub fn allow(&mut self, v: u64) {
        self.0 |= v;
    }

    /// Revoke given permissions
    pub fn revoke(&mut self, v: u64) {
        self.0 &= !v;
    }

    /// Restrict to given permissions
    pub fn restrict(&mut self, v: u64) {
        self.0 &= v;
    }

    pub fn has(&self, permission: Permission) -> bool {
        let v = permission as u64;
        (self.0 & v) == v
    }
}

impl From<Override> for OverrideField {
    fn from(v: Override) -> Self {
        Self {
            a: v.allow as i64,
            d: v.deny as i64,
        }
    }
}

impl From<OverrideField> for Override {
    fn from(v: OverrideField) -> Self {
        Self {
            allow: v.a as u64,
            deny: v.d as u64,
        }
    }
}

impl From<i64> for PermissionValue {
    fn from(v: i64) -> Self {
        Self(v as u64)
    }
}

impl From<u64> for PermissionValue {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<PermissionValue> for u64 {
    fn from(v: PermissionValue) -> Self {
        v.0
    }
}

pub fn calculate_server_permissions(server: &Server, member: &Member) -> PermissionValue {
    if member.id.server != server.id {
        return PermissionValue::from(0u64);
    }
    if member.id.user == server.owner {
        return (Permission::GrantAllSafe as u64).into();
    }
    let mut permissions: PermissionValue = server.default_permissions.into();

    let mut roles: Vec<(i64, Override)> = member
        .roles
        .iter()
        .filter_map(|id| server.roles.get(id).map(|r| (r.rank, r.permissions.into())))
        .collect();
    roles.sort_by(|a, b| a.0.cmp(&b.0).reverse());
    for (_, v) in roles {
        permissions.apply(v);
    }

    if member.in_timeout() {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    permissions
}

pub fn calculate_server_channel_permissions(
    server: &Server,
    default_permissions: &Option<OverrideField>,
    role_permissions: &HashMap<String, OverrideField>,
    member: &Member,
) -> PermissionValue {
    if member.id.server != server.id {
        return PermissionValue::from(0u64);
    }
    if member.id.user == server.owner {
        return (Permission::GrantAllSafe as u64).into();
    }
    let mut permissions = calculate_server_permissions(server, member);

    if let Some(default) = default_permissions {
        permissions.apply((*default).into());
    }

    let mut roles: Vec<(i64, Override)> = role_permissions
        .iter()
        .filter(|(id, _)| member.roles.contains(*id))
        .filter_map(|(id, permission)| {
            server
                .roles
                .get(id)
                .map(|role| (role.rank, (*permission).into()))
        })
        .collect();
    roles.sort_by(|a, b| a.0.cmp(&b.0).reverse());

    for (_, v) in roles {
        permissions.apply(v);
    }

    if member.in_timeout() {
        permissions.restrict(*ALLOW_IN_TIMEOUT);
    }

    permissions
}

pub fn calculate_sm_permissions(owner: &str, user_id: &str) -> PermissionValue {
    if user_id == owner {
        (*DEFAULT_PERMISSION_SAVED_MESSAGES).into()
    } else {
        0_u64.into()
    }
}

pub fn calculate_dm_permissions(recipients: &HashSet<String>, user_id: &str) -> PermissionValue {
    if recipients.contains(user_id) {
        (*DEFAULT_PERMISSION_DIRECT_MESSAGE).into()
    } else {
        0_u64.into()
    }
}

pub fn calculate_group_permissions(
    owner: &str,
    recipients: &HashSet<String>,
    permissions: Option<i64>,
    user_id: &str,
) -> PermissionValue {
    if user_id == owner {
        return (Permission::GrantAllSafe as u64).into();
    }
    if !recipients.contains(user_id) {
        return 0_u64.into();
    }
    permissions
        .map(|p| p as u64)
        .unwrap_or(*DEFAULT_PERMISSION_DIRECT_MESSAGE)
        .into()
}
