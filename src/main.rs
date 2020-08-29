use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_model::gateway::GatewayIntents;

use futures::StreamExt;
use std::sync::Arc;
use std::{env, error::Error};

mod queue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // List of intents to enable
    // TODO: Move to config
    let intents = Some(GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES);

    // This is also the default.
    let scheme = ShardScheme::Auto;

    let cluster = Cluster::builder(env::var("DISCORD_TOKEN")?)
        .shard_scheme(scheme)
        .intents(intents)
        .queue(Arc::new(Box::new(queue::RedisQueue)))
        .build()
        .await?;

    let mut events = cluster.events();

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    while let Some((id, event)) = events.next().await {
        println!("Shard: {}, Event: {:?}", id, event.kind());
    }

    Ok(())
}
