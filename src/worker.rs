use crate::reader::amqp_reader;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::cluster::{Cluster, ShardScheme};
use twilight_gateway::queue::Queue;
use twilight_http::Client;
use twilight_model::gateway::{event::DispatchEvent, Intents};

use lapin::{
    options::{
        BasicConsumeOptions, BasicPublishOptions, ExchangeDeclareOptions, QueueBindOptions,
        QueueDeclareOptions,
    },
    types::FieldTable,
    BasicProperties, Channel, Connection, ConnectionProperties, ExchangeKind,
};
use simd_json::to_vec;
use tokio::{spawn, stream::StreamExt};
use tokio_amqp::LapinTokioExt;

use std::convert::TryFrom;
use std::error::Error;
use std::sync::Arc;

pub struct WorkerConfig<'a> {
    pub token: &'a str,
    pub http_client: Client,
    pub queue: Arc<Box<dyn Queue>>,
    pub cache: InMemoryCache,
    pub intents: Intents,
    pub scheme: ShardScheme,
    pub cluster_id: usize,
    pub amqp_uri: String,
    pub cache_enabled: bool,
}

impl WorkerConfig<'_> {
    pub async fn build(self) -> Result<Worker, Box<dyn Error + Send + Sync>> {
        let cluster = Cluster::builder(self.token, self.intents)
            .shard_scheme(self.scheme)
            .http_client(self.http_client)
            .queue(self.queue)
            .build()
            .await?;

        let amqp_conn =
            Connection::connect(&self.amqp_uri, ConnectionProperties::default().with_tokio())
                .await?;
        let send_channel = amqp_conn.create_channel().await?;

        Ok(Worker {
            amqp_channel: send_channel,
            cluster,
            cache: self.cache,
            cluster_id: self.cluster_id,
            cache_enabled: self.cache_enabled,
        })
    }
}

pub struct Worker {
    amqp_channel: Channel,
    cluster: Cluster,
    cache: InMemoryCache,
    cluster_id: usize,
    cache_enabled: bool,
}

impl Worker {
    pub async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let exchange_name = format!("rateway-{}", self.cluster_id);
        let incoming_queue = format!("rateway-incoming-{}", self.cluster_id);
        self.amqp_channel
            .exchange_declare(
                &exchange_name,
                ExchangeKind::Direct,
                ExchangeDeclareOptions {
                    passive: false,
                    durable: true,
                    auto_delete: false,
                    internal: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await?;
        self.amqp_channel
            .queue_declare(
                &incoming_queue,
                QueueDeclareOptions::default(),
                FieldTable::default(),
            )
            .await?;
        self.amqp_channel
            .queue_bind(
                &incoming_queue,
                &exchange_name,
                "cache",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        self.amqp_channel
            .queue_bind(
                &incoming_queue,
                &exchange_name,
                "gateway",
                QueueBindOptions::default(),
                FieldTable::default(),
            )
            .await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let exchange_name = format!("rateway-{}", self.cluster_id);
        let incoming_queue = format!("rateway-incoming-{}", self.cluster_id);
        let mut events = self.cluster.events();

        let cluster_spawn = self.cluster.clone();
        let cluster_amqp = self.cluster.clone();
        let cluster_cache = self.cache.clone();
        let cluster_channel = self.amqp_channel.clone();

        spawn(async move {
            cluster_spawn.up().await;
        });

        let consumer = self
            .amqp_channel
            .basic_consume(
                &incoming_queue,
                "read-task",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        spawn(amqp_reader(
            self.cluster_id,
            consumer,
            cluster_channel,
            cluster_amqp,
            cluster_cache,
        ));

        while let Some((_, event)) = events.next().await {
            if self.cache_enabled {
                self.cache.update(&event);
            }
            if let Ok(dispatch_evt) = DispatchEvent::try_from(event) {
                // We can assume Some since this is a Dispatch event
                let kind = dispatch_evt.kind().name().unwrap();
                let serialized = to_vec(&dispatch_evt)?;
                self.amqp_channel
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
}
