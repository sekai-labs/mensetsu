# Dual-Mode Multi-Agent Workflow Platform: Implementation Tracker

Final state as of 2026-10-14. A phase is marked `VERIFIED` only when the evidence cited below backs it. Earlier revisions of this tracker overstated progress; that history is superseded by this document.

## Status Legend

- `VERIFIED`: backed by cited, reproducible test or E2E output.
- `NOT_STARTED`: not implemented.

## Architecture As Built

The original plan called for a Postgres ledger and queue. It was replaced:

- Ledger: SurrealDB (`server/api/src/schema.surql`), written by `src/orchestration/ledger.py`.
- Queue: Redis Streams consumer groups (`src/orchestration/redis_queue.py`).
- Checkpoints: LangGraph `AsyncRedisSaver`, keyed by `thread_id = run_id`, with a TTL.
- Workers run in a separate container. The API container starts no worker.

No Temporal, Postgres, asyncpg or psycopg code or dependencies remain. The legacy `generation`, `llm`, `research` and `extractor` layers, about 4,100 lines, were deleted together with the tests that covered only them.

## Evidence

| Source | Result | When |
|---|---|---|
| `server/intelligence` pytest, full suite (`--timeout=300 --timeout-method=thread`) | 46 passed, 0 failed, 25m08s | 2026-10-14, started 11:13 |
| Tests added after that full run: `test_worker_claim_backoff.py` (3), the empty-reply reprompt tests in `test_agent_runtime_tools.py` (2), `test_concurrency_limits.py` (7) | Targeted runs: 14/14, 10/10, and 11/11 (limits + overlap + agent runtime tools); not yet part of a full run | 2026-10-14, 12:23, 13:17 and 13:49 |
| `cargo test --test workflow_authz_tests` | 9 passed, 0 failed | E2E pass 10 |
| Web vitest (`app/web/test`) | 15 files, 53 tests passed (figure from the E2E owner; no log captured under `/tmp/e2e-logs/pass10`) | E2E pass 10 |
| Playwright desktop (`tests/e2e/workflow.desktop.spec.ts`) | 9/9 passed, 14.6m | E2E pass 10 |
| Playwright mobile (`tests/e2e/workflow.mobile.spec.ts`) | 1/1 passed | E2E pass 10 |
| Live static run `run_a34bca78…` | completed, 9/9 tasks, 414 events, 12m35s | E2E pass 10 |
| Live dynamic run `run_74557aa8…` | completed, plan revision 2, 11 tasks completed and 1 skipped, 570 events, 18m57s | E2E pass 10 |
| Console probe, desktop and mobile, 5 routes each | 0 hydration errors, 0 console messages | E2E pass 10 |
| Redis key hygiene | test Redis DBSIZE 0 before and after every pytest run; E2E reset verified DBSIZE 0 | 2026-10-14 |
| LLM request timeouts | dropped from 19 (pass 8) to 0 (pass 9) after per-agent timeouts and 2-question batches | E2E pass 9 |

### Pass 10 Per-Section Counts

All API probes for both runs returned 200.

| Section | Static run | Dynamic run |
|---|---|---|
| Events | 414 | 570 |
| Task graph nodes | 9 | 12 |
| Agent executions | 17 | 20 |
| Tool invocations | 95 | 132 |
| Artifacts | 33 | 40 |
| Delegations | 1 | 2 |
| Role requirements | 7 | 8 |
| Match strengths / gaps | 4 / 4 | 4 / 4 |
| Company sources | 16 | 20 |
| Workspace sources | 16 | 36 |
| Claims | 8 | 16 |
| Knowledge graph nodes | 14 | 28 |
| Learning units | 6 | 6 |
| Interview questions | 17 | 17 |

## Phase Tracker

### Phase 1: Audit & System Discovery: `VERIFIED`

- Removed the legacy activity and LLM-router layer, the Playwright fallback, and the unused dependencies (`anthropic`, `cryptography`, `langchain-anthropic`, `playwright`).
- A clean-venv import gate (`uv sync --frozen --no-dev`, then `import src.main, src.worker`) passes.
- `ruff check .` is clean.
- No comments or docstrings remain in `src` or `tests`.

### Phase 2: Contracts & Public v1 Specification: `VERIFIED`

- Pydantic v2 contracts live in `src/contracts/v1.py`; the exported schema is `server/api/contracts/schema/workflows.v1.json`.
- Evidence: `test_contracts_staleness`, `test_fixtures_staleness`, `test_router_routes_contract`; web `workflow-contract.test.ts`, `contracts.test.ts`, `workflow-openapi.test.ts`; pass 10 `contracts-check.log` and `schema-parity.log`.

### Phase 3: SurrealDB Ledger & Redis Streams Queue: `VERIFIED`

- Events get a per-run monotonic `seq`. All SurrealQL is parameterized. Delegations are stored as `RELATE` edges.
- An event is appended only after the state it announces is persisted. The planner saves its task rows before `plan.created`, and the worker marks a run failed before `workflow.failed`.
- Evidence: `test_concurrency_events`, `test_delegation_injection_regression`, `test_schema_records`, `test_redis_checkpointer`; sequences are contiguous 1..N in both pass 10 runs.

### Phase 4: Static Workflow Engine: `VERIFIED`

- Graph: normalize, then a fan-out to company, job and technical research, then synthesize, curriculum, question generation (8 concurrent 2-question batches with at least 4 per category), validation, and persist.
- Resume extraction is grounded: every evidence quote must appear verbatim in the CV, and a failed extraction fails the task instead of falling back to invented data.
- Evidence: `test_e2e_static_workflow`, `test_no_fabricated_literals`, `test_task_level_retry`, `test_activities`; pass 10 static run 9/9 tasks.

### Phase 5: Dynamic Workflow Engine: `VERIFIED`

- A planner produces a validated DAG (known agents, unique IDs, existing dependencies, acyclic, bounded size and depth). Tasks are dispatched with LangGraph `Send`.
- There is one bounded revision. It never drops required tasks, and an invalid revision is rejected without failing the run.
- Evidence: `test_dynamic_workflow`; Playwright desktop case 9; pass 10 dynamic run at plan revision 2.

### Phase 6: Agent Runtime & Registry: `VERIFIED`

- LangChain v1 `create_agent` with model-call and tool-call limits, a budget finalizer, and argument-alias normalization.
- Bounded LLM retries: timeouts, 429 and 5xx/524 are retried. The request timeout is set per agent (interview-generation and curriculum use 180 s).
- Re-prompt on a missing output tool call: a reply with no tool call triggers a request to call the output tool, at most twice, within the same execution.
- Secret and PII sanitizer.
- Evidence: `test_agent_runtime_tools`, `test_budget_finalizer`, `test_registry_strict`, `test_sanitizer`, `test_injection_safety`, `test_security_defenses`, `test_provenance_strict`, `test_provenance_validation`.

### Phase 7: Constrained Delegation & Parallelism Bounds: `VERIFIED`

- Delegation permission, depth, child-count and cycle limits are enforced and persisted.
- `src/orchestration/limits.py` adds process-wide semaphores. Executions acquire global (`MAX_CONCURRENT_AGENTS`, 8), then per-run (`MAX_CONCURRENT_AGENTS_PER_RUN`, 4), then per-agent (`max_concurrency` in agent.yaml, 3), in that fixed order. Each LLM call also takes a per-provider slot (`LLM_MAX_INFLIGHT`, 6) inside the retry middleware, so a retry waiting out its backoff holds no slot.
- Slots are async context managers, so they are released on error and on cancellation. Per-run semaphores are created on first use and removed when their last execution ends.
- A delegated child takes only its agent slot, because its parent already holds the global and per-run slots; this avoids a parent deadlocking on its own child.
- A wait longer than 1 s emits `agent.activity` "Waiting for execution slot (<limit>)".
- Evidence: `test_concurrency_limits` 7/7. It checks that observed peaks never exceed each limit (global 3, per-run 2, per-agent 1 and 3, provider 2), that 3 executions in one run still overlap, that cancelling releases slots and empties the run map, the wait notice above 1 s and none below it, and that a nested delegation does not deadlock. Also `test_delegation_limits` and `test_concurrency_overlap`; pass 10 runs recorded 1 and 2 delegations.

### Phase 8: Persistence, Crash Recovery & Leases: `VERIFIED`

- Redis lease with compare-and-set heartbeat refresh. The lease counts as lost on a failed refresh plus a failed retry, on 3 consecutive exceptions, or when no renewal happened within the TTL.
- The worker fails closed at startup. Transient Redis errors in the claim loop log WARN with backoff from 1 s up to 10 s, and ERROR after 5 consecutive failures.
- Redis clients use socket timeouts (10 s, longer than the 2 s blocking read), keepalive, health checks and retry-on-timeout.
- Evidence: `test_subprocess_kill_recovery` (a real `SIGKILL` mid-run; the run resumes with no task re-executed), `test_worker_crash_recovery`, `test_worker_lease_race`, `test_worker_concurrent_lease_isolation`, `test_worker_retry_classification`, `test_worker_fail_closed`, `test_worker_readiness_signal`, `test_worker_claim_backoff`.

### Phase 9: Unified Ingestion Path & Cutover: `VERIFIED`

- `POST /internal/v1/workflows/runs` is the only way to create a run. Legacy `/api/v1/workflows*`, `/internal/v1/pipelines/runs` and `BackgroundTasks` paths are gone (no matches in `src`).
- Bearer-token auth uses a constant-time comparison. `/health/ready` reports SurrealDB and Redis status.
- Evidence: `test_auth_matrix`, `test_runs_isolation_404`, `test_router_routes_contract`.

### Phase 10: Rust API Gateway & SSE Proxy: `VERIFIED`

- Owner-scoped proxy routes. The SSE proxy maps `Last-Event-ID` to `after_seq` and preserves `id`/`event`/data framing.
- Evidence: `workflow_authz_tests` 9/9, including foreign or unknown runs returning 404 on every route, the SSE translation, and a 502 on upstream contract mismatch.

### Phase 11: Nuxt Runs List & Detail Views: `VERIFIED`

- Events are replayed past `last_seq`, with gap recovery and SSE subscription. Progress and phase come from ledger counts.
- Evidence: vitest 15 files/53 tests; Playwright desktop cases 1–3 (create, mid-run reload with no duplicate events, completion with overlapping parallel intervals).

### Phase 12: Task DAG & Delegation Graphs: `VERIFIED`

- Evidence: Playwright desktop case 6 (delegation view, or its documented empty state) and case 9 (graph expansion and revision marker after `plan.created`).

### Phase 13: Inspectors, Logs & Artifact Viewers: `VERIFIED`

- Evidence: Playwright desktop cases 4 (agent inspector with sanitized tool inputs), 5 (rendered log lines match the logs endpoint), 7 (artifact provenance and navigation) and 8 (every workspace section non-empty); the mobile case (inspector sheet and separate logs tab).

### Phase 14: End-to-End, Kill-Recovery & Final Cutover: `VERIFIED`

- Evidence: Playwright pass 10 (desktop 9/9, mobile 1/1); live static and dynamic runs completed; 0 hydration errors; `test_subprocess_kill_recovery`.

## Known Limitations

- **Web search:** SearXNG relies on the bing and yahoo engines (qwant was removed after CAPTCHAs). Result quality and availability depend on how those engines treat this host's IP.
- **Empty-reply re-prompt:** only unit-tested. Pass 9 and pass 10 never triggered it live.
- **Test Redis memory cap:** the `mensetsu-test-redis` container runs without `--maxmemory`. The pytest session sets `maxmemory 1gb` through `CONFIG SET`; the container itself should be started with `--maxmemory 1gb --maxmemory-policy noeviction`, matching the compose `redis` service.
- **E2E duration:** the desktop Playwright file takes about 14.5 minutes, of which the static-run completion case takes about 12.2 minutes. A full static run takes about 12.5 minutes and a dynamic run about 19 minutes against the live LLM gateway.
- **Pytest coverage of the latest tests:** the 46/46 full run predates `test_worker_claim_backoff.py`, the empty-reply tests, `test_concurrency_limits.py`, `test_execution_lifecycle.py`, the degraded-run test and the attempt-continuation test. Those passed only in targeted runs (latest: 27/27 on 2026-10-14 15:55).
- **Concurrency limits are per process:** each worker process enforces its own limits. With several worker containers the effective global cap is the per-process value times the number of workers. The limiter has not yet been exercised in a live E2E pass.
- **Agent wall budget is active time:** an execution's deadline pauses while it waits for a provider slot or a delegated child, so a parent's total elapsed time can reach its own budget plus its children's caps. Delegation is refused when less than the child's cap plus 90 s of active budget remains, and research is forced to finalize at 85% of the active budget.
