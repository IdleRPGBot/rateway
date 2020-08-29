use std::future::Future;
use std::pin::Pin;
use twilight_gateway::queue::Queue;

#[derive(Debug)]
pub struct RedisQueue;

impl Queue for RedisQueue {
    // TODO!
    fn request(&'_ self, _shard_id: [u64; 2]) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {})
    }
}
