use crate::model::{CacheEntity, CacheRequest};
use lapin::{
    options::{BasicAckOptions, BasicPublishOptions},
    types::AMQPValue,
    BasicProperties, Channel, Consumer, Error,
};
use log::error;
use simd_json::{from_slice, to_vec};
use tokio::stream::StreamExt;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;

pub async fn amqp_reader(
    cluster_id: usize,
    mut consumer: Consumer,
    amqp_channel: Channel,
    cluster: Cluster,
    cache: InMemoryCache,
) -> Result<(), Error> {
    let exchange_name = format!("rateway-{}", cluster_id);
    while let Some(delivery) = consumer.next().await {
        let (channel, mut delivery) = delivery.expect("error in consumer");
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await?;
        match delivery.routing_key.as_str() {
            "cache" => {
                let data = match from_slice::<CacheRequest>(&mut delivery.data) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("Error deserializing cache request: {}", e);
                        continue;
                    }
                };
                // Rust can be annoying
                let send_data = match data.r#type {
                    CacheEntity::CurrentUser => to_vec(&cache.current_user()).ok(),
                    CacheEntity::GuildChannel => {
                        let channel = &cache.guild_channel(data.arguments[0].into());
                        channel.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Emoji => {
                        let emoji = cache.emoji(data.arguments[0].into());
                        emoji.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Group => {
                        let group = cache.group(data.arguments[0].into());
                        group.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Guild => {
                        let guild = cache.guild(data.arguments[0].into());
                        guild.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Member => {
                        let member =
                            cache.member(data.arguments[0].into(), data.arguments[1].into());
                        member.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Message => {
                        let message =
                            cache.message(data.arguments[0].into(), data.arguments[1].into());
                        message.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Presence => {
                        let presence =
                            cache.presence(data.arguments[0].into(), data.arguments[1].into());
                        presence.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::PrivateChannel => {
                        let channel = cache.private_channel(data.arguments[0].into());
                        channel.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::Role => {
                        let role = cache.role(data.arguments[0].into());
                        role.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::User => {
                        let user = cache.user(data.arguments[0].into());
                        user.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::VoiceChannelStates => {
                        let states = cache.voice_channel_states(data.arguments[0].into());
                        states.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                    CacheEntity::VoiceState => {
                        let state =
                            cache.voice_state(data.arguments[0].into(), data.arguments[1].into());
                        state.as_ref().map(|r| to_vec(&r).ok()).flatten()
                    }
                };
                amqp_channel
                    .basic_publish(
                        &exchange_name,
                        &data.return_routing_key,
                        BasicPublishOptions::default(),
                        send_data.unwrap_or_default(),
                        BasicProperties::default(),
                    )
                    .await?;
            }
            "gateway" => {
                if let Some(headers) = delivery.properties.headers() {
                    if let Some(shard_id) = headers.inner().get("shard_id") {
                        // Sometimes Rust sucks
                        let actual_id = match shard_id {
                            AMQPValue::LongInt(val) => *val as u64,
                            AMQPValue::LongLongInt(val) => *val as u64,
                            AMQPValue::LongUInt(val) => *val as u64,
                            AMQPValue::ShortInt(val) => *val as u64,
                            AMQPValue::ShortShortInt(val) => *val as u64,
                            AMQPValue::ShortShortUInt(val) => *val as u64,
                            AMQPValue::ShortUInt(val) => *val as u64,
                            _ => continue,
                        };
                        if let Err(e) = cluster.command_raw(actual_id, delivery.data).await {
                            error!("Error sending gateway command: {}", e);
                        };
                    }
                }
            }
            _ => continue,
        };
    }

    Ok(())
}
