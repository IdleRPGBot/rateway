use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct CacheRequest {
    pub r#type: String,
    pub return_routing_key: String,
}
