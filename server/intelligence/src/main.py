import asyncio

import uvicorn

from src.service import app
from src.worker import main as run_worker


async def start():
    asyncio.create_task(run_worker())
    config = uvicorn.Config(app, host="0.0.0.0", port=8001, log_level="info")
    server = uvicorn.Server(config)
    await server.serve()

if __name__ == "__main__":
    asyncio.run(start())
