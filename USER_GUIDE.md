# User Guide to using rateway

This aims to explain how to implement a client library for rateway properly. It is subject to change as rateway reaches a 1.0.0 release, after that we will aim to be backwards compatible.

## AMQP Connections

As rateway exposes its entire API over AMQP, it is important to notice that you will need to connect to an AMQP server just like you connect rateway.

rateway uses a worker concept, which means that you will have to identify each of your workers with a specific ID that equals a number (hereby called _i_) from 1 to _n_, where _n_ is the number of rateway clusters, which itself is _(total_shard_count + extra_shards) / shards_per_cluster_ as specified in the configuration. We recommend the extra shards to be able to spawn extra workers in advance.

Each worker then should connect to the AMQP exchange created by rateway called rateway-_i_.

## Receiving gateway events

Normal gateway events that you subscribed to with intents from rateway will be sent with a routing key of _DISCORD_EVENT_NAME_, please refer [to Discord's docs](https://discord.com/developers/docs/topics/gateway#commands-and-events-gateway-events) for the specifics. The content of the message will be the according payload JSON.

To implement readingn a consumer, you could do:

- Connect to AMQP
- Declare the rateway-_i_ exchange
- Declare a queue for your worker
- Bind all the events you want (e.g. `MESSAGE_CREATE`) to the queue
- Loop the queue receiving and dispatch your events

## Sending gateway commands

To send commands to the gateway, you can send a message to the rateway-_i_ exchange. The routing key **must** be `gateway` and it expects a header in the message called `shard_id` that equals the shard to send a command to. The message body should be JSON and will be forwarded to Discord, refer [to their Documentation for the JSON format](https://discord.com/developers/docs/topics/gateway#commands-and-events-gateway-commands).

## Querying the cache

rateway will keep a cache of all entities except messages by default to make seamless worker restarts without wiping cache possible.
Querying it can be done via AMQP as well.

Right now, querying can only be done via IDs (which are required to be integers). To query an entity cache by ID, send a message to the exchange with the routing key `cache` and a JSON body of the following format:

```json
{
  "type": "EntityType",
  "arguments": [0123456789],
  "return_routing_key": "abcdef"
}
```

The arguments should be an array of IDs with 1 or 2 members, the order and amount expected following below in brackets.

All supported entities are:

- CurrentUser (No arguments)
- GuildChannel (ChannelID)
- Emoji (EmojiID)
- Group (GroupID)
- Guild (GuildID)
- Member (GuildID, UserID)
- Message (ChannelID, MessageID)
- Presence (GuildID, UserID)
- PrivateChannel (ChannelID)
- Role (RoleID)
- User (UserID)
- VoiceChannelStates (ChannelID)
- VoiceState (UserID, ChannelID)

The resulting object will be sent in JSON with the routing key **as specified in the request payload**.

For example, `{"type": "Guild", "arguments": [430017996304678923], "return_routing_key": "guild-430017996304678923"}` will send the JSON data of the guild with the ID 430017996304678923 over the exchange with the routing key `guild-430017996304678923`.
