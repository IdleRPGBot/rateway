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

        await exchange.publish(aio_pika.Message(b'{"shard_id": 0, "data": {"op": 3, "d": {"game": {"name": "Save the Oxford Comma", "type": 0}, "status": "online", "afk": false}}}'), "gateway")


if __name__ == "__main__":
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main(loop))
    loop.close()
