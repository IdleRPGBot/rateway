use simd_json::to_vec;
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_gateway::queue::{LargeBotQueue, Queue};
use twilight_http::Client;
use twilight_model::gateway::{event::DispatchEvent, GatewayIntents};

use lapin::{
    options::{BasicPublishOptions, ExchangeDeclareOptions},
    types::FieldTable,
    BasicProperties, Connection, ConnectionProperties, ExchangeKind,
};
use tokio_amqp::*;

use log::info;
use tokio::task::{spawn, JoinHandle};

use futures::StreamExt;
use std::convert::{TryFrom, TryInto};
use std::iter::Iterator;
use std::sync::Arc;
use std::{env, error::Error};

async fn worker(
    token: &str,
    http_client: Client,
    queue: Arc<Box<dyn Queue>>,
    intents: Option<GatewayIntents>,
    scheme: ShardScheme,
    cluster_id: usize,
    amqp_uri: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a Discord cluster
    let cluster = Cluster::builder(token)
        .shard_scheme(scheme)
        .intents(intents)
        .http_client(http_client)
        .queue(queue)
        .build()
        .await?;

    // Event filter will probably wait for https://github.com/twilight-rs/twilight/issues/464

    // Connect to AMQP
    let amqp_conn =
        Connection::connect(&amqp_uri, ConnectionProperties::default().with_tokio()).await?;
    // Send channel
    let send_channel = amqp_conn.create_channel().await?;
    // Queue for sending
    let exchange_name = format!("rateway-{}", cluster_id);
    send_channel
        .exchange_declare(
            &exchange_name,
            ExchangeKind::Direct,
            ExchangeDeclareOptions {
                passive: false,
                durable: true,
                auto_delete: true,
                internal: false,
                nowait: false, // TODO: Consider this to be true
            },
            FieldTable::default(),
        )
        .await?;

    let mut events = cluster.events();

    let cluster_spawn = cluster.clone();

    spawn(async move {
        cluster_spawn.up().await;
    });

    while let Some((_, event)) = events.next().await {
        if let Ok(dispatch_evt) = DispatchEvent::try_from(event) {
            // We can assume Some since this is a Dispatch event
            let kind = dispatch_evt.kind().name().unwrap();
            let serialized = to_vec(&dispatch_evt)?;
            send_channel
                .basic_publish(
                    &exchange_name,
                    &kind,
                    BasicPublishOptions::default(),
                    serialized,
                    BasicProperties::default(),
                )
                .await?;
        }
    }

    Ok(())
}

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

    let total_shards = gateway.shards + additional_shards;

    let intents = Some(GatewayIntents::from_bits_truncate(intent_value));

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
        let new_client = client.clone();
        let new_token = token.clone();
        let new_amqp_uri = amqp_uri.clone();
        let scheme = ShardScheme::try_from((shard_start..=shard_end, total_shards))?;
        let handle = spawn(async move {
            worker(
                &new_token,
                new_client,
                new_queue,
                intents,
                scheme,
                idx + 1,
                new_amqp_uri,
            )
            .await
            .unwrap();
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
