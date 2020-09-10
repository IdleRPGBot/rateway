use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub enum CacheEntity {
    CurrentUser,
    GuildChannel,
    Emoji,
    Group,
    Guild,
    Member,
    Message,
    Presence,
    PrivateChannel,
    Role,
    User,
    VoiceChannelStates,
    VoiceState,
}

#[derive(Deserialize, Debug)]
pub struct CacheRequest {
    pub r#type: CacheEntity,
    pub arguments: Vec<u64>,
    pub return_routing_key: String,
}
