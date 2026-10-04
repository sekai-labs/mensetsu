import logging
from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Any

from src.llm.roles import ModelRole

logger = logging.getLogger('mensetsu.llm.telemetry')
ESTIMATED_COSTS: dict[str, tuple[float, float]] = {'gpt-4o-mini': (0.00015, 0.0006), 'gpt-4o': (0.0025, 0.01), 'claude-3-5-sonnet': (0.003, 0.015), 'claude-3-5-haiku': (0.0008, 0.004), 'default': (0.001, 0.002)}

@dataclass
class LLMUsageRecord:
    role: ModelRole
    provider: str
    model: str
    prompt_tokens: int
    completion_tokens: int
    total_tokens: int
    latency_ms: float
    cost_estimate_usd: float
    timestamp: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    metadata: dict[str, Any] = field(default_factory=dict)

class LLMTelemetry:

    def __init__(self):
        self._records: list[LLMUsageRecord] = []

    def record_usage(self, role: ModelRole, provider: str, model: str, prompt_tokens: int, completion_tokens: int, latency_ms: float, metadata: dict[str, Any] | None=None) -> LLMUsageRecord:
        rates = ESTIMATED_COSTS.get(model, ESTIMATED_COSTS['default'])
        cost = prompt_tokens / 1000.0 * rates[0] + completion_tokens / 1000.0 * rates[1]
        total_tokens = prompt_tokens + completion_tokens
        record = LLMUsageRecord(role=role, provider=provider, model=model, prompt_tokens=prompt_tokens, completion_tokens=completion_tokens, total_tokens=total_tokens, latency_ms=latency_ms, cost_estimate_usd=round(cost, 6), metadata=metadata or {})
        self._records.append(record)
        logger.info(f'[LLM Telemetry] role={role.value} provider={provider} model={model} tokens={total_tokens} (prompt={prompt_tokens}, comp={completion_tokens}) latency={latency_ms:.1f}ms cost=${cost:.6f}')
        return record

    def get_summary(self) -> dict[str, Any]:
        total_tokens = sum((r.total_tokens for r in self._records))
        total_cost = sum((r.cost_estimate_usd for r in self._records))
        total_calls = len(self._records)
        avg_latency = sum((r.latency_ms for r in self._records)) / total_calls if total_calls > 0 else 0.0
        return {'total_calls': total_calls, 'total_tokens': total_tokens, 'total_cost_usd': round(total_cost, 6), 'average_latency_ms': round(avg_latency, 2), 'by_role': {role.value: {'calls': len([r for r in self._records if r.role == role]), 'tokens': sum((r.total_tokens for r in self._records if r.role == role)), 'cost_usd': round(sum((r.cost_estimate_usd for r in self._records if r.role == role)), 6)} for role in ModelRole if any((r.role == role for r in self._records))}}

    def clear(self):
        self._records.clear()
telemetry = LLMTelemetry()
