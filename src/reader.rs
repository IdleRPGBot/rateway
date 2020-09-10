use crate::model::CacheRequest;
use lapin::{options::BasicAckOptions, types::AMQPValue, Consumer};
use simd_json::from_slice;
use tokio::stream::StreamExt;
use twilight_gateway::Cluster;

use std::error::Error;

pub async fn amqp_reader(
    mut consumer: Consumer,
    cluster: Cluster,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // TODO: Make more robust by not using ? and continue instead
    while let Some(delivery) = consumer.next().await {
        let (channel, mut delivery) = delivery.expect("error in consumer");
        channel
            .basic_ack(delivery.delivery_tag, BasicAckOptions::default())
            .await?;
        match delivery.routing_key.as_str() {
            "cache" => {
                let data: CacheRequest = from_slice(&mut delivery.data)?;
                // TODO: Add proper cache reading implementation
                println!("{:?}", data);
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
