# rateway

A stateless gateway for Discord Bots, especially with Travitia in mind.

Scaling well, it processes messages from the Discord gateway and puts them in AMQP exchanges.

## Configuration

Configuration is entirely done in enviroment variables.

`DISCORD_TOKEN` (**required**): Bot Token

`INTENTS` (**required**): Intents value ([from here](https://ziad87.net/intents/))

`AMQP_URI` (**required**): AMQP URI to push events to

`SHARDS_PER_CLUSTER` (optional, defaults to 8): Shards per cluster

`EXTRA_SHARDS` (optional, defaults to 8): Extra shards to spawn for reserve

## Scalability and Performance

rateway utilizes [simd-json](https://github.com/simd-lite/simd-json) and [cloudflare-zlib](https://gitlab.com/kornelski/cloudflare-zlib-sys) for extremely fast gateway parsing. It supports gateway intents and queues connections over the gateway and its HTTP calls properly, making it practically impossible to hit ratelimits.

rateway puts shards into clusters and spawns a new task for each. Tasks are accelerated in threads across all CPU cores based on their load, using the work-stealing concept. This makes it highly efficient and very scalable, across high and low core count CPUs alike.
