# rateway

A stateful gateway for Discord Bots, especially with Travitia in mind.

Scaling well, it processes messages from the Discord gateway and puts them in AMQP exchanges.

## Why yet another one?

I am aware that there are projects like [spectacles](https://github.com/spec-tacles/) which try to solve this easily. rateway tries to solve a lot of the issues we had when trying to move an existing bot codebase to a seperate gateway.

- rateway is highly multi-threaded (parallelism) and concurrent (asynchronous)
- rateway is written in Rust and therefore safe, performant and well-optimized
- rateway keeps events grouped per cluster, where each gateway cluster should equal one worker on your client side. This allows for not worrying about your REACTION_ADD events arriving on a different worker while you're paginating a help menu
- rateway keeps a shared cache, so you don't have to. Querying it can be done via AMQP

## Documentation

One day.

## Configuration

Configuration is preferably done in enviroment variables, alternatively, a config file can be passed with `-c /path/to/rateway.toml`.

### Enviroment Variables

`DISCORD_TOKEN` (**required**): Bot Token

`INTENTS` (**required**): Intents value ([from here](https://ziad87.net/intents/))

`AMQP_URI` (**required**): AMQP URI to push events to

`SHARDS_PER_CLUSTER` (optional, defaults to 8): Shards per cluster

`EXTRA_SHARDS` (optional, defaults to 8): Extra shards to spawn for reserve

### Configuration File

```toml
token = "token"
amqp = "amqp://guest:guest@localhost/"

[shards]
per_cluster = 8
extra = 8
```

## Scalability and Performance

rateway utilizes [simd-json](https://github.com/simd-lite/simd-json) and [cloudflare-zlib](https://gitlab.com/kornelski/cloudflare-zlib-sys) for extremely fast gateway parsing. It supports gateway intents and queues connections over the gateway and its HTTP calls properly, making it practically impossible to hit ratelimits.

rateway puts shards into clusters and spawns a new task for each. Tasks are accelerated in threads across all CPU cores based on their load, using the work-stealing concept. This makes it highly efficient and very scalable, across high and low core count CPUs alike.
