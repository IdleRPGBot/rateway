"""
To run this example, fire up rateway and execute it.
This will send a payload to Discord via rateway
Here, we change the playing status of the bot
"""
import asyncio

import aio_pika

# The payload used by Discord to change a playing status for a bot.
# Here it's "Playing rawr"
PAYLOAD = b'{"d":{"afk":false,"activities":[{"type":0,"name":"rawr"}],"since":null,"status":"online"},"op":3}'


async def main():
    # Connect to the AMQP server. Change this to your username and password
    connection = await aio_pika.connect_robust("amqp://broker:secret@127.0.0.1/")

    # This will close the connection when leaving the context manager
    async with connection:
        # Every connection has a channel that we use to perform operations
        channel = await connection.channel()

        # Declare the rateway exchange that we will connect to
        # This is worker 1 so we use rateway-1
        #
        # Ignore the parameters, those are the ones used by rateway internally
        # Anything else would error
        exchange = await channel.declare_exchange(
            "rateway-1",
            type="direct",
            auto_delete=False,
            durable=True,
        )

        # Send a request to rateway
        # to relay this payload to Discord
        # via the shard 0
        await exchange.publish(
            aio_pika.Message(PAYLOAD),
            headers={"shard_id": 0},
            routing_key="gateway",
        )


if __name__ == "__main__":
    asyncio.run(main())
