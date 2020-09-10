use crate::model::{CacheEntity, CacheRequest};
use lapin::{
    options::{BasicAckOptions, BasicPublishOptions},
    types::AMQPValue,
    BasicProperties, Channel, Consumer,
};
use simd_json::{from_slice, to_vec};
use tokio::stream::StreamExt;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Cluster;

use std::error::Error;

pub async fn amqp_reader(
    cluster_id: usize,
    mut consumer: Consumer,
    amqp_channel: Channel,
    cluster: Cluster,
    cache: InMemoryCache,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let exchange_name = format!("rateway-{}", cluster_id);
    // TODO: Make more robust by not using ? and continue instead
    while let Some(delivery) = consumer.next().await {
        let (channel, mut delivery) = delivery.expect("error in consumer");
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await?;
        match delivery.routing_key.as_str() {
            "cache" => {
                let data: CacheRequest = from_slice(&mut delivery.data)?;
                // Rust can be annoying
                let send_data = match data.r#type {
                    CacheEntity::CurrentUser => Some(to_vec(&cache.current_user())?),
                    CacheEntity::GuildChannel => {
                        let channel = &cache.guild_channel(data.arguments[0].into());
                        if let Some(result) = channel {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Emoji => {
                        let emoji = cache.emoji(data.arguments[0].into());
                        if let Some(result) = emoji {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Group => {
                        let group = cache.group(data.arguments[0].into());
                        if let Some(result) = group {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Guild => {
                        let guild = cache.guild(data.arguments[0].into());
                        if let Some(result) = guild {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Member => {
                        let member =
                            cache.member(data.arguments[0].into(), data.arguments[1].into());
                        if let Some(result) = member {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Message => {
                        let message =
                            cache.message(data.arguments[0].into(), data.arguments[1].into());
                        if let Some(result) = message {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Presence => {
                        let presence =
                            cache.presence(data.arguments[0].into(), data.arguments[1].into());
                        if let Some(result) = presence {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::PrivateChannel => {
                        let channel = cache.private_channel(data.arguments[0].into());
                        if let Some(result) = channel {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::Role => {
                        let role = cache.role(data.arguments[0].into());
                        if let Some(result) = role {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::User => {
                        let user = cache.user(data.arguments[0].into());
                        if let Some(result) = user {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::VoiceChannelStates => {
                        let states = cache.voice_channel_states(data.arguments[0].into());
                        if let Some(result) = states {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
                    }
                    CacheEntity::VoiceState => {
                        let state =
                            cache.voice_state(data.arguments[0].into(), data.arguments[1].into());
                        if let Some(result) = state {
                            Some(to_vec(&result)?)
                        } else {
                            None
                        }
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
                        let data = String::from_utf8(delivery.data)?;
                        cluster.raw_command(actual_id, data).await?;
                    }
                }
            }
            _ => continue,
        };
    }

    Ok(())
}
