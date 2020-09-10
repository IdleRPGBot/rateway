import asyncio
import aio_pika


async def main(loop):
    connection = await aio_pika.connect_robust(
        "amqp://broker:secret@127.0.0.1/", loop=loop
    )

    async with connection:
        # Creating channel
        channel = await connection.channel()  # type: aio_pika.Channel

        # Declaring exchange
        exchange = await channel.declare_exchange(
            "rateway-1",
            type="direct",
            durable=True,
        )
        # Declare queue
        queue = await channel.declare_queue(
            "consumer",
            auto_delete=False,
        )
        await queue.bind(exchange, "guild-430017996304678923")
        await exchange.publish(
            aio_pika.Message(
                b'{"type": "Guild", "arguments": [430017996304678923], "return_routing_key": "guild-430017996304678923"}'
            ),
            "cache",
        )

        async with queue.iterator() as queue_iter:
            # Cancel consuming after __aexit__
            async for message in queue_iter:
                async with message.process():
                    print(message.routing_key.upper(), message.body)
                    break


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main(loop))
    loop.close()
