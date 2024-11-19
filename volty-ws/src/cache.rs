use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use futures_util::Future;
use tokio::sync::{OnceCell, RwLock};
use volty_http::{error::HttpError, ApiError, Http};
use volty_types::{
    channels::{channel::Channel, message::Message},
    media::emoji::Emoji,
    permissions::{
        calculate_dm_permissions, calculate_group_permissions,
        calculate_server_channel_permissions, calculate_server_permissions,
        calculate_sm_permissions, PermissionValue,
    },
    servers::{server::Server, server_member::Member},
    users::user::{RelationshipStatus, User},
    ws::server::ServerMessage,
    RevoltConfig,
};

#[derive(Clone, Default)]
pub struct Cache {
    inner: Arc<InnerCache>,
}

impl Deref for Cache {
    type Target = InnerCache;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Cache {
    pub fn new() -> Self {
        Self::default()
    }
}

enum MemberCache {
    Partial(moka::future::Cache<String, Member>),
    Full(RwLock<HashMap<String, Member>>),
}

impl Default for MemberCache {
    fn default() -> Self {
        Self::Partial(moka::future::Cache::new(128))
    }
}

impl MemberCache {
    async fn get(&self, user_id: &str) -> Option<Member> {
        match self {
            MemberCache::Partial(cache) => cache.get(user_id).await,
            MemberCache::Full(cache) => cache.read().await.get(user_id).cloned(),
        }
    }

    pub async fn try_get_with<F>(&self, user_id: String, init: F) -> Result<Member, Arc<HttpError>>
    where
        F: Future<Output = Result<Member, HttpError>>,
    {
        match self {
            MemberCache::Partial(cache) => cache.try_get_with(user_id, init).await,
            MemberCache::Full(_) => unreachable!(),
        }
    }

    async fn insert(&self, user_id: String, member: Member) {
        match self {
            MemberCache::Partial(cache) => cache.insert(user_id, member).await,
            MemberCache::Full(cache) => {
                cache.write().await.insert(user_id, member);
            }
        }
    }

    async fn remove(&self, user_id: &str) -> Option<Member> {
        match self {
            MemberCache::Partial(cache) => cache.remove(user_id).await,
            MemberCache::Full(cache) => cache.write().await.remove(user_id),
        }
    }

    async fn values(&self) -> Vec<Member> {
        match self {
            MemberCache::Partial(cache) => cache.iter().map(|(_, m)| m).collect(),
            MemberCache::Full(cache) => cache.read().await.values().cloned().collect(),
        }
    }

    fn is_full(&self) -> bool {
        matches!(self, MemberCache::Full(_))
    }

    fn make_full(&mut self, members: Vec<Member>) {
        match self {
            MemberCache::Partial(_) => {
                let members = members
                    .into_iter()
                    .map(|m| (m.id.user.clone(), m))
                    .collect();
                *self = MemberCache::Full(RwLock::new(members));
            }
            MemberCache::Full(_) => unreachable!(),
        }
    }
}

pub struct InnerCache {
    api_info: OnceCell<RevoltConfig>,
    user_id: OnceLock<String>,
    user_mention: OnceLock<String>,
    user: RwLock<Option<User>>,

    users: moka::future::Cache<String, User>,
    servers: RwLock<HashMap<String, Server>>,
    channels: RwLock<HashMap<String, Channel>>,
    emojis: RwLock<HashMap<String, Emoji>>,

    members: RwLock<HashMap<String, MemberCache>>,
    messages: moka::future::Cache<String, Message>,

    user_dms: RwLock<HashMap<String, String>>,
}

impl Default for InnerCache {
    fn default() -> Self {
        Self {
            api_info: OnceCell::new(),
            user_id: Default::default(),
            user_mention: Default::default(),
            user: Default::default(),
            users: moka::future::Cache::new(1024),
            servers: Default::default(),
            channels: Default::default(),
            emojis: Default::default(),
            members: Default::default(),
            messages: moka::future::Cache::new(4096),
            user_dms: Default::default(),
        }
    }
}

impl InnerCache {
    pub async fn api_info(&self, http: &Http) -> Result<RevoltConfig, HttpError> {
        self.api_info
            .get_or_try_init(|| async move { http.api_info().await })
            .await
            .cloned()
    }

    pub fn user_id(&self) -> &str {
        self.user_id.get().expect("Only called after ready.")
    }

    pub fn user_mention(&self) -> &str {
        self.user_mention.get().expect("Only called after ready.")
    }

    pub async fn user(&self) -> User {
        self.user
            .read()
            .await
            .clone()
            .expect("Only called after ready.")
    }

    pub async fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).await
    }

    pub async fn fetch_user(&self, http: &Http, user_id: &str) -> Result<User, HttpError> {
        self.users
            .try_get_with(user_id.to_string(), async {
                http.fetch_user(user_id).await
            })
            .await
            .map_err(|e| (*e).clone())
    }

    pub async fn get_server(&self, server_id: &str) -> Option<Server> {
        self.servers.read().await.get(server_id).cloned()
    }

    pub async fn get_channel(&self, channel_id: &str) -> Option<Channel> {
        self.channels.read().await.get(channel_id).cloned()
    }

    pub async fn fetch_dm(&self, http: &Http, user_id: &str) -> Result<Channel, HttpError> {
        if let Some(channel_id) = self.user_dms.read().await.get(user_id) {
            Ok(self.get_channel(channel_id).await.unwrap())
        } else {
            http.open_dm(user_id).await
        }
    }

    pub async fn fetch_server_permissions(
        &self,
        http: &Http,
        server_id: &str,
        user_id: &str,
    ) -> Result<PermissionValue, HttpError> {
        let Some(server) = self.get_server(server_id).await else {
            return Err(ApiError::NotFound.into());
        };
        let member = self.fetch_member(http, server_id, user_id).await?;
        Ok(calculate_server_permissions(&server, &member))
    }

    pub async fn fetch_channel_permissions(
        &self,
        http: &Http,
        channel_id: &str,
        user_id: &str,
    ) -> Result<PermissionValue, HttpError> {
        let Some(channel) = self.get_channel(channel_id).await else {
            return Err(ApiError::NotFound.into());
        };
        let permissions = match &channel {
            Channel::SavedMessages { user, .. } => calculate_sm_permissions(user, user_id),
            Channel::DirectMessage { recipients, .. } => {
                calculate_dm_permissions(recipients, user_id)
            }
            Channel::Group {
                owner,
                recipients,
                permissions,
                ..
            } => calculate_group_permissions(owner, recipients, *permissions, user_id),
            Channel::TextChannel {
                server_id,
                default_permissions,
                role_permissions,
                ..
            }
            | Channel::VoiceChannel {
                server_id,
                default_permissions,
                role_permissions,
                ..
            } => {
                let Some(server) = self.get_server(server_id).await else {
                    return Err(ApiError::NotFound.into());
                };
                let member = self.fetch_member(http, server_id, user_id).await?;
                calculate_server_channel_permissions(
                    &server,
                    default_permissions,
                    role_permissions,
                    &member,
                )
            }
        };
        Ok(permissions)
    }

    pub async fn get_member(&self, server_id: &str, user_id: &str) -> Option<Member> {
        self.members.read().await.get(server_id)?.get(user_id).await
    }

    pub async fn fetch_member(
        &self,
        http: &Http,
        server_id: &str,
        user_id: &str,
    ) -> Result<Member, HttpError> {
        let s_members = self.members.read().await;
        if let Some(members) = s_members.get(server_id) {
            if members.is_full() {
                return members
                    .get(user_id)
                    .await
                    .ok_or(HttpError::Api(ApiError::NotFound));
            }
            members
                .try_get_with(user_id.to_string(), async {
                    http.fetch_member(server_id, user_id).await
                })
                .await
                .map_err(|e| (*e).clone())
        } else {
            Err(HttpError::Api(ApiError::NotFound))
        }
    }

    pub async fn fetch_members(
        &self,
        http: &Http,
        server_id: &str,
    ) -> Result<Vec<Member>, HttpError> {
        self.ensure_members(http, server_id).await?;
        let s_members = self.members.read().await;
        if let Some(members) = s_members.get(server_id) {
            if members.is_full() {
                return Ok(members.values().await);
            }
        }
        Err(HttpError::Api(ApiError::NotFound))
    }

    pub async fn ensure_members(&self, http: &Http, server_id: &str) -> Result<(), HttpError> {
        let s_members = self.members.read().await;
        if let Some(members) = s_members.get(server_id) {
            if members.is_full() {
                return Ok(());
            }
            drop(s_members);
            let response = http.fetch_members(server_id).await?;
            if let Some(members) = self.members.write().await.get_mut(server_id) {
                members.make_full(response.members);
            }
            for user in response.users {
                self.users.insert(user.id.clone(), user).await;
            }
            Ok(())
        } else {
            Err(HttpError::Api(ApiError::NotFound))
        }
    }

    pub async fn get_emoji(&self, emoji_id: &str) -> Option<Emoji> {
        self.emojis.read().await.get(emoji_id).cloned()
    }

    pub async fn get_message(&self, message_id: &str) -> Option<Message> {
        self.messages.get(message_id).await
    }

    pub async fn fetch_message(
        &self,
        http: &Http,
        channel_id: &str,
        message_id: &str,
    ) -> Result<Message, HttpError> {
        if let Some(message) = self.messages.get(message_id).await {
            return Ok(message);
        }
        let message = http.fetch_message(channel_id, message_id).await?;
        self.messages
            .insert(message_id.to_string(), message.clone())
            .await;
        Ok(message)
    }
}

#[async_trait]
pub trait UpdateCache {
    async fn update(&self, message: ServerMessage);
}

#[async_trait]
impl UpdateCache for InnerCache {
    async fn update(&self, message: ServerMessage) {
        use ServerMessage::*;
        let cache = moka::future::CacheBuilder::default().build();
        cache.insert("key".to_string(), 0).await;
        match message {
            Bulk { v } => {
                for message in v {
                    self.update(message).await;
                }
            }
            Authenticated => {}
            Ready {
                users,
                servers,
                channels,
                members,
                emojis,
            } => {
                let user = users
                    .iter()
                    .find(|u| matches!(u.relationship, Some(RelationshipStatus::User)))
                    .expect("User should be sent in Ready")
                    .clone();
                let user_id = user.id.clone();
                self.user_id.get_or_init(|| user_id.clone());
                self.user_mention.get_or_init(|| format!("<@{user_id}>"));
                self.user.write().await.replace(user);
                self.users.invalidate_all();
                for user in users {
                    self.users.insert(user.id.clone(), user).await;
                }

                let servers = servers.into_iter().map(|s| (s.id.clone(), s)).collect();
                let _ = std::mem::replace(self.servers.write().await.deref_mut(), servers);

                let channels: HashMap<_, _> = channels
                    .into_iter()
                    .map(|c| (c.id().to_string(), c))
                    .collect();
                let mut user_dms = self.user_dms.write().await;
                user_dms.clear();
                for channel in channels.values() {
                    if let Channel::DirectMessage { id, recipients, .. } = channel {
                        let other = recipients.iter().find(|&i| i != &user_id).unwrap();
                        user_dms.insert(other.clone(), id.clone());
                    }
                }
                let _ = std::mem::replace(self.channels.write().await.deref_mut(), channels);

                let emojis = emojis
                    .unwrap_or_default()
                    .into_iter()
                    .map(|e| (e.id.clone(), e))
                    .collect();
                let _ = std::mem::replace(self.emojis.write().await.deref_mut(), emojis);

                let mut new_members: HashMap<String, MemberCache> = HashMap::new();
                for member in members {
                    new_members
                        .entry(member.id.server.clone())
                        .or_default()
                        .insert(member.id.user.clone(), member)
                        .await;
                }
                let _ = std::mem::replace(self.members.write().await.deref_mut(), new_members);

                self.messages.invalidate_all();
            }
            Pong { .. } => {}

            Message(message) => {
                if let Some(channel) = self.channels.write().await.get_mut(&message.channel_id) {
                    match channel {
                        Channel::DirectMessage {
                            last_message_id, ..
                        }
                        | Channel::Group {
                            last_message_id, ..
                        }
                        | Channel::TextChannel {
                            last_message_id, ..
                        } => {
                            *last_message_id = Some(message.id.clone());
                        }
                        Channel::SavedMessages { .. } | Channel::VoiceChannel { .. } => {}
                    }
                }
                self.messages.insert(message.id.clone(), message).await;
            }
            MessageUpdate {
                id,
                channel_id: _,
                data,
            } => {
                if let Some(mut message) = self.messages.get(&id).await {
                    message.apply_options(data);
                    self.messages.insert(id, message).await;
                }
            }
            MessageAppend {
                id,
                channel_id: _,
                append,
            } => {
                if let Some(mut message) = self.messages.get(&id).await {
                    if let Some(embeds) = append.embeds {
                        if let Some(m_embeds) = &mut message.embeds {
                            m_embeds.extend(embeds);
                        } else {
                            message.embeds = Some(embeds);
                        }
                        self.messages.insert(id, message).await;
                    }
                }
            }
            MessageDelete { id, channel_id: _ } => {
                self.messages.invalidate(&id).await;
            }
            MessageReact {
                id,
                channel_id: _,
                user_id,
                emoji_id,
            } => {
                if let Some(mut message) = self.messages.get(&id).await {
                    message
                        .reactions
                        .entry(emoji_id)
                        .or_default()
                        .insert(user_id);
                    self.messages.insert(id, message).await;
                }
            }
            MessageUnreact {
                id,
                channel_id: _,
                user_id,
                emoji_id,
            } => {
                if let Some(mut message) = self.messages.get(&id).await {
                    if let Some(reactions) = message.reactions.get_mut(&emoji_id) {
                        reactions.shift_remove(&user_id);
                        if reactions.is_empty() {
                            message.reactions.shift_remove(&emoji_id);
                        }
                        self.messages.insert(id, message).await;
                    }
                }
            }
            MessageRemoveReaction {
                id,
                channel_id: _,
                emoji_id,
            } => {
                if let Some(mut message) = self.messages.get(&id).await {
                    message.reactions.shift_remove(&emoji_id);
                    self.messages.insert(id, message).await;
                }
            }
            BulkMessageDelete { channel_id: _, ids } => {
                for id in &ids {
                    self.messages.invalidate(id).await;
                }
            }

            ChannelCreate(channel) => {
                if let Channel::DirectMessage { id, recipients, .. } = &channel {
                    let user_id = self.user_id();
                    let other = recipients.iter().find(|&i| i != user_id).unwrap();
                    self.user_dms
                        .write()
                        .await
                        .insert(other.clone(), id.clone());
                }
                self.channels
                    .write()
                    .await
                    .insert(channel.id().to_string(), channel);
            }
            ChannelUpdate { id, data, clear } => {
                if let Some(channel) = self.channels.write().await.get_mut(&id) {
                    data.apply(channel);
                    for field in clear {
                        field.remove(channel);
                    }
                }
            }
            ChannelDelete { id } => {
                if let Some(Channel::DirectMessage { recipients, .. }) =
                    self.channels.write().await.remove(&id)
                {
                    let user_id = self.user_id();
                    let other = recipients.iter().find(|&i| i != user_id).unwrap();
                    self.user_dms.write().await.remove(other);
                }
            }
            ChannelGroupJoin { id, user_id } => {
                if let Some(Channel::Group { recipients, .. }) =
                    self.channels.write().await.get_mut(&id)
                {
                    recipients.insert(user_id);
                }
            }
            ChannelGroupLeave { id, user_id } => {
                if let Some(Channel::Group { recipients, .. }) =
                    self.channels.write().await.get_mut(&id)
                {
                    recipients.remove(&user_id);
                }
            }
            ChannelStartTyping { .. } => {}
            ChannelStopTyping { .. } => {}
            ChannelAck { .. } => {}

            ServerCreate {
                id,
                server,
                channels,
                emojis,
            } => {
                self.servers.write().await.insert(id.clone(), server);
                let mut c_channels = self.channels.write().await;
                for channel in channels {
                    c_channels.insert(channel.id().to_string(), channel);
                }
                let mut c_emojis = self.emojis.write().await;
                for emoji in emojis {
                    c_emojis.insert(emoji.id.clone(), emoji);
                }
                let user_id = self.user_id().to_string();
                let members = MemberCache::default();
                members
                    .insert(user_id.clone(), Member::new(id.clone(), user_id))
                    .await;
                self.members.write().await.insert(id, members);
            }
            ServerUpdate { id, data, clear } => {
                if let Some(server) = self.servers.write().await.get_mut(&id) {
                    server.apply_options(data);
                    for field in clear {
                        field.remove(server);
                    }
                }
            }
            ServerDelete { id } => {
                self.servers.write().await.remove(&id);
                self.members.write().await.remove(&id);
                self.channels
                    .write()
                    .await
                    .retain(|_, c| c.server_id() != Some(&id));
                self.emojis
                    .write()
                    .await
                    .retain(|_, e| e.parent.id() == Some(&id));
            }
            ServerMemberUpdate { id, data, clear } => {
                if let Some(members) = self.members.read().await.get(&id.server) {
                    if let Some(mut member) = members.get(&id.user).await {
                        member.apply_options(data);
                        for field in clear {
                            field.remove(&mut member);
                        }
                        members.insert(id.user, member).await;
                    }
                }
            }
            ServerMemberJoin { id, user_id } => {
                if let Some(members) = self.members.read().await.get(&id) {
                    let member = Member::new(id, user_id.clone());
                    members.insert(user_id, member).await;
                }
            }
            ServerMemberLeave { id, user_id } => {
                if Some(&user_id) == self.user_id.get() {
                    if let Some(server) = self.servers.write().await.remove(&id) {
                        let mut channels = self.channels.write().await;
                        for channel in server.channels {
                            channels.remove(&channel);
                        }
                        self.members.write().await.remove(&id);
                    }
                } else if let Some(members) = self.members.read().await.get(&id) {
                    members.remove(&user_id).await;
                }
            }
            ServerRoleUpdate {
                id,
                role_id,
                data,
                clear,
            } => {
                if let Some(server) = self.servers.write().await.get_mut(&id) {
                    // ServerRoleUpdate is also for RoleCreate
                    let role = server.roles.entry(role_id).or_default();
                    role.apply_options(data);
                    for field in clear {
                        field.remove(role);
                    }
                }
            }
            ServerRoleDelete { id, role_id } => {
                if let Some(server) = self.servers.write().await.get_mut(&id) {
                    server.roles.remove(&role_id);
                }
                if let Some(members) = self.members.read().await.get(&id) {
                    for mut member in members.values().await {
                        if member.roles.remove(&role_id) {
                            members.insert(member.id.user.clone(), member).await;
                        }
                    }
                }
            }

            UserUpdate { id, data, clear } => {
                if let Some(mut user) = self.users.get(&id).await {
                    user.apply_options(data);
                    for field in clear {
                        field.remove(&mut user);
                    }
                    if user.id == self.user_id() {
                        self.user.write().await.replace(user.clone());
                    }
                    self.users.insert(id, user).await;
                }
            }
            UserRelationship {
                id,
                user,
                status: _,
            } => {
                self.users.insert(id, user).await;
            }
            UserSettingsUpdate { .. } => {}
            UserPlatformWipe { user_id, flags: _ } => {
                self.users.invalidate(&user_id).await;
            }

            EmojiCreate(emoji) => {
                self.emojis.write().await.insert(emoji.id.clone(), emoji);
            }
            EmojiDelete { id } => {
                self.emojis.write().await.remove(&id);
            }

            Auth => {}
        }
    }
}
