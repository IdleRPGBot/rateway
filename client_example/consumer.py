"""
To run this example, fire up rateway and execute it.
This will listen to the events specified and print them to stdout
"""
import asyncio

import aio_pika

# Anything you like and need
LIST_OF_EVENTS_WE_WANT_TO_GET = ["MESSAGE_CREATE", "REACTION_ADD"]


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

        # Declare a queue to send all the events that we want in
        queue = await channel.declare_queue(
            "consumer-dispatch",
            auto_delete=False,
        )
        # Put all events with the events we want to get in the queue from the exchange
        for event in LIST_OF_EVENTS_WE_WANT_TO_GET:
            await queue.bind(exchange, event)

        # Iterate over all incoming messages in the queue
        # that means all events
        async with queue.iterator() as queue_iter:
            async for message in queue_iter:
                async with message.process():
                    # message.routing_key is the event name, e.g. MESSAGE_CREATE
                    # message.body is the raw JSON from Discord
                    print(message.routing_key.upper(), message.body)


if __name__ == "__main__":
    asyncio.run(main())
