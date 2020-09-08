import asyncio
import aio_pika


async def main(loop):
    connection = await aio_pika.connect_robust(
        "amqp://broker:secret@127.0.0.1/", loop=loop
    )

    async with connection:
        # Creating channel
        channel = await connection.channel()    # type: aio_pika.Channel

        # Declaring exchange
        exchange = await channel.declare_exchange(
            "rateway-1",
            type="direct",
            auto_delete=True,
            durable=True,
        )

        await exchange.publish(aio_pika.Message(b'{"type": "guild", "return_routing_key": "guild-192571925071"}'), "cache")


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main(loop))
    loop.close()
