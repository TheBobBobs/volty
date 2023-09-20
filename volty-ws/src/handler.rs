use async_trait::async_trait;
use volty_types::{
    channels::{
        channel::{Channel, FieldsChannel, PartialChannel},
        message::{AppendMessage, Message, PartialMessage},
    },
    media::emoji::Emoji,
    servers::{
        server::{FieldsRole, FieldsServer, PartialRole, PartialServer, Server},
        server_member::{FieldsMember, Member, MemberCompositeKey, PartialMember},
    },
    users::{
        user::{FieldsUser, PartialUser, RelationshipStatus, User},
        user_settings::UserSettings,
    },
    ws::{common::Ping, server::ServerMessage},
};

#[allow(unused_variables)]
#[async_trait]
pub trait RawHandler {
    async fn on_authenticated(&self) {}

    async fn on_ready(
        &self,
        users: Vec<User>,
        servers: Vec<Server>,
        channels: Vec<Channel>,
        members: Vec<Member>,
        emojis: Vec<Emoji>,
    ) {
    }

    async fn on_pong(&self, data: Ping) {}

    async fn on_message(&self, message: Message) {}
    async fn on_message_update(&self, id: String, channel_id: String, data: PartialMessage) {}
    async fn on_message_append(&self, id: String, channel_id: String, append: AppendMessage) {}
    async fn on_message_delete(&self, id: String, channel_id: String) {}
    async fn on_message_react(
        &self,
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    ) {
    }
    async fn on_message_unreact(
        &self,
        id: String,
        channel_id: String,
        user_id: String,
        emoji_id: String,
    ) {
    }
    async fn on_message_remove_reaction(&self, id: String, channel_id: String, emoji_id: String) {}
    async fn on_bulk_message_delete(&self, channel_id: String, ids: Vec<String>) {}

    async fn on_channel_create(&self, channel: Channel) {}
    async fn on_channel_update(&self, id: String, data: PartialChannel, clear: Vec<FieldsChannel>) {
    }
    async fn on_channel_delete(&self, id: String) {}
    async fn on_channel_group_join(&self, id: String, user_id: String) {}
    async fn on_channel_group_leave(&self, id: String, user_id: String) {}
    async fn on_channel_start_typing(&self, id: String, user_id: String) {}
    async fn on_channel_stop_typing(&self, id: String, user_id: String) {}
    async fn on_channel_ack(&self, id: String, user_id: String, message_id: String) {}

    async fn on_server_create(&self, id: String, server: Server, channels: Vec<Channel>) {}
    async fn on_server_update(&self, id: String, data: PartialServer, clear: Vec<FieldsServer>) {}
    async fn on_server_delete(&self, id: String) {}
    async fn on_server_member_update(
        &self,
        id: MemberCompositeKey,
        data: PartialMember,
        clear: Vec<FieldsMember>,
    ) {
    }
    async fn on_server_member_join(&self, id: String, user_id: String) {}
    async fn on_server_member_leave(&self, id: String, user_id: String) {}
    async fn on_server_role_update(
        &self,
        id: String,
        role_id: String,
        data: PartialRole,
        clear: Vec<FieldsRole>,
    ) {
    }
    async fn on_server_role_delete(&self, id: String, role_id: String) {}

    async fn on_user_update(&self, id: String, data: PartialUser, clear: Vec<FieldsUser>) {}
    async fn on_user_relationship(&self, id: String, user: User, status: RelationshipStatus) {}
    async fn on_user_settings_update(&self, id: String, update: UserSettings) {}
    async fn on_user_platform_wipe(&self, user_id: String, flags: i32) {}

    async fn on_emoji_create(&self, emoji: Emoji) {}
    async fn on_emoji_delete(&self, id: String) {}

    async fn on_auth(&self) {}

    async fn on_event(&self, event: ServerMessage) {
        use ServerMessage::*;
        match event {
            Bulk { v } => {
                for event in v {
                    self.on_event(event).await;
                }
            }
            Authenticated => {
                self.on_authenticated().await;
            }
            Ready {
                users,
                servers,
                channels,
                members,
                emojis,
            } => {
                self.on_ready(
                    users,
                    servers,
                    channels,
                    members,
                    emojis.unwrap_or_default(),
                )
                .await;
            }
            Pong { data } => {
                self.on_pong(data).await;
            }
            Message(message) => {
                self.on_message(message).await;
            }
            MessageUpdate {
                id,
                channel_id,
                data,
            } => {
                self.on_message_update(id, channel_id, data).await;
            }
            MessageAppend {
                id,
                channel_id,
                append,
            } => {
                self.on_message_append(id, channel_id, append).await;
            }
            MessageDelete { id, channel_id } => {
                self.on_message_delete(id, channel_id).await;
            }
            MessageReact {
                id,
                channel_id,
                user_id,
                emoji_id,
            } => {
                self.on_message_react(id, channel_id, user_id, emoji_id)
                    .await;
            }
            MessageUnreact {
                id,
                channel_id,
                user_id,
                emoji_id,
            } => {
                self.on_message_unreact(id, channel_id, user_id, emoji_id)
                    .await;
            }
            MessageRemoveReaction {
                id,
                channel_id,
                emoji_id,
            } => {
                self.on_message_remove_reaction(id, channel_id, emoji_id)
                    .await;
            }
            BulkMessageDelete { channel_id, ids } => {
                self.on_bulk_message_delete(channel_id, ids).await;
            }
            ChannelCreate(channel) => {
                self.on_channel_create(channel).await;
            }
            ChannelUpdate { id, data, clear } => {
                self.on_channel_update(id, data, clear).await;
            }
            ChannelDelete { id } => {
                self.on_channel_delete(id).await;
            }
            ChannelGroupJoin { id, user_id } => {
                self.on_channel_group_join(id, user_id).await;
            }
            ChannelGroupLeave { id, user_id } => {
                self.on_channel_group_leave(id, user_id).await;
            }
            ChannelStartTyping { id, user_id } => {
                self.on_channel_start_typing(id, user_id).await;
            }
            ChannelStopTyping { id, user_id } => {
                self.on_channel_stop_typing(id, user_id).await;
            }
            ChannelAck {
                id,
                user_id,
                message_id,
            } => {
                self.on_channel_ack(id, user_id, message_id).await;
            }
            ServerCreate {
                id,
                server,
                channels,
            } => {
                self.on_server_create(id, server, channels).await;
            }
            ServerUpdate { id, data, clear } => {
                self.on_server_update(id, data, clear).await;
            }
            ServerDelete { id } => {
                self.on_server_delete(id).await;
            }
            ServerMemberUpdate { id, data, clear } => {
                self.on_server_member_update(id, data, clear).await;
            }
            ServerMemberJoin { id, user_id } => {
                self.on_server_member_join(id, user_id).await;
            }
            ServerMemberLeave { id, user_id } => {
                self.on_server_member_leave(id, user_id).await;
            }
            ServerRoleUpdate {
                id,
                role_id,
                data,
                clear,
            } => {
                self.on_server_role_update(id, role_id, data, clear).await;
            }
            ServerRoleDelete { id, role_id } => {
                self.on_server_role_delete(id, role_id).await;
            }
            UserUpdate { id, data, clear } => {
                self.on_user_update(id, data, clear).await;
            }
            UserRelationship { id, user, status } => {
                self.on_user_relationship(id, user, status).await;
            }
            UserSettingsUpdate { id, update } => {
                self.on_user_settings_update(id, update).await;
            }
            UserPlatformWipe { user_id, flags } => {
                self.on_user_platform_wipe(user_id, flags).await;
            }
            EmojiCreate(emoji) => {
                self.on_emoji_create(emoji).await;
            }
            EmojiDelete { id } => {
                self.on_emoji_delete(id).await;
            }
            Auth => {
                self.on_auth().await;
            }
        }
    }
}
