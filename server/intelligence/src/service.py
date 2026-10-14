import os

import redis.asyncio as aioredis
from fastapi import FastAPI, Header, HTTPException, status

from src.db.client import SurrealDirectClient
from src.orchestration.auth import get_internal_token, verify_internal_auth
from src.orchestration.redis_queue import REDIS_CLIENT_OPTIONS
from src.orchestration.router import router as internal_workflow_router

get_internal_token()

app = FastAPI(title="Mensetsu Intelligence Service")


app.include_router(internal_workflow_router)


@app.get("/health/live")
async def health_live():
    return {"status": "alive"}


@app.get("/health/ready")
async def health_ready(authorization: str | None = Header(None)):
    surreal_ok = False
    redis_ok = False
    details = {}

    try:
        surreal_client = SurrealDirectClient()
        await surreal_client.execute_query("INFO FOR DB;")
        surreal_ok = True
    except Exception as e:
        details["surrealdb_error"] = str(e)

    live_workers_count = 0
    try:
        redis_url = os.environ.get("REDIS_URL", "redis://127.0.0.1:6379")
        r = aioredis.from_url(redis_url, **REDIS_CLIENT_OPTIONS)
        await r.ping()
        worker_keys = await r.keys("mensetsu:worker:*:alive")
        live_workers_count = len(worker_keys)
        await r.aclose()
        redis_ok = True
    except Exception as e:
        details["redis_error"] = str(e)
    details["live_workers"] = live_workers_count
    is_ready = surreal_ok and redis_ok

    if not authorization:
        if is_ready:
            return {"status": "ready"}
        raise HTTPException(
            status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
            detail={"status": "unhealthy"},
        )

    try:
        await verify_internal_auth(authorization)
    except HTTPException:
        if is_ready:
            return {"status": "ready"}
        raise HTTPException(status_code=503, detail={"status": "unhealthy"})

    if is_ready:
        return {"status": "ready", "surrealdb": "connected", "redis": "connected"}

    raise HTTPException(
        status_code=status.HTTP_503_SERVICE_UNAVAILABLE,
        detail={"status": "unhealthy", "diagnostics": details},
    )
