import os

import pytest
from langgraph.checkpoint.redis.aio import AsyncRedisSaver
from langgraph.graph import END, START, StateGraph
from pydantic import BaseModel


class SimpleState(BaseModel):
    step: int = 0
    val: str = ""


@pytest.mark.asyncio
async def test_redis_checkpointer_real_container():
    redis_url = os.environ.get("TEST_REDIS_URL", "redis://127.0.0.1:16379")
    thread_id = "test_run_recovery_123"

    saver_a = AsyncRedisSaver(redis_url=redis_url)
    await saver_a.asetup()
    builder = StateGraph(SimpleState)

    def node_1(state: SimpleState) -> dict:
        return {"step": 1, "val": "first_pass"}

    def node_2(state: SimpleState) -> dict:
        return {"step": 2, "val": "second_pass"}

    builder.add_node("node_1", node_1)
    builder.add_node("node_2", node_2)
    builder.add_edge(START, "node_1")
    builder.add_edge("node_1", "node_2")
    builder.add_edge("node_2", END)

    graph_a = builder.compile(checkpointer=saver_a)
    res_a = await graph_a.ainvoke(
        {"step": 0, "val": "init"}, config={"configurable": {"thread_id": thread_id}}
    )
    assert res_a["step"] == 2
    assert res_a["val"] == "second_pass"

    saver_b = AsyncRedisSaver(redis_url=redis_url)
    await saver_b.asetup()
    tuple_b = await saver_b.aget_tuple(config={"configurable": {"thread_id": thread_id}})
    assert tuple_b.checkpoint["channel_values"]["val"] == "second_pass"
