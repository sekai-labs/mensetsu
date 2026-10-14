import asyncio

from src.orchestration.runtime.worker import run_worker_cli

main = run_worker_cli

if __name__ == "__main__":
    asyncio.run(run_worker_cli())
