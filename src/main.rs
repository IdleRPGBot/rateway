use twilight_cache_inmemory::InMemoryCacheBuilder;
use twilight_gateway::cluster::ShardScheme;
use twilight_gateway::queue::{LargeBotQueue, Queue};
use twilight_http::Client;
use twilight_model::gateway::Intents;

use log::info;
use tokio::task::{spawn, JoinHandle};

use std::convert::{TryFrom, TryInto};
use std::iter::Iterator;
use std::sync::Arc;
use std::{env, error::Error};

mod model;
mod reader;
mod worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Read config from env
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let intent_value: u64 = std::env::var("INTENTS")
        .expect("INTENTS not set")
        .parse()
        .expect("Cannot parse intents");
    let shards_per_cluster: usize = std::env::var("SHARDS_PER_CLUSTER")
        .unwrap_or_else(|_| String::from("8"))
        .parse()
        .expect("Cannot parse shards per cluster");
    let additional_shards: u64 = std::env::var("EXTRA_SHARDS")
        .unwrap_or_else(|_| String::from("8"))
        .parse()
        .expect("Cannot parse extra shards");
    let amqp_uri = std::env::var("AMQP_URI").expect("AMQP_URI not set");

    // Set up a HTTPClient
    let client = Client::new(token.clone());

    // Check total shards required
    let gateway = client
        .gateway()
        .authed()
        .await
        .expect("Cannot fetch shard information");

    // Set up a cache
    let cache = InMemoryCacheBuilder::new().message_cache_size(0).build();

    let total_shards = gateway.shards + additional_shards;

    let intents = Some(Intents::from_bits_truncate(intent_value));

    // Set up a queue for syncing the auth
    // This uses the max_concurrency from the gateway automatically
    let queue: Arc<Box<dyn Queue>> = Arc::new(Box::new(
        LargeBotQueue::new(
            gateway
                .session_start_limit
                .max_concurrency
                .try_into()
                .unwrap(),
            &client,
        )
        .await,
    ));

    info!("Starting up rateway with {} shards", total_shards);
    info!("Intents: {:?}", intents);

    let mut all_handles: Vec<JoinHandle<()>> = Vec::new();

    // Spawn a thread for each cluster
    // Tokio will use a new thread due to rt-threaded
    for (idx, shard_range) in (0..total_shards)
        .collect::<Vec<u64>>()
        .chunks(shards_per_cluster)
        .enumerate()
    {
        let shard_start = shard_range[0];
        let shard_end = shard_range[shard_range.len() - 1];
        let new_queue = queue.clone();
        let new_cache = cache.clone();
        let new_client = client.clone();
        let new_token = token.clone();
        let new_amqp_uri = amqp_uri.clone();
        let scheme = ShardScheme::try_from((shard_start..=shard_end, total_shards))?;

        let worker_config = worker::WorkerConfig {
            amqp_uri: new_amqp_uri,
            cache: new_cache,
            cluster_id: idx + 1,
            http_client: new_client,
            intents,
            queue: new_queue,
            scheme,
            token: &new_token,
        };

        let worker = worker_config.build().await?;
        worker.initialize().await?;

        let handle = spawn(async move {
            worker.run().await.unwrap();
        });

        all_handles.push(handle);

        info!(
            "Spawned cluster #{} (shards {}-{})",
            idx + 1,
            shard_start,
            shard_end
        );
    }

    for handle in all_handles {
        handle.await.expect("Task failed");
    }

    Ok(())
}
