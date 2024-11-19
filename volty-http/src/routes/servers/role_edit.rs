use reqwest::Method;
use serde::{Deserialize, Serialize};
use validator::Validate;
use volty_types::{
    servers::server::{FieldsRole, Role},
    util::regex::RE_COLOUR,
};

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
pub struct RoleEdit {
    /// Role name
    #[validate(length(min = 1, max = 32))]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    /// Role colour
    #[validate(length(min = 1, max = 128), regex(path = *RE_COLOUR))]
    #[serde(skip_serializing_if = "Option::is_none")]
    colour: Option<String>,

    /// Whether this role should be displayed separately
    #[serde(skip_serializing_if = "Option::is_none")]
    hoist: Option<bool>,

    /// Ranking position
    ///
    /// Smaller values take priority.
    #[serde(skip_serializing_if = "Option::is_none")]
    rank: Option<i64>,

    /// Fields to remove from role object
    #[validate(length(min = 1))]
    #[serde(skip_serializing_if = "Option::is_none")]
    remove: Option<Vec<FieldsRole>>,
}

impl RoleEdit {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl std::fmt::Display) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn colour(mut self, colour: impl std::fmt::Display) -> Self {
        self.colour = Some(colour.to_string());
        self
    }

    pub fn hoist(mut self) -> Self {
        self.hoist = Some(true);
        self
    }

    pub fn unhoist(mut self) -> Self {
        self.hoist = Some(false);
        self
    }

    pub fn rank(mut self, rank: impl Into<i64>) -> Self {
        self.rank = Some(rank.into());
        self
    }

    pub fn remove(mut self, remove: impl Into<FieldsRole>) -> Self {
        let remove = remove.into();
        if let Some(r) = &mut self.remove {
            r.push(remove);
        } else {
            self.remove = Some(vec![remove])
        }
        self
    }
}

impl Http {
    pub async fn edit_role(
        &self,
        server_id: impl std::fmt::Display,
        role_id: impl std::fmt::Display,
        edit: impl Into<RoleEdit>,
    ) -> Result<Role, HttpError> {
        let data: RoleEdit = edit.into();
        data.validate()?;
        let path = format!("servers/{server_id}/roles/{role_id}");
        let request = self.request(Method::PATCH, &path)?.json(&data);
        self.send_request(request).await
    }
}
