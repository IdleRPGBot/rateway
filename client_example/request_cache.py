"""
To run this example, fire up rateway and execute it.
This will send a request to get a guild entity from cache
and listen for the reply
"""
import asyncio

import aio_pika


async def main():
    # Connect to the AMQP server. Change this to your username and password
    connection = await aio_pika.connect_robust("amqp://guest:guest@127.0.0.1/")

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

        # Declare a queue to send all the events that we want in
        # In this case, we would ideally make one queue for all cache stuff
        # and bind to it when we wait for a reply
        queue = await channel.declare_queue(
            "consumer-queue",
            auto_delete=False,
        )
        # Make the queue listen to the returned value
        await queue.bind(exchange, "guild-430017996304678923")

        # Send a request for a cache entity to rateway
        # return_routing_key is the same that we bind to in the line above
        await exchange.publish(
            aio_pika.Message(
                b'{"type": "Guild", "arguments": [430017996304678923], "return_routing_key": "guild-430017996304678923"}'
            ),
            routing_key="cache",
        )

        # Iterate over all incoming messages in the cache queue
        async with queue.iterator() as queue_iter:
            async for message in queue_iter:
                async with message.process():
                    # the message.body will be the discord entity
                    print(message.routing_key.upper(), message.body)
                    # we only listen to one so stop here
                    break


if __name__ == "__main__":
    asyncio.run(main())
