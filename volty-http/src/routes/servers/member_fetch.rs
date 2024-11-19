use std::collections::HashMap;

use reqwest::Method;
use serde::{Deserialize, Serialize};
use volty_types::servers::{server::Role, server_member::Member};

use crate::{error::HttpError, Http};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum MemberResponse {
    Member(Member),
    MemberWithRoles(MemberWithRoles),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemberWithRoles {
    member: Member,
    roles: HashMap<String, Role>,
}

impl Http {
    pub async fn fetch_member(
        &self,
        server_id: impl std::fmt::Display,
        user_id: impl std::fmt::Display,
    ) -> Result<Member, HttpError> {
        let path = format!("servers/{server_id}/members/{user_id}");
        let request = self.request(Method::GET, &path)?;
        let result: Result<MemberResponse, _> = self.send_request(request).await;
        result.map(|m| match m {
            MemberResponse::Member(m) => m,
            MemberResponse::MemberWithRoles(MemberWithRoles { member, roles: _ }) => member,
        })
    }

    pub async fn fetch_member_with_roles(
        &self,
        server_id: impl std::fmt::Display,
        user_id: impl std::fmt::Display,
    ) -> Result<MemberWithRoles, HttpError> {
        let path = format!("servers/{server_id}/members/{user_id}?roles=true");
        let request = self.request(Method::GET, &path)?;
        let result: Result<MemberResponse, _> = self.send_request(request).await;
        result.map(|m| {
            let MemberResponse::MemberWithRoles(data) = m else {
                panic!();
            };
            data
        })
    }
}
