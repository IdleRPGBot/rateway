use serde::Deserialize;
use std::{fs, path::PathBuf};
use toml::from_str;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
    pub amqp: String,
    pub intents: u64,
    pub shards: ConfigShards,
}

#[derive(Debug, Deserialize)]
pub struct ConfigShards {
    pub per_cluster: usize,
    pub extra: u64,
}

pub fn load_config(p: PathBuf) -> Config {
    let file = fs::read_to_string(p).unwrap();
    let config: Config = from_str(&file).unwrap();
    config
}

pub fn load_env() -> Config {
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let intents: u64 = std::env::var("INTENTS")
        .expect("INTENTS not set")
        .parse()
        .expect("Cannot parse intents");
    let per_cluster: usize = std::env::var("SHARDS_PER_CLUSTER")
        .unwrap_or_else(|_| String::from("8"))
        .parse()
        .expect("Cannot parse shards per cluster");
    let extra: u64 = std::env::var("EXTRA_SHARDS")
        .unwrap_or_else(|_| String::from("8"))
        .parse()
        .expect("Cannot parse extra shards");
    let amqp = std::env::var("AMQP_URI").expect("AMQP_URI not set");
    Config {
        token,
        amqp,
        intents,
        shards: ConfigShards {
            per_cluster,
            extra,
        },
    }
}
