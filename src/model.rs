use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct CacheRequest {
    pub r#type: String,
    pub return_routing_key: String,
}

#[derive(Deserialize, Debug)]
pub struct GatewaySendData {
    pub shard_id: u64,
    pub data: Value,
}
