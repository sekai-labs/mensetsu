"""Autonomous agent runtime: model selection, budgets, and execution loop."""
import asyncio
import contextlib
import json
import logging
import os
import time
import uuid
from datetime import datetime, timezone
from typing import Any, Callable

from langchain.agents import create_agent
from langchain.agents.middleware import (
    AgentMiddleware,
    ModelCallLimitMiddleware,
    ToolCallLimitMiddleware,
    wrap_model_call,
    wrap_tool_call,
)
from langchain.agents.structured_output import ToolStrategy
from langchain_core.language_models.chat_models import BaseChatModel
from langchain_openai import ChatOpenAI
from pydantic import BaseModel

from src.agents.agent_tools import ToolsMixin
from src.agents.registry import registry
from src.agents.runtime_support import (
    EVIDENCE_FLOOR_BY_AGENT,
    FINALIZE_MIN_WALL_TIME_RESERVE_S,
    FINALIZE_MODEL_CALL_HEADROOM,
    FINALIZE_RESERVED_MODEL_CALLS,
    FINALIZE_RESERVED_TOOL_CALLS,
    FINALIZE_TIME_RATIO,
    LLM_REQUEST_TIMEOUT_S,
    MAX_MISSING_OUTPUT_TOOL_NUDGES,
    AgentExecutionError,
    ExecutionBudgetClock,
    build_llm_retry_middleware,
    build_provider_slot_middleware,
    current_agent_id_var,
    current_budget_clock_var,
    current_ledger_var,
    current_llm_config_var,
    current_lost_lease_var,
    current_model_factory_var,
    has_explicit_llm_config,
    llm_config_scope,
)
from src.agents.schemas import (
    AGENT_OUTPUT_SCHEMAS,
    AgentTaskOutput,
)
from src.contracts.v1 import (
    AgentExecution,
    Artifact,
    Delegation,
    ExecutionBudgets,
    ExecutionBudgetsUsed,
    ToolInvocation,
)
from src.fetcher.search_engine import AgentSearchEngine
from src.orchestration.id_validator import validate_id
from src.orchestration.ledger import SurrealLedgerRepository
from src.orchestration.limits import agent_limits, provider_key
from src.orchestration.sanitizer import sanitize_data

__all__ = [
    "AgentExecutionError",
    "AutonomousAgentRuntime",
    "ExecutionBudgetClock",
    "LLM_REQUEST_TIMEOUT_S",
    "MAX_MISSING_OUTPUT_TOOL_NUDGES",
    "agent_runtime",
    "build_llm_retry_middleware",
    "build_provider_slot_middleware",
    "current_agent_id_var",
    "current_budget_clock_var",
    "current_ledger_var",
    "current_llm_config_var",
    "current_lost_lease_var",
    "current_model_factory_var",
    "has_explicit_llm_config",
    "llm_config_scope",
]

logger = logging.getLogger("mensetsu.agents.runtime")


class AutonomousAgentRuntime(ToolsMixin):
    def __init__(
        self,
        ledger: SurrealLedgerRepository | None = None,
        default_ledger: SurrealLedgerRepository | None = None,
    ):
        self._default_ledger = ledger or default_ledger or SurrealLedgerRepository()
        self._global_model_factory: Callable[[], BaseChatModel] | None = None
        self._search_engine: AgentSearchEngine | None = None

    @property
    def ledger(self) -> SurrealLedgerRepository:

        ctx_ledger = current_ledger_var.get()
        return ctx_ledger if ctx_ledger is not None else self._default_ledger

    def set_model_factory(self, factory: Callable[[], BaseChatModel] | None) -> None:

        self._global_model_factory = factory
        current_model_factory_var.set(factory)

    def set_search_engine(self, engine: AgentSearchEngine | None) -> None:
        self._search_engine = engine

    def get_search_engine(self) -> AgentSearchEngine:
        if self._search_engine is None:
            self._search_engine = AgentSearchEngine()
        return self._search_engine

    def get_chat_model(
        self, role: str = "research", request_timeout_s: float = LLM_REQUEST_TIMEOUT_S
    ) -> BaseChatModel:
        factory = current_model_factory_var.get() or self._global_model_factory
        if factory:
            return factory()

        from src.agents.model_router import resolve_chat_model_for_role

        llm_config = current_llm_config_var.get()
        if has_explicit_llm_config(llm_config, role):
            return resolve_chat_model_for_role(role, llm_config, timeout_s=request_timeout_s)

        base_url = os.environ.get("LLM_OPENAI_BASE_URL")
        api_key = os.environ.get("LLM_OPENAI_API_KEY")
        model = os.environ.get("LLM_DEFAULT_MODEL", "gemini-3.8-flash-high")
        if base_url and api_key:
            return ChatOpenAI(
                base_url=base_url,
                api_key=api_key,
                model=model,
                temperature=0.2,
                timeout=request_timeout_s,
                max_retries=0,
            )
        return resolve_chat_model_for_role(role, None, timeout_s=request_timeout_s)

    async def _close_orphaned_child(
        self,
        child_task: asyncio.Task | None,
        child_exec: AgentExecution,
        delegation: Delegation,
        parent: AgentExecution,
        run_id: str,
        reason: str,
    ) -> None:
        if child_task is not None and not child_task.done():
            child_task.cancel()
        if child_task is not None:
            with contextlib.suppress(BaseException):
                await child_task
        ledger = self.ledger
        try:
            persisted = await ledger.get_agent_execution(run_id, child_exec.id)
            if persisted is None or persisted.status in ("pending", "running"):
                child_exec.status = "cancelled"
                child_exec.error = reason
                child_exec.completed_at = datetime.now(timezone.utc)
                await ledger.save_agent_execution(child_exec)
                await ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.failed",
                    task_id=child_exec.task_id,
                    execution_id=child_exec.id,
                    payload={"execution_id": child_exec.id, "error": reason},
                )
            if delegation.status not in ("completed", "failed", "rejected"):
                delegation.status = "failed"
                delegation.error = reason
                await ledger.save_delegation(delegation)
                await ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="delegation.failed",
                    task_id=parent.task_id,
                    execution_id=parent.id,
                    payload={"delegation_id": delegation.id, "error": reason},
                )
        except Exception as ce:
            logger.error(f"Could not close orphaned child {child_exec.id} of {parent.id}: {ce}")

    async def execute_agent_loop(
        self,
        agent_id: str,
        objective: str,
        task_id: str,
        run_id: str,
        input_data: dict[str, Any],
        model_override: BaseChatModel | None = None,
        depth: int = 0,
        ancestors: list[str] | None = None,
        execution_id: str | None = None,
        parent_execution_id: str | None = None,
    ) -> tuple[AgentExecution, dict[str, Any]]:
        execution_id = execution_id or f"exec_{uuid.uuid4().hex[:12]}"
        ledger = self.ledger

        async def announce_wait(limit_label: str) -> None:
            await ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.activity",
                task_id=task_id,
                execution_id=execution_id,
                payload={
                    "execution_id": execution_id,
                    "activity_type": "waiting_for_slot",
                    "description": f"{agent_id}: Waiting for execution slot ({limit_label})",
                },
            )
        slot_wait_started = time.perf_counter()
        async with agent_limits.execution_slot(run_id, agent_id, on_wait=announce_wait):
            slot_wait_s = time.perf_counter() - slot_wait_started
            if slot_wait_s >= 1.0:
                logger.info(
                    f"Execution {execution_id} ({agent_id}): execution slot wait took {slot_wait_s:.1f}s; "
                    "deadline not started during it"
                )
            return await self._run_agent_loop(
                agent_id=agent_id,
                objective=objective,
                task_id=task_id,
                run_id=run_id,
                input_data=input_data,
                model_override=model_override,
                depth=depth,
                ancestors=ancestors,
                execution_id=execution_id,
                parent_execution_id=parent_execution_id,
            )

    async def _run_agent_loop(
        self,
        agent_id: str,
        objective: str,
        task_id: str,
        run_id: str,
        input_data: dict[str, Any],
        model_override: BaseChatModel | None,
        depth: int,
        ancestors: list[str] | None,
        execution_id: str,
        parent_execution_id: str | None,
    ) -> tuple[AgentExecution, dict[str, Any]]:
        current_agent_id_var.set(agent_id)
        defn = registry.get(agent_id)
        current_ancestors = (ancestors or []) + [agent_id]

        from src.agents.schemas import (
            current_execution_saved_evidence_ids_var,
            current_saved_evidence_ids_var,
        )

        current_execution_saved_evidence_ids_var.set(set())
        existing_artifacts = await self.ledger.list_artifacts_for_run(run_id)
        saved_ev_ids = {a.id for a in existing_artifacts if a.type == "evidence"}
        current_saved_evidence_ids_var.set(saved_ev_ids)
        chat_model = model_override or self.get_chat_model(role=agent_id, request_timeout_s=defn.llm_request_timeout_s)
        model_name = getattr(chat_model, "model_name", getattr(chat_model, "model", "custom-model"))
        provider_name = provider_key(chat_model)
        exec_rec = AgentExecution(
            id=execution_id,
            run_id=run_id,
            task_id=task_id,
            parent_execution_id=parent_execution_id,
            agent_id=agent_id,
            objective=objective,
            status="running",
            delegation_depth=depth,
            model_provider=provider_name,
            model_name=model_name,
            budgets=ExecutionBudgets(
                tool_calls=defn.tool_calls_budget,
                model_calls=defn.model_calls_budget,
                wall_time_ms=defn.wall_time_seconds * 1000,
            ),
            budgets_used=ExecutionBudgetsUsed(),
            started_at=datetime.now(timezone.utc),
        )
        await self.ledger.save_agent_execution(exec_rec)

        await self.ledger.append_event_transactional(
            run_id=run_id,
            event_type="agent.started",
            task_id=task_id,
            execution_id=exec_rec.id,
            payload={
                "execution_id": exec_rec.id,
                "agent_id": agent_id,
                "objective": objective,
                "model": exec_rec.model_name,
            },
        )
        self_ledger = self.ledger

        class ActivityMiddleware(AgentMiddleware):
            async def abefore_model(self, state: Any, runtime: Any = None) -> Any:
                _msgs = state.get("messages", []) if isinstance(state, dict) else []
                step_num = exec_rec.budgets_used.model_calls + 1
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.activity",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    payload={
                        "execution_id": exec_rec.id,
                        "activity_type": "model_step_started",
                        "description": f"Step {step_num}: Dispatched prompt to {exec_rec.model_name} (in-flight)",
                    },
                )
                return None

            async def aafter_model(self, state: Any, runtime: Any = None) -> Any:
                exec_rec.budgets_used.model_calls += 1
                msgs = state.get("messages", []) if isinstance(state, dict) else []
                if msgs:
                    last_m = msgs[-1]
                    tc_list = getattr(last_m, "tool_calls", []) or []
                    step_desc = ""
                    if tc_list:
                        intents = [f"call {t.get('name')}" for t in tc_list]
                        step_desc = f"Action: {', '.join(intents)}"
                    elif getattr(last_m, "content", None):
                        txt = str(last_m.content).strip()
                        if txt and not txt.startswith("{"):
                            step_desc = f"Reasoning: {txt[:160]}"
                        else:
                            step_desc = f"Generated structured output deliverable ({len(txt)} chars)"
                    if step_desc:
                        await self_ledger.append_event_transactional(
                            run_id=run_id,
                            event_type="agent.activity",
                            task_id=task_id,
                            execution_id=exec_rec.id,
                            payload={
                                "execution_id": exec_rec.id,
                                "activity_type": "model_step",
                                "description": step_desc,
                            },
                        )
                    usage = getattr(last_m, "usage_metadata", None)
                    if isinstance(usage, dict):
                        tokens_in = usage.get("input_tokens")
                        tokens_out = usage.get("output_tokens")
                        if (
                            "input_tokens" in usage
                            and "output_tokens" in usage
                            and isinstance(tokens_in, int)
                            and not isinstance(tokens_in, bool)
                            and tokens_in >= 0
                            and isinstance(tokens_out, int)
                            and not isinstance(tokens_out, bool)
                            and tokens_out >= 0
                        ):
                            await self_ledger.append_event_transactional(
                                run_id=run_id,
                                event_type="agent.model_usage",
                                task_id=task_id,
                                execution_id=exec_rec.id,
                                payload={
                                    "agent_id": agent_id,
                                    "model_provider": provider_name,
                                    "model_name": model_name,
                                    "input_tokens": tokens_in,
                                    "output_tokens": tokens_out,
                                    "total_tokens": tokens_in + tokens_out,
                                },
                            )
                return None

        @wrap_tool_call
        async def intercept_tool_call(request, handler):
            lost_ev = current_lost_lease_var.get()
            if lost_ev and lost_ev.is_set():
                raise RuntimeError("Lease lost: cooperative abort at tool boundary.")
            tc = getattr(request, "tool_call", {}) or {}
            t_name = tc.get("name", "tool") if isinstance(tc, dict) else getattr(tc, "name", "tool")
            t_args = tc.get("args", {}) if isinstance(tc, dict) else getattr(tc, "args", {})
            st = time.perf_counter()
            t_id = f"tool_{uuid.uuid4().hex[:12]}"
            inv = ToolInvocation(
                id=t_id,
                execution_id=validate_id(exec_rec.id, "execution_id"),
                tool_name=t_name,
                status="running",
                duration_ms=0.0,
                sanitized_input=sanitize_data(t_args),
                output_summary="",
                started_at=datetime.now(timezone.utc),
            )
            await self_ledger.save_tool_invocation(inv, run_id=run_id)
            await self_ledger.append_event_transactional(
                run_id=run_id,
                event_type="tool.started",
                task_id=task_id,
                execution_id=exec_rec.id,
                tool_invocation_id=inv.id,
                payload={
                    "tool_invocation_id": inv.id,
                    "tool_name": t_name,
                    "sanitized_input": inv.sanitized_input,
                },
            )

            exec_rec.tool_call_count += 1
            exec_rec.budgets_used.tool_calls += 1
            try:
                out = await handler(request)
                dur = (time.perf_counter() - st) * 1000
                out_content = getattr(out, "content", str(out))
                inv.status = "completed"
                inv.duration_ms = dur
                inv.output_summary = str(out_content)[:300]
                await self_ledger.save_tool_invocation(inv, run_id=run_id)
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="tool.completed",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    tool_invocation_id=inv.id,
                    payload={
                        "tool_invocation_id": inv.id,
                        "tool_name": t_name,
                        "duration_ms": dur,
                        "summary": inv.output_summary,
                    },
                )
                return out
            except Exception as te:
                dur = (time.perf_counter() - st) * 1000
                inv.status = "failed"
                inv.duration_ms = dur
                inv.error = str(te)
                await self_ledger.save_tool_invocation(inv, run_id=run_id)
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="tool.failed",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    tool_invocation_id=inv.id,
                    payload={"tool_invocation_id": inv.id, "tool_name": t_name, "error": str(te)},
                )
                raise te

        tools = self.create_tools_for_execution(
            execution=exec_rec,
            run_id=run_id,
            depth=depth,
            ancestors=current_ancestors,
            tool_call_callback=None,
            model_override=model_override,
        )

        from src.agents.schemas import CURRICULUM_STAGE_SCHEMAS, CategoryQuestionBatchOutput

        if agent_id == "interview-generation" and input_data and "target_category" in input_data:
            schema_cls = CategoryQuestionBatchOutput
        elif agent_id == "curriculum" and input_data and "curriculum_stage" in input_data:
            schema_cls = CURRICULUM_STAGE_SCHEMAS[input_data["curriculum_stage"]]
        else:
            schema_cls = AGENT_OUTPUT_SCHEMAS.get(agent_id, AgentTaskOutput)
        tool_strategy = ToolStrategy(schema_cls, handle_errors=True)
        from langchain_core.messages import AIMessage, HumanMessage, SystemMessage

        response_tool_name = schema_cls.__name__

        evidence_floor = EVIDENCE_FLOOR_BY_AGENT.get(agent_id, 0)
        save_tools = {"read_source", "save_evidence"}

        def own_saved_ids() -> list[str]:
            from src.agents.schemas import current_execution_saved_evidence_ids_var

            return sorted(current_execution_saved_evidence_ids_var.get() or [])

        async def emit_budget_activity(description: str) -> None:
            logger.info(f"Execution {exec_rec.id} ({agent_id}): {description}")
            await self_ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.activity",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "execution_id": exec_rec.id,
                    "activity_type": "budget_control",
                    "description": description,
                },
            )

        budget_mode = {"value": "normal"}

        @wrap_model_call
        async def budget_finalizer_middleware(request, handler):
            used_tools = exec_rec.budgets_used.tool_calls
            used_models = exec_rec.budgets_used.model_calls
            rem_tools = exec_rec.budgets.tool_calls - used_tools
            rem_models = exec_rec.budgets.model_calls - used_models
            active_s = budget_clock.active_elapsed_s()
            rem_time_ratio = 1.0 - (active_s / max(1.0, budget_clock.budget_s))
            saved_ids = own_saved_ids()

            rem_time_s = budget_clock.remaining_s()
            if (
                rem_tools <= FINALIZE_RESERVED_TOOL_CALLS
                or rem_models <= FINALIZE_RESERVED_MODEL_CALLS
                or rem_time_ratio <= FINALIZE_TIME_RATIO
                or rem_time_s <= FINALIZE_MIN_WALL_TIME_RESERVE_S
            ):
                if budget_mode["value"] != "finalize":
                    budget_mode["value"] = "finalize"
                    await emit_budget_activity(
                        f"Research budget exhausted: forcing final output with {len(saved_ids)} saved evidence item(s)"
                    )
                saved_list = ", ".join(saved_ids) if saved_ids else "none"
                warn_msg = SystemMessage(
                    content=(
                        "CRITICAL: Research budget is nearly exhausted. "
                        "Stop calling research tools. You must call the final output tool now "
                        f"('{response_tool_name}') with your accumulated evidence and findings. "
                        f"You saved {len(saved_ids)} evidence item(s) in this execution: {saved_list}. "
                        "Cite only these exact IDs (or IDs from the task context); do not invent IDs."
                    )
                )
                msgs = list(request.messages) + [warn_msg]
                req = request.override(messages=msgs, tools=[])
                return await handler(req)

            missing = max(0, evidence_floor - len(saved_ids))
            if missing and rem_tools <= missing * 2 + 2:
                if budget_mode["value"] != "reserve":
                    budget_mode["value"] = "reserve"
                    await emit_budget_activity(
                        f"Research budget low: restricting to read/save tools; {missing} more evidence needed"
                    )
                reserve_msg = SystemMessage(
                    content=(
                        f"Budget reserve: you still need {missing} more saved evidence item(s) and have "
                        f"{rem_tools} tool call(s) left. web_search is disabled. Use read_source on a URL "
                        "you already found, then save_evidence with an exact quote from it."
                    )
                )
                kept = [t for t in request.tools if getattr(t, "name", None) in save_tools]
                req = request.override(messages=list(request.messages) + [reserve_msg], tools=kept)
                return await handler(req)
            return await handler(request)

        llm_retry_middleware = build_llm_retry_middleware(f"Execution {exec_rec.id} ({agent_id})")

        missing_output_nudges = {"count": 0}

        @wrap_model_call
        async def require_output_tool_middleware(request, handler):
            response = await handler(request)
            while missing_output_nudges["count"] < MAX_MISSING_OUTPUT_TOOL_NUDGES:
                ai_msg = next(
                    (m for m in reversed(getattr(response, "result", []) or []) if isinstance(m, AIMessage)),
                    None,
                )
                if ai_msg is None or ai_msg.tool_calls:
                    return response
                missing_output_nudges["count"] += 1
                logger.warning(
                    f"Execution {exec_rec.id} ({agent_id}): model returned no tool call "
                    f"(content {len(str(ai_msg.content or ''))} chars); re-prompting for {response_tool_name} "
                    f"({missing_output_nudges['count']}/{MAX_MISSING_OUTPUT_TOOL_NUDGES})"
                )
                await self_ledger.append_event_transactional(
                    run_id=run_id,
                    event_type="agent.activity",
                    task_id=task_id,
                    execution_id=exec_rec.id,
                    payload={
                        "execution_id": exec_rec.id,
                        "activity_type": "output_reprompt",
                        "description": f"No tool call returned; re-prompting for {response_tool_name}",
                    },
                )
                nudge = HumanMessage(
                    content=(
                        f"Your last reply contained no tool call. You must call {response_tool_name} "
                        "now with your complete structured output. Do not reply with plain text."
                    )
                )
                response = await handler(
                    request.override(messages=list(request.messages) + [ai_msg, nudge])
                )
            return response

        middleware = [
            ModelCallLimitMiddleware(
                run_limit=exec_rec.budgets.model_calls + FINALIZE_MODEL_CALL_HEADROOM
            ),
            ToolCallLimitMiddleware(
                run_limit=max(1, exec_rec.budgets.tool_calls - FINALIZE_RESERVED_TOOL_CALLS)
            ),
            ActivityMiddleware(),
            intercept_tool_call,
            budget_finalizer_middleware,
            require_output_tool_middleware,
            llm_retry_middleware,
            build_provider_slot_middleware(provider_key(chat_model)),
        ]

        agent = create_agent(
            model=chat_model,
            tools=tools,
            system_prompt=defn.instructions,
            response_format=tool_strategy,
            middleware=middleware,
        )
        start_perf = time.perf_counter()
        timeout_seconds = float(defn.wall_time_seconds)
        budget_clock = ExecutionBudgetClock(f"Execution {exec_rec.id} ({agent_id})", timeout_seconds)
        clock_token = current_budget_clock_var.set(budget_clock)
        try:
            try:
                async with asyncio.timeout(timeout_seconds) as timeout_cm:
                    budget_clock.start(timeout_cm)
                    user_msg = f"Task: {objective}\nContext: {json.dumps(input_data)}"
                    result = await agent.ainvoke({"messages": [{"role": "user", "content": user_msg}]})
            finally:
                current_budget_clock_var.reset(clock_token)

            exec_rec.budgets_used.wall_time_ms = (time.perf_counter() - start_perf) * 1000
            if budget_clock.excluded_s:
                logger.info(
                    f"Execution {exec_rec.id} ({agent_id}) excluded from wall budget: "
                    f"{ {k: round(v, 1) for k, v in budget_clock.excluded_by_reason.items()} }"
                )

            structured_data = None
            if isinstance(result, dict):
                structured = result.get("structured_response")
                if isinstance(structured, BaseModel):
                    exec_rec.public_summary = getattr(structured, "summary", str(structured))
                    structured_data = structured
                elif isinstance(structured, dict):
                    exec_rec.public_summary = structured.get("summary", str(structured))
                    structured_data = structured

                msgs = result.get("messages", [])
                total_in = 0
                total_out = 0
                for m in msgs:
                    usage = getattr(m, "usage_metadata", None)
                    if isinstance(usage, dict):
                        total_in += usage.get("input_tokens", 0)
                        total_out += usage.get("output_tokens", 0)
                exec_rec.budgets_used.tokens_in = total_in
                exec_rec.budgets_used.tokens_out = total_out
            if not exec_rec.public_summary or structured_data is None:
                last_m_str = str(result.get("messages", [])[-1:]) if isinstance(result, dict) else str(result)
                logger.error(
                    f"Agent {agent_id} failed to produce valid structured output for objective: {objective}. Last message: {last_m_str[:300]}"
                )
                raise ValueError(
                    f"Agent {agent_id} failed to produce valid structured output for objective: {objective}"
                )
            AGENT_ARTIFACT_TYPE_MAP = {
                "company-research": "research_report",
                "job-analysis": "finding",
                "technical-research": "research_report",
                "interview-generation": "question_set",
                "evidence-evaluator": "validation_result",
            }
            art_type = AGENT_ARTIFACT_TYPE_MAP.get(agent_id, "research_report")
            dict_content = (
                structured_data.model_dump()
                if hasattr(structured_data, "model_dump")
                else structured_data
            )
            art = Artifact(
                id=f"art_{uuid.uuid4().hex[:12]}",
                run_id=run_id,
                task_id=task_id,
                agent_execution_id=exec_rec.id,
                type=art_type,
                title=f"{defn.name} Deliverable",
                evidence_ids=list(saved_ev_ids),
                content=dict_content,
            )
            await self.ledger.save_artifact(art)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="artifact.created",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "artifact_id": art.id,
                    "type": art.type,
                    "title": art.title,
                },
            )
            exec_rec.artifact_ids.append(art.id)
            exec_rec.status = "completed"
            exec_rec.completed_at = datetime.now(timezone.utc)
            await self.ledger.save_agent_execution(exec_rec)

            total_tokens = exec_rec.budgets_used.tokens_in + exec_rec.budgets_used.tokens_out
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.completed",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={
                    "execution_id": exec_rec.id,
                    "summary": exec_rec.public_summary,
                    "tool_calls": exec_rec.tool_call_count,
                    "tokens_used": total_tokens,
                },
            )
            return exec_rec, dict_content
        except Exception as e:
            exec_rec.budgets_used.wall_time_ms = (time.perf_counter() - start_perf) * 1000
            if budget_clock.excluded_s:
                logger.info(
                    f"Execution {exec_rec.id} ({agent_id}) excluded from wall budget: "
                    f"{ {k: round(v, 1) for k, v in budget_clock.excluded_by_reason.items()} }"
                )
            exec_rec.status = "failed"
            if isinstance(e, (asyncio.TimeoutError, TimeoutError)):
                error_text = f"wall-time budget exceeded ({int(timeout_seconds)}s)"
            else:
                error_text = str(e) or e.__class__.__name__
            exec_rec.error = error_text
            exec_rec.completed_at = datetime.now(timezone.utc)
            await self.ledger.save_agent_execution(exec_rec)
            await self.ledger.append_event_transactional(
                run_id=run_id,
                event_type="agent.failed",
                task_id=task_id,
                execution_id=exec_rec.id,
                payload={"execution_id": exec_rec.id, "error": error_text},
            )
            if error_text != str(e):
                raise AgentExecutionError(error_text) from e
            raise


agent_runtime = AutonomousAgentRuntime()
