use twilight_cache_inmemory::InMemoryCacheBuilder;
use twilight_gateway::cluster::ShardScheme;
use twilight_gateway::queue::{LargeBotQueue, Queue};
use twilight_http::Client;
use twilight_model::gateway::Intents;

use log::info;
use structopt::StructOpt;
use tokio::task::{spawn, JoinHandle};

use std::convert::{TryFrom, TryInto};
use std::iter::Iterator;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, error::Error};

mod config;
mod model;
mod reader;
mod worker;

#[derive(StructOpt, Debug)]
#[structopt(name = "rateway", about = "A stateful gateway for Discord Bots")]
struct Opt {
    #[structopt(short = "c", long = "config")]
    path_to_config_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Try to read config file from CLI, else use env
    let opt = Opt::from_args();
    let config = match opt.path_to_config_file {
        Some(file) => config::load_config(file),
        None => config::load_env(),
    };

    // Set up a HTTPClient
    let client = Client::new(config.token.clone());

    // Check total shards required
    let gateway = client
        .gateway()
        .authed()
        .await
        .expect("Cannot fetch shard information");

    // Set up a cache
    let cache = InMemoryCacheBuilder::new().message_cache_size(0).build();

    let total_shards = gateway.shards + config.shards.extra;

    let intents = Intents::from_bits_truncate(config.intents);

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
        .chunks(config.shards.per_cluster)
        .enumerate()
    {
        let shard_start = shard_range[0];
        let shard_end = shard_range[shard_range.len() - 1];
        let new_queue = queue.clone();
        let new_cache = cache.clone();
        let new_client = client.clone();
        let new_token = config.token.clone();
        let new_amqp_uri = config.amqp.clone();
        let cache_enabled = config.cache_enabled;
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
            cache_enabled,
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
