import json
import logging
import os
import time
from typing import Any, AsyncGenerator, Optional

import redis.asyncio as aioredis

logger = logging.getLogger("mensetsu.orchestration.redis_queue")
DEFAULT_STREAM_KEY = "mensetsu:jobs:stream"
DEFAULT_GROUP_NAME = "mensetsu-intelligence-workers"
DEFAULT_REDIS_URL = os.environ.get("REDIS_URL", "redis://127.0.0.1:16379")
CLAIM_BLOCK_MS = 2000
REDIS_CLIENT_OPTIONS: dict[str, Any] = {
    "socket_timeout": 10.0,
    "socket_connect_timeout": 5.0,
    "socket_keepalive": True,
    "health_check_interval": 30,
    "retry_on_timeout": True,
}


class RedisStreamJobQueue:
    def __init__(
        self,
        redis_url: str = DEFAULT_REDIS_URL,
        stream_key: str = DEFAULT_STREAM_KEY,
        group_name: str = DEFAULT_GROUP_NAME,
    ):
        self.redis_url = redis_url
        self.stream_key = stream_key
        self.group_name = group_name
        self.client: Optional[aioredis.Redis] = None

    async def get_client(self) -> aioredis.Redis:
        if self.client is None:
            self.client = aioredis.from_url(
                self.redis_url, decode_responses=True, **REDIS_CLIENT_OPTIONS
            )

            try:
                await self.client.xgroup_create(
                    name=self.stream_key, groupname=self.group_name, id="$", mkstream=True
                )
            except Exception as e:
                if "BUSYGROUP" not in str(e):
                    logger.warning(f"Consumer group setup: {e}")
        return self.client

    async def enqueue_job(self, run_id: str, payload: dict[str, Any], delay_ms: int = 0) -> str:

        client = await self.get_client()
        due_ts = (time.time() * 1000) + delay_ms
        job_msg = {
            "run_id": run_id,
            "payload": json.dumps(payload),
            "enqueued_at": str(time.time()),
            "due_ts": str(due_ts),
        }
        if delay_ms > 0:
            delay_key = f"{self.stream_key}:delayed"
            await client.zadd(delay_key, {json.dumps(job_msg): due_ts})
            return f"delayed:{due_ts}"

        msg_id = await client.xadd(
            name=self.stream_key, fields=job_msg, maxlen=10000, approximate=True
        )
        return msg_id

    PROMOTION_LUA = """
local delay_key = KEYS[1]
local stream_key = KEYS[2]
local max_score = ARGV[1]
local limit = tonumber(ARGV[2]) or 10

local items = redis.call('ZRANGEBYSCORE', delay_key, 0, max_score, 'LIMIT', 0, limit)
local promoted = 0
for i, item_str in ipairs(items) do
    local removed = redis.call('ZREM', delay_key, item_str)
    if removed > 0 then
        local data = cjson.decode(item_str)
        local xadd_args = {'XADD', stream_key, 'MAXLEN', '~', '10000', '*'}
        for k, v in pairs(data) do
            table.insert(xadd_args, k)
            table.insert(xadd_args, tostring(v))
        end
        redis.call(unpack(xadd_args))
        promoted = promoted + 1
    end
end
return promoted
"""

    async def claim_job(
        self,
        worker_id: str,
        block_ms: int = CLAIM_BLOCK_MS,
        min_idle_time: int = 30000,
    ) -> dict[str, Any] | None:

        client = await self.get_client()

        delay_key = f"{self.stream_key}:delayed"
        now_ms = time.time() * 1000
        await client.eval(self.PROMOTION_LUA, 2, delay_key, self.stream_key, str(now_ms), 10)

        try:
            autoclaimed = await client.xautoclaim(
                name=self.stream_key,
                groupname=self.group_name,
                consumername=worker_id,
                min_idle_time=min_idle_time,
                start_id="0-0",
                count=1,
            )
            if autoclaimed and len(autoclaimed) >= 2 and autoclaimed[1]:
                msg = autoclaimed[1][0]
                msg_id = msg[0]
                data = msg[1]
                return {
                    "msg_id": msg_id,
                    "run_id": data["run_id"],
                    "payload": json.loads(data["payload"]),
                    "reclaimed": True,
                }
        except Exception as e:
            logger.debug(f"Autoclaim pass: {e}")

        resp = await client.xreadgroup(
            groupname=self.group_name,
            consumername=worker_id,
            streams={self.stream_key: ">"},
            count=1,
            block=block_ms,
        )
        if resp and resp[0] and resp[0][1]:
            msg = resp[0][1][0]
            msg_id = msg[0]
            data = msg[1]
            return {
                "msg_id": msg_id,
                "run_id": data["run_id"],
                "payload": json.loads(data["payload"]),
                "reclaimed": False,
            }
        return None

    async def ack_job(self, msg_id: str) -> None:
        client = await self.get_client()
        await client.xack(self.stream_key, self.group_name, msg_id)

    LUA_REFRESH_LEASE = """
local key = KEYS[1]
local token = ARGV[1]
local pexpire_ms = tonumber(ARGV[2])

if redis.call('GET', key) == token then
    return redis.call('PEXPIRE', key, pexpire_ms)
else
    return 0
end
"""

    async def acquire_lease(self, run_id: str, lease_token: str, ttl_ms: int = 30000) -> bool:

        client = await self.get_client()
        key = f"mensetsu:lease:{run_id}"
        res = await client.set(key, lease_token, px=ttl_ms, nx=True)
        return res is True or res == "OK"

    async def refresh_lease(self, run_id: str, lease_token: str, ttl_ms: int = 30000) -> bool:

        client = await self.get_client()
        key = f"mensetsu:lease:{run_id}"
        res = await client.eval(self.LUA_REFRESH_LEASE, 1, key, lease_token, str(ttl_ms))
        return res == 1 or res is True

    LUA_RELEASE_LEASE = """
if redis.call('GET', KEYS[1]) == ARGV[1] then
    return redis.call('DEL', KEYS[1])
else
    return 0
end
"""

    async def release_lease(self, run_id: str, lease_token: str) -> None:

        client = await self.get_client()
        key = f"mensetsu:lease:{run_id}"
        await client.eval(self.LUA_RELEASE_LEASE, 1, key, lease_token)

    async def reset_stream_idle(self, worker_id: str, msg_id: str) -> None:

        client = await self.get_client()
        try:
            await client.xclaim(
                name=self.stream_key,
                groupname=self.group_name,
                consumername=worker_id,
                min_idle_time=0,
                message_ids=[msg_id],
                justid=True,
            )
        except Exception as e:
            logger.debug(f"Failed to reset stream idle for {msg_id}: {e}")

    async def set_lease_heartbeat(
        self, run_id: str, lease_token: str, ttl_seconds: int = 30
    ) -> None:
        client = await self.get_client()
        key = f"mensetsu:lease:{run_id}"
        await client.set(key, lease_token, ex=ttl_seconds)

    async def verify_lease_token(self, run_id: str, lease_token: str) -> bool:
        client = await self.get_client()
        key = f"mensetsu:lease:{run_id}"
        val = await client.get(key)
        return val == lease_token

    async def cancel_run(self, run_id: str) -> None:
        client = await self.get_client()
        key = f"mensetsu:cancel:{run_id}"
        await client.set(key, "1", ex=86400)

    async def publish_event(self, run_id: str, event_json: str) -> None:
        client = await self.get_client()
        channel = f"mensetsu:events:{run_id}"
        await client.publish(channel, event_json)

    async def subscribe_events(self, run_id: str) -> AsyncGenerator[str, None]:
        client = await self.get_client()
        pubsub = client.pubsub()
        channel = f"mensetsu:events:{run_id}"
        await pubsub.subscribe(channel)
        try:
            while True:
                msg = await pubsub.get_message(ignore_subscribe_messages=True, timeout=1.0)
                if msg and msg["type"] == "message":
                    yield msg["data"]
        finally:
            await pubsub.unsubscribe(channel)
            await pubsub.close()

    async def close(self) -> None:
        if self.client:
            await self.client.aclose()
            self.client = None
