use crate::model::{CacheRequest, GatewaySendData};
use lapin::{options::BasicAckOptions, Consumer};
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
                let data: GatewaySendData = from_slice(&mut delivery.data)?;
                cluster.command(data.shard_id, &data.data).await?;
            }
            _ => continue,
        };
    }

    Ok(())
}
