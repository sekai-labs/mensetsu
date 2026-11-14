from types import SimpleNamespace

import pytest

import src.orchestration.service as service_module
from src.orchestration.service import WorkflowService


@pytest.mark.asyncio
async def test_tool_output_pages_by_utf8_bytes_without_splitting_characters(monkeypatch):
    summary = "A🙂éB"

    class FakeLedger:
        async def get_tool_invocation(self, _run_id, _tool_invocation_id):
            return SimpleNamespace(output_summary=summary)

    monkeypatch.setattr(service_module, "get_ledger", lambda: FakeLedger())
    first = await WorkflowService.get_tool_output("run", "tool", offset=0, limit=2)
    assert first is not None
    assert first.content == "A🙂"
    assert first.offset == 0
    assert first.total_bytes == len(summary.encode("utf-8"))
    assert first.has_more

    next_offset = first.offset + len(first.content.encode("utf-8"))
    second = await WorkflowService.get_tool_output("run", "tool", offset=next_offset, limit=5)
    assert second is not None
    assert second.offset == next_offset
    assert second.content == "éB"
    assert not second.has_more
    assert first.content + second.content == summary
