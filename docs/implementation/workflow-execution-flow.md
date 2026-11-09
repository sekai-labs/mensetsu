# Mensetsu Workflow Execution Flow

This document describes the **implemented** execution path from a REST request to a durable workflow run. It covers the three trigger paths for the Trivita AI / Senior DevOps Engineer scenario:

1. **Workspace create, static** — `POST /api/v1/workspaces` without a custom prompt (or with `workflow_mode: "static"`).
2. **Workspace create, hybrid** — `POST /api/v1/workspaces` from the web wizard with `workflow_mode: "hybrid"`.
3. **Manual dynamic run** — `POST /api/v1/workflows/runs` against an existing workspace with `workflow_id: "interview-preparation-dynamic"`.

The paths share the same service boundary: the Rust API owns REST, authorization, workspace/run persistence queries, and API contracts; the Python Intelligence service owns the gRPC control plane, run ledger, Redis queue, and LangGraph execution. There is no internal REST call from Rust to Python.

---

## 1. Trigger paths at a glance

| Path | REST entry point | Selected `workflow_id` | Response contains a run ID? |
|---|---|---|---|
| Workspace create, static | `POST /api/v1/workspaces` | `interview-preparation-static` | **No** — returns `WorkspaceResponseDto` |
| Workspace create, hybrid | `POST /api/v1/workspaces` | `interview-preparation-hybrid` | **No** — returns `WorkspaceResponseDto` |
| Manual dynamic run | `POST /api/v1/workflows/runs` | `interview-preparation-dynamic` | **Yes** — returns `WorkflowRun.id` |

Workspace creation triggers a run as a side effect. The API does not return that run's ID, so callers must list runs for the workspace (Section 4). The manual run endpoint is the only trigger that returns the created run synchronously.

For the Trivita AI / Senior DevOps Engineer scenario:

- Native clients (`app/mobile/android/.../WorkspaceListScreen.kt`, `app/mobile/ios/Mensetsu/Services/WorkspaceService.swift`, `app/desktop/src/services/api.rs`) send company, role, resume, and JD without `prompt` or `workflow_mode`; `Workspace::new` therefore defaults to `static`.
- The web wizard (`app/web/pages/interviews/new.vue`) explicitly sends `workflow_mode: "hybrid"` and maps the notes field to `prompt`.
- The Workflows page dialog (`app/web/components/workflow/CreateWorkflowRunDialog.vue`) creates a manual run for an existing workspace; selecting Dynamic sends `workflow_id: "interview-preparation-dynamic"`.

---

## 2. End-to-end architecture

```mermaid
sequenceDiagram
    autonumber
    actor Client as Client / Browser
    participant REST as Rust Axum REST API (:3000)
    participant SDB as SurrealDB (:8000)
    participant Control as Intelligence gRPC DataControlService (:50051)
    participant Ledger as SurrealDB Ledger
    participant Queue as Redis Stream (mensetsu:jobs:stream)
    participant Worker as DurableJobWorker
    participant Graph as LangGraph (static / hybrid / dynamic)
    participant Stream as WorkflowEventStreamService
    participant LLM as LLM gateway / SearXNG

    Client->>REST: POST /api/v1/workspaces or POST /api/v1/workflows/runs
    alt Workspace create
        REST->>SDB: BEGIN; UPSERT workspace, resume, job_description; COMMIT
        REST->>SDB: SELECT workspace, resume, job_description, LLM settings
    else Manual run
        REST->>SDB: verify_workspace_owner
        REST->>SDB: SELECT workspace, resume, job_description, LLM settings
    end
    REST->>Control: CreateRunCommand (over StreamControl, internal bearer token)
    Control->>Ledger: create workflow_run (status=pending, phase=queued)
    Control->>Ledger: append workflow.created (seq=1)
    Control->>Queue: XADD job (or idempotent Lua enqueue)
    Control-->>REST: 201 + WorkflowRun snapshot
    REST-->>Client: 201 (WorkspaceResponseDto or WorkflowRun)

    Worker->>Queue: XREADGROUP / XAUTOCLAIM claim
    Worker->>Queue: acquire lease + heartbeat every 5s
    Worker->>Ledger: status=running, phase=running, workflow.started
    Worker->>Graph: build and invoke selected graph
    Graph->>LLM: agent calls and tool loops
    Graph->>Ledger: task.started, agent.activity, tool.*, task.completed
    Graph->>Ledger: workflow.completed / workflow.failed
    Graph->>SDB: persist match, graph, learning units, questions

    Client->>REST: GET /api/v1/workflows/runs/{run_id}/events
    REST->>Stream: gRPC StreamWorkflowEvents (subscribe, after_seq)
    Stream-->>REST: ExecutionEventFrame
    REST-->>Client: SSE (id=seq, event=type, data=JSON payload)
```

### 2.1 Control plane is gRPC, not REST

- `server/api/src/interfaces/grpc/client.rs` defines `GrpcClient`. It opens one long-lived bidirectional `DataControlService.StreamControl` stream and multiplexes commands by `correlation_id`.
- `contracts/grpc/proto/v1/control.proto` defines `CreateRunCommand`, `ResumePlanCommand`, `ListRunsCommand`, `GetRunCommand`, and the other control commands.
- `server/intelligence/src/grpc/control_servicer.py` (`DataControlServicer.StreamControl`) dispatches each command to `WorkflowService` in `server/intelligence/src/orchestration/service.py`.
- `server/intelligence/src/main.py` starts the gRPC server; `INTELLIGENCE_PORT` defaults to `50051`. The current source entry point starts only gRPC; the legacy internal REST router is not present under `server/intelligence/src`.

The Rust API authenticates to Intelligence with `Authorization: Bearer ${INTERNAL_SERVICE_TOKEN}`. The gRPC `GrpcAuthInterceptor` (`server/intelligence/src/grpc/auth_interceptor.py`) validates that token and only exempts gRPC health checks. This is a service-to-service trust boundary, not a per-user authorization boundary.

### 2.2 Event plane

- `contracts/grpc/proto/v1/streaming.proto` defines `WorkflowEventStreamService.StreamWorkflowEvents`.
- `server/intelligence/src/grpc/streaming_servicer.py` replays ledger history after `after_seq`, then follows live Redis-published events, and stops on `workflow.completed`, `workflow.failed`, or `workflow.cancelled`.
- `server/api/src/interfaces/grpc/client.rs::stream_run_events` converts each `ExecutionEventFrame` to an SSE event: `id` = sequence number, `event` = event type, `data` = the event JSON payload.
- The REST route is `GET /api/v1/workflows/runs/{run_id}/events` in `server/api/src/interfaces/http/workflows.rs`.

---

## 3. Authentication and authorization

### 3.1 User REST requests

`server/api/src/interfaces/http/extractors/authenticated_user.rs` accepts either:

- `Authorization: Bearer <access token>`, or
- the `mensetsu_access` cookie.

It verifies the JWT access token, loads the session from SurrealDB, rejects an inactive session, and rejects a session/user mismatch.

- `POST /api/v1/workspaces` and `POST /api/v1/workflows/runs` require `AuthenticatedUser`.
- `trigger_workflow_run` calls `verify_workspace_owner` (`server/api/src/interfaces/http/workspaces/auth_check.rs`) before forwarding the command.
- Run read/stream/cancel endpoints call `WorkflowService::authorize_run` (`server/api/src/contexts/workflow/application/mod.rs`), which resolves the run's workspace and requires the workspace owner to match the caller. Non-owners receive `404` for run endpoints.
- `GET /api/v1/workflows/runs` validates `workspace_id` ownership before querying.

### 3.2 Internal service requests

- Rust → Intelligence gRPC: `INTERNAL_SERVICE_TOKEN`.
- Intelligence → Rust resume-storage gRPC: the same token, sent by `static_input.py` when a resume upload must be downloaded.
- The public REST API never exposes the internal token; the gRPC port is an internal service endpoint, not a client API.

---

## 4. Run IDs and run discovery

### 4.1 Endpoints that return a run ID

| Endpoint | Response | Run ID location |
|---|---|---|
| `POST /api/v1/workflows/runs` | `201 WorkflowRun` | `id` (normalized, bare ID) |
| `GET /api/v1/workflows/runs` | `200 RunsPageResponse` | each `items[].id` |
| `GET /api/v1/workflows/runs/{run_id}` | `200 WorkflowRun` | `id` |
| `GET /api/v1/workflows/runs/{run_id}/...` | run-scoped resource | path `run_id` |

`POST /api/v1/workflows/runs` accepts `Idempotency-Key`; when it is used and the request is replayed, the same `WorkflowRun.id` is returned.

### 4.2 Endpoints that do **not** return a run ID

| Endpoint | Returns | Notes |
|---|---|---|
| `POST /api/v1/workspaces` | `201 WorkspaceResponseDto` | Creates a workspace and triggers a run, but the DTO has no run field. |
| `GET /api/v1/workspaces` / `GET /api/v1/workspaces/{id}` | workspace DTOs | No run field. |
| `PATCH /api/v1/workspaces/{id}/objective` | `200 {"workspace_id", "current_objective", "status"}` | Targets the latest run; does not create one. |
| `GET /api/v1/workspaces/{id}/events` | workspace progress SSE | Polls workspace phase/progress, not run events. |
| `POST /api/v1/workspaces/{id}/import-jd` | JD import result | Does not create a run. |

### 4.3 Discovering the run created by a workspace

Call `GET /api/v1/workflows/runs?workspace_id=<workspace_id>&limit=1`. The Rust repository (`server/api/src/contexts/workflow/infrastructure/repository.rs`) orders by `created_at DESC, id DESC`, so the first item is the newest run for that workspace.

This is the pattern used by `app/web/pages/interviews/[id]/research.vue`: it lists runs for the workspace with `limit: 1` and uses `items[0].id` as the active run. If a workspace has multiple runs, this returns the newest one, which may be a later manual run rather than the run created with the workspace.

The list endpoint accepts `workspace_id`, `status`, `type` (`static`, `dynamic`, `hybrid`), `limit` (1–100, default 50), and `cursor`. It returns `next_cursor` only when more rows exist.

---

## 5. Trigger path 1: workspace create, static

### 5.1 Request and mode selection

`POST /api/v1/workspaces` takes `CreateWorkspaceRequestDto` (`server/api/src/contexts/workspace/application/dtos.rs`). The DTO is strict: unknown fields are rejected.

Relevant fields:

```json
{
  "company_name": "Trivita AI",
  "role_title": "Senior DevOps Engineer",
  "resume_text": "...",
  "job_description": "...",
  "company_urls": ["https://trivita.ai"],
  "seniority": "senior",
  "workflow_mode": "static",
  "prompt": null
}
```

`Workspace::new` (`server/api/src/contexts/workspace/domain/mod.rs`) resolves the mode as follows:

- explicit `workflow_mode` if provided;
- otherwise `"hybrid"` when `prompt` is present;
- otherwise `"static"`.

`create_workspace` (`server/api/src/interfaces/http/workspaces.rs`) maps the resolved mode to a workflow ID:

```text
"static"  -> interview-preparation-static
"dynamic" -> interview-preparation-dynamic
anything else -> interview-preparation-hybrid
```

Therefore the static path is reached by:

- native clients that omit both `prompt` and `workflow_mode`, or
- any client that sends `workflow_mode: "static"`.

An unrecognized `workflow_mode` string does **not** return `400`; it falls through to hybrid.

### 5.2 Data writes

The handler builds the workspace and executes one SurrealDB transaction:

- `workspace:<ws_id>` with `company_name`, `role_title`, `status = "pending"`, `current_phase = "preparing_input"`, `progress_percent = 0`, `seniority`, `target_date`, `prep_time_weeks`, `user_prompt`, `current_objective`, `workflow_mode`, `company_urls`, timestamps.
- `resume:res_<ws_id>` with `raw_text` when `resume_text` is supplied, or with `source_upload_id`/`source_object_key`/`source_filename` when `resume_upload_id` is supplied.
- `job_description:jd_<ws_id>` with `raw_text` and/or `jd_url` when either is supplied.

For `resume_upload_id`, the transaction also performs a compare-and-swap attachment:

```text
resume_upload must be owned by the caller, status = "completed",
workspace_id is NONE/NULL, and still match object_key.
```

If zero rows are updated, the transaction throws `Resume upload CAS attachment conflict` and the REST layer returns `409`.

### 5.3 Trigger

After the transaction, `create_workspace` calls:

```rust
start_workspace_run(&state, &user_id, &ws_id, "interview-preparation-static".to_string(), Some(&ws_id))
```

`start_workspace_run` reads the stored workspace, resume, and JD, resolves the LLM configuration, builds `CreateRunPayload`, and calls `state.grpc_client.create_run(&payload, idempotency_key)`.

The `idempotency_key` here is the newly generated workspace ID. Because every POST generates a new workspace ID, this does not make repeated HTTP requests idempotent. See Section 9.

### 5.4 Execution

The worker (`server/intelligence/src/orchestration/runtime/worker.py`) selects `build_static_workflow` for `interview-preparation-static` and sets the initial progress estimate to `STATIC_TASK_COUNT = 9`.

The static graph (`server/intelligence/src/orchestration/static_workflow.py`) is deterministic:

```text
normalize
  ├─ company_research
  ├─ job_analysis
  └─ technical_research
        └─ synthesize
             └─ curriculum
                  └─ generate_questions
                       └─ validate_questions
                            └─ persist_outputs
```

The nine task records are:

1. `task_normalize_<run8>` — CV/JD normalization; owner `job-analysis`.
2. `task_company_<run8>` — company research.
3. `task_job_<run8>` — job analysis.
4. `task_tech_<run8>` — technical research.
5. `task_synth_<run8>` — evidence evaluator / match analysis.
6. `task_curriculum_<run8>` — knowledge graph and learning units.
7. `task_qgen_<run8>` — question generation.
8. `task_qval_<run8>` — question validation.
9. `task_persist_<run8>` — domain-entity persistence and finalization.

Static specifics:

- `normalize` uses `resume_upload_id` when present. It calls the Rust resume-storage gRPC service at `API_GRPC_ENDPOINT` (or `API_GRPC_BIND`, default `api:50052`), downloads the PDF from the returned URL only when the URL origin strictly matches `S3_DOWNLOAD_ALLOWED_ORIGIN`, and enforces a 5 MiB limit.
- If no resume text and no upload are supplied, `normalize` substitutes a generic candidate description instead of failing.
- If no JD text is supplied but `jd_url` is present, `normalize` fetches and sanitizes the URL through the SSRF-validated reader. An empty JD is not rejected at create time.
- Question generation targets 7 categories × 3 batches × 5 questions = 105 questions and requires at least 15 per category; the persistence check requires at least 15 persisted questions.
- Finalization requires at least 12 skills, 10 skill relations, 6 learning units, and 15 questions, then marks the run `completed` and the workspace `ready` / `100%`.

### 5.5 Response and follow-up

The response is `201 Created` with `WorkspaceResponseDto`. It contains the workspace `id`, `status`, `current_phase`, and `progress_percent`, but no `workflow_run` ID. The web wizard uses the returned workspace `id` to navigate to `/interviews/{cleanId}/overview`; it does not navigate to a run page.

If `start_workspace_run` fails, `create_workspace` logs the error, marks the workspace `failed` / `0%`, and still returns `201 Created` with `status = "failed"`. Callers must inspect the response or query the workspace; the HTTP status alone does not prove the run started.

To obtain the run ID, list runs for the workspace:

```http
GET /api/v1/workflows/runs?workspace_id=<ws_id>&limit=1
```

or connect to `GET /api/v1/workspaces/{id}/events` for coarse workspace progress until the run writes `ready`.

---

## 6. Trigger path 2: workspace create, hybrid

### 6.1 Request and mode selection

The web wizard (`app/web/pages/interviews/new.vue`) sends:

```json
{
  "company_name": "Trivita AI",
  "role_title": "Senior DevOps Engineer",
  "workflow_mode": "hybrid",
  "company_urls": ["..."],
  "seniority": "senior",
  "resume_text": "...",
  "job_description": "...",
  "prompt": "..."
}
```

The explicit `workflow_mode: "hybrid"` wins over the prompt-based default. `Workspace::new` also resolves to hybrid when `prompt` is present and `workflow_mode` is omitted; the web wizard makes it explicit. `create_workspace` maps it to `interview-preparation-hybrid` and calls `start_workspace_run(..., Some(&ws_id))`.

The same data-write, authorization, and run-discovery rules as Section 5 apply. The response is again `WorkspaceResponseDto` with no run ID.

### 6.2 Execution: what “hybrid” actually does

The worker selects `build_hybrid_workflow` (`server/intelligence/src/orchestration/hybrid_workflow.py`). The graph is:

```text
route_and_normalize
  └─ planner
       └─ dynamic_router
            ├─ execute_dynamic_task (0..N ready tasks)
            └─ synthesize
                 └─ curriculum
                      └─ interview
                           └─ finalize
```

Actual behavior in the current code:

1. **`route_and_normalize`**
   - Sets phase `analyzing_input` at 5%.
   - Calls `route_workflow_request` (`server/intelligence/src/agents/workflow_router.py`) with the prompt, company, role, resume, JD, and seniority.
   - The router returns a `WorkflowRoutingDecision` with `selected_workflow`, `confidence`, `reasoning`, `extracted_company`, and `extracted_role`.
   - Only `extracted_company` and `extracted_role` are consumed. `selected_workflow` is written to a `routing_decision` artifact but is **not** used to switch to the static or dynamic graph. The graph remains hybrid.
   - Runs the static workflow in `normalize_only` mode to normalize the CV/JD.
   - Emits an initial `plan.created` event describing three fixed research tasks.

2. **`planner`**
   - Creates a `task_plan_<run8>_v<revision>` record.
   - Builds a fixed three-agent plan (`company-research`, `technical-research`, `job-analysis`), minus IDs already completed, all with no dependencies and `required = true`.
   - Validates that plan with `validate_plan_dag`.
   - It does **not** invoke the LLM planner agent. The hybrid workflow's adaptivity is limited to the router's company/role extraction and the prompt included in each research objective.

3. **`execute_dynamic_task`**
   - The `dynamic_router` sends all ready tasks concurrently using LangGraph `Send`.
   - Each task runs through `agent_runtime.execute_agent_loop` with the resolved agent ID and an objective derived from the user prompt.
   - Task failures are recorded; completed task IDs and research results are merged into the state.

4. **`synthesize`**
   - Runs `evidence-evaluator` over the candidate CV, JD requirements, company intelligence, technical stack, and missing-research list.
   - Writes match analysis.

5. **`curriculum`**
   - Runs the `curriculum` agent to build the knowledge graph, then generates at least 6 learning units (`CURRICULUM_MIN_UNITS = 6`) with concurrency 3.

6. **`interview`**
   - Runs `interview-generation` in parallel batches: 7 categories × 5 batches × 3 questions, concurrency 3.
   - Requires at least 15 questions per category and at least 100 total; in practice the category check requires 105.
   - Task fails if the question bank is incomplete.

7. **`finalize`**
   - Persists resume, JD, company research, sources, claims, match analysis, curriculum, learning units, and questions.
   - Marks the run `completed`, sets `degraded`/`degraded_reasons` when research or generation failed, and marks the workspace `ready` / `100%`.

The worker's initial progress estimate for hybrid is `6` tasks (`is_hybrid -> 6`). Task records in the ledger include the planner task plus the three research tasks, synthesis, curriculum, and questions; finalization is a node without its own task record.

### 6.3 Duplicate revision-1 plan event

`route_and_normalize` emits `plan.created` for revision 1 with three fixed tasks. `planner` also emits `plan.created` for revision 1 with the same three tasks after validation. Consumers should treat both as planning metadata for revision 1; there is no separate plan revision yet.

---

## 7. Trigger path 3: manual dynamic run

### 7.1 Request

```http
POST /api/v1/workflows/runs
Authorization: Bearer <access token>
Idempotency-Key: <optional>
Content-Type: application/json

{
  "workspace_id": "<existing workspace id>",
  "workflow_id": "interview-preparation-dynamic"
}
```

`TriggerWorkflowRunRequestDto` (`server/api/src/interfaces/http/workflows.rs`) is strict: only `workspace_id` and `workflow_id` are accepted; unknown fields return `422`.

The handler:

1. Normalizes `workspace_id` (strips `workspace:` and backticks).
2. Calls `verify_workspace_owner`.
3. Reads `Idempotency-Key` from the request headers.
4. Calls `start_workspace_run` with the requested `workflow_id`.
5. Normalizes record IDs in the returned run and returns `201 Created`.

`start_workspace_run` re-reads the workspace, resume, and JD from SurrealDB. It does not create or modify workspace data. It resolves LLM configuration and passes the workspace's `workflow_mode` along, but the worker selects the graph from `workflow_id`, not `workflow_mode`.

The manual endpoint can create any of the three known workflow IDs. The web dialog currently offers only Static and Dynamic; hybrid manual runs must be requested through the API or another client.

### 7.2 Execution: dynamic graph

`build_dynamic_workflow` (`server/intelligence/src/orchestration/dynamic_workflow.py`) compiles:

```text
normalize
  └─ plan
       └─ join
            ├─ execute_dynamic_task (ready tasks, parallel)
            ├─ revise (after root tasks are finished)
            └─ outputs
```

1. **`normalize`** — reuses the static `normalize_only` graph. This is where PDF download, JD URL fetch, and CV/JD parsing happen.
2. **`plan`** — invokes the `planner` agent with company, role, URLs, JD, seniority, plannable agents, and limits. The planner returns a task DAG. `validate_plan_dag` enforces:
   - `MAX_PLAN_TASKS = 15`
   - `MAX_RESEARCH_TASKS_PER_REVISION = 4`
   - `MAX_PLAN_DEPTH = 4`
   - `MAX_PLAN_TOOL_CALLS = 240`
   - unique, non-empty task IDs matching the ID pattern
   - known agents only, and only `company-research`, `job-analysis`, `technical-research` are plannable
   - acyclic dependencies with no duplicate or missing dependency IDs
   - at least one required company/technical research task when research tasks are present
   - `job-analysis` tasks must be `required`
   The planner gets two attempts; a rejected plan is re-prompted with `previous_plan_rejected`.
3. **`join` / `route_from_join`** — dispatches all tasks whose dependencies are complete via `Send`, or moves to `revise` once all root tasks are finished, or to `outputs` when no tasks remain. A stalled DAG raises an error.
4. **`execute_dynamic_task`** — runs the assigned agent through `agent_runtime.execute_agent_loop`, with two task attempts. A required task failure fails the run; an optional task failure records `missing_research` and continues.
5. **`revise`** — this is a **coverage repair**, not an LLM plan review:
   - It removes pending, non-required tasks for agents that already completed and are not dependencies of other pending tasks.
   - It adds one task for each missing plannable agent (`company-research`, `job-analysis`, `technical-research`).
   - It validates the revised plan; if invalid, it keeps the previous plan and records a warning log.
   - On change, it increments `plan_revision`, emits `plan.revised`, `task.added`, and `task.removed`, and sets phase `plan_final`.
6. **`outputs`** — requires `job` results and at least one of `company` or `tech`, then invokes the static workflow in `outputs_only` mode. That runs `synthesize -> curriculum -> generate_questions -> validate_questions -> persist_outputs` with the dynamic run's research results.

### 7.3 Response

The endpoint returns `201 Created` with `WorkflowRun` (`server/api/src/interfaces/http/workflow_dtos.rs`), including:

- `id` — the run ID, normalizable to a bare `run_...` form;
- `workflow_id`, `workflow_name`, `workspace_id`, `user_id`;
- `type` — `static`, `dynamic`, or `hybrid`;
- `status` — initially `pending`;
- `current_phase` — initially `queued`;
- `plan_revision`, `progress`, `last_seq`, timestamps.

No run ID listing is needed for this path. The Workflows page uses the returned `id` to navigate to `/workflows/runs/{run.id}`.

---

## 8. Data and runtime prerequisites

### 8.1 Data prerequisites

All paths read from the same SurrealDB records:

| Record | Fields read / behavior |
|---|---|
| `workspace:<ws_id>` | `company_name`, `role_title`, `seniority`, `company_urls`, `user_prompt`, `workflow_mode`, `user_id` |
| `resume:res_<ws_id>` | `raw_text` for pasted text, or `source_upload_id`/`source_object_key`/`source_filename` for PDF uploads |
| `job_description:jd_<ws_id>` | `raw_text` and/or `jd_url` |
| `resume_upload:<upload_id>` | `user_id` matches the caller, `status = "completed"`, not attached to another workspace |

Workspace creation requires either `company_name` or `prompt`, and either `role_title` or `prompt`. It does not require a resume, JD, or URL. `trigger_workflow_run` also does not gate on data completeness. Consequences:

- A run can start with a generic resume description if neither resume text nor upload exists.
- A run can start with an empty JD if neither `job_description` nor `jd_url` exists; job-analysis then has little or no input and may fail.
- A run can start before a workspace is `ready`; there is no active-run or readiness check.

### 8.2 LLM configuration

`start_workspace_run` calls `resolve_llm_config` (`server/api/src/interfaces/http/workspaces.rs`). It collects:

- `user_llm_provider` rows for the user;
- enabled `system_llm_provider` rows;
- `user_model_routing.slots`;
- legacy `user_llm_config` (shared config if the user has none).

Keys are decrypted with `ENCRYPTION_KEY` and passed to Intelligence as `llm_config_json`. `server/intelligence/src/agents/model_router.py` resolves credentials in this order:

1. user's personal provider key;
2. admin/system provider key;
3. legacy config;
4. environment fallback (`LLM_OPENAI_API_KEY`/`OPENAI_API_KEY`, `LLM_OPENAI_BASE_URL`/`OPENAI_BASE_URL`).

The per-role slots are `router`, `planner`, `research`, and `generation`.

### 8.3 Runtime services and environment

| Component | Requirement |
|---|---|
| Rust REST API | Listening on `HOST:PORT` (default `0.0.0.0:3000`); `INTELLIGENCE_GRPC_ENDPOINT` (default `http://127.0.0.1:50051`); `INTERNAL_SERVICE_TOKEN`; `JWT_SECRET`; `ENCRYPTION_KEY`; SurrealDB settings |
| Intelligence gRPC | `INTELLIGENCE_PORT` (default `50051`) and `INTELLIGENCE_HOST`; `INTERNAL_SERVICE_TOKEN`; `SURREAL_ENDPOINT`; `REDIS_URL` |
| Durable worker | `python -m src.worker`; `SURREAL_ENDPOINT`; `REDIS_URL`; `SEARXNG_URL`; `CHECKPOINT_TTL_MINUTES`; `INTERNAL_SERVICE_TOKEN`; `API_GRPC_ENDPOINT`; `S3_DOWNLOAD_ALLOWED_ORIGIN` |
| SurrealDB | `workflow_run`, `workflow_task`, `execution_event`, `execution_log`, `artifact`, `agent_execution`, `run_idempotency`, and domain tables |
| Redis | Job stream `mensetsu:jobs:stream`, consumer group `mensetsu-intelligence-workers`, lease keys `mensetsu:lease:<run_id>`, LangGraph checkpoints |
| SearXNG | `web_search` tool backend |
| MinIO/S3 | Resume uploads and the Rust resume-storage gRPC service on `API_GRPC_BIND` (default `127.0.0.1:50052`, compose `0.0.0.0:50052`) |
| LLM gateway | Reachable from the worker; worker startup is fail-closed |

Worker startup (`_require_llm_env` in `runtime/worker.py`) refuses to start when `LLM_OPENAI_BASE_URL` or `LLM_OPENAI_API_KEY` is unset or empty, even if per-user provider config exists in SurrealDB.

Redis URL defaulting differs by process: `DurableJobWorker` defaults to `redis://127.0.0.1:16379`, while `docker-compose.yml` sets `redis://redis:6379`. Set `REDIS_URL` explicitly outside compose.

The API does not execute graphs. Without a running worker, a successfully created run remains `pending` in the queue indefinitely.

---

## 9. Idempotency (implemented behavior)

### 9.1 Manual run endpoint

`POST /api/v1/workflows/runs` supports an optional `Idempotency-Key` header. The Rust handler reads it and passes it into `CreateRunCommand.idempotency_key`.

Intelligence behavior (`WorkflowService.create_run` in `server/intelligence/src/orchestration/service.py`):

- The key is scoped to `[user_id, workspace_id, "create_run", idempotency_key]`.
- The SurrealDB record ID is `idem_<sha256(scope_key)>`; the raw key is not persisted.
- A canonical request includes workspace, user, operation, workflow ID, company, role, URLs, resume, upload, JD, seniority, prompt, workflow mode, and LLM config. Its SHA-256 is the request hash.
- `create_idempotent_workflow_run` (`server/intelligence/src/orchestration/ledger/runs.py`) performs one transaction:
  1. create `run_idempotency`;
  2. create `workflow_run`;
  3. append `workflow.created` at `seq = 1`.
- Replaying the same key and payload returns the existing run.
- Reusing the key with a different payload raises `IdempotencyConflict`, which the gRPC servicer maps to status `409` (`Idempotency-Key was already used with a different request.`).
- The raw key is neither stored nor logged.

Queue publication follows the ledger transaction. When a key is present, `RedisStreamJobQueue.enqueue_job` runs `IDEMPOTENT_ENQUEUE_LUA`, which uses `mensetsu:jobs:stream:run:<run_id>` as a guard and `SET`s the message ID. Retrying after an enqueue failure or an ambiguous enqueue response reuses the existing run and cannot add a second queue entry for that run.

### 9.2 Workspace-create path

`create_workspace` passes `Some(&ws_id)` as the idempotency key. Since `ws_id` is a new UUID generated inside the request, two separate `POST /api/v1/workspaces` calls produce different workspace IDs, different scope keys, and different runs. This key does not make workspace creation retry-safe; it only ensures the internal create-run call and its queue publication are deduplicated for that one workspace.

### 9.3 Without an idempotency key

When no key is supplied:

- `create_workflow_run` and `append_event_transactional("workflow.created")` are separate ledger operations;
- `enqueue_job` performs a plain `XADD` with no run-ID guard;
- a client retry can create a second run and a second queue message.

The web `createWorkflowRun` call (`app/web/services/workflow.service.ts`) sends only the JSON body and no `Idempotency-Key`, so browser retries rely on the user/UI not resubmitting.

### 9.4 Timeout ambiguity

`GrpcClient::send_command` waits up to 30 seconds for a terminal envelope. If the command timed out after Intelligence created the run and enqueued it, a retry with the same idempotency key returns the existing run; a retry without a key creates another run.

---

## 10. Run lifecycle and observability

### 10.1 Creation

- `WorkflowService.create_run` validates `workflow_id` against `DEFINITIONS` in `server/intelligence/src/orchestration/shared.py`. Unknown IDs raise `ValueError`; the gRPC servicer returns `400`, and REST maps it to `400 Bad Request`.
- The run is created with `status = "pending"`, `current_phase = "queued"`, `plan_revision = 1`, and `last_seq = 1` after `workflow.created`.
- The job payload contains the workspace inputs and the resolved `llm_config`.

### 10.2 Worker claim and execution

`DurableJobWorker.execute_job`:

- Claims a stream message via `XREADGROUP` or `XAUTOCLAIM`.
- Acquires `mensetsu:lease:<run_id>` with a token and TTL (default 20 s).
- Starts a heartbeat every 5 s (`WORKER_HEARTBEAT_INTERVAL_S`), renewing the lease and resetting stream idle time.
- If the run is already `cancelled`, acknowledges and exits.
- On the first attempt, sets `status = "running"`, `started_at`, `current_phase = "running"`, initial progress estimate, and emits `workflow.started`.
- On a reclaimed message or existing checkpoint, emits `execution.resumed` and `worker.recovered`.
- Builds the graph based on `workflow_id`:
  - `interview-preparation-static` → `build_static_workflow`
  - `interview-preparation-hybrid` → `build_hybrid_workflow`
  - otherwise → `build_dynamic_workflow`
- Invokes the graph with the run-scoped LangGraph `thread_id`.
- Requires the final run status to be `completed`; otherwise it raises and enters failure handling.
- Acknowledges the queue message only after successful completion or a terminal failure decision.

Transient worker failures (`is_retryable_error`) are retried up to `max_attempts` (default 3) with exponential delay, emitting `workflow.retrying`. Non-transient failures mark the run `failed`, set `error_message`, sweep tasks to terminal, emit `workflow.failed`, and update the workspace to `failed` unless it was already `ready`.

### 10.3 Events and SSE

Run event stream:

```http
GET /api/v1/workflows/runs/{run_id}/events?after_seq=<n>
Last-Event-ID: <n>
```

- Ownership is checked before the gRPC subscription.
- `after_seq` takes precedence over `Last-Event-ID`.
- The stream emits SSE frames with `id` = sequence, `event` = event type, `data` = JSON.
- `streaming_servicer.py` replays ledger history, then follows live events, and terminates on `workflow.completed`, `workflow.failed`, or `workflow.cancelled`.
- The REST SSE layer adds a 15 s keepalive.

Related read surfaces:

- `GET /api/v1/workflows/runs/{run_id}/events/history?after_seq=&limit=`
- `GET /api/v1/workflows/runs/{run_id}/graph`
- `GET /api/v1/workflows/runs/{run_id}/delegations/graph`
- `GET /api/v1/workflows/runs/{run_id}/logs`
- `GET /api/v1/workflows/runs/{run_id}/tasks/{task_id}`
- `GET /api/v1/workflows/runs/{run_id}/agents`
- `GET /api/v1/workflows/runs/{run_id}/agents/{execution_id}`
- `GET /api/v1/workflows/runs/{run_id}/agents/{execution_id}/tools`
- `GET /api/v1/workflows/runs/{run_id}/tools/{tool_invocation_id}/output`
- `GET /api/v1/workflows/runs/{run_id}/delegations`
- `GET /api/v1/workflows/runs/{run_id}/artifacts`

Workspace progress is separate:

```http
GET /api/v1/workspaces/{id}/events
```

`event_handlers.rs` polls the workspace row every 500 ms, emits `progress` only when `current_phase` or `progress_percent` changes, and closes at `ready` or `failed`. It does not carry run IDs or task events.

### 10.4 Cancellation

`POST /api/v1/workflows/runs/{run_id}/cancel` authorizes the run and calls `WorkflowService.cancel_run`:

- sets `status = "cancelled"`;
- sweeps the run to a terminal state;
- emits `workflow.cancelled`.

The worker checks `cancelled` at job start and after a graph exception, but in-flight graph nodes and agent loops are not interrupted by the status change alone. Cancellation is therefore cooperative at job/worker boundaries, not an immediate stop of running agent work.

### 10.5 Finalization

Static and dynamic runs finalize through the static `persist_outputs` / `finalize_run` path. Hybrid finalizes in `hybrid_outputs.finalize_node`.

Common outcomes:

- `workflow_run.status = "completed"`, `current_phase = "completed"`, `completed_at`, `duration_ms`.
- `progress.percentage = 100`.
- `workflow.completed` event.
- Workspace `status = "ready"`, `current_phase = "ready"`, `progress_percent = 100`.
- Optional `degraded = true` plus `degraded_reasons` when research or generation was incomplete.

`update_workspace_progress` does not downgrade a workspace that is already `ready`, including the `failed` path; the worker logs that the workspace is only downgraded if it was not already ready.

---

## 11. Current limitations and sharp edges

1. **No run ID from workspace creation.** `POST /api/v1/workspaces` returns only `WorkspaceResponseDto`; callers must list runs. There is no `GET /api/v1/workspaces/{id}/active-run` endpoint.
2. **201 does not mean the run started.** If `start_workspace_run` fails after the workspace transaction, the response is still `201` with `status = "failed"`.
3. **Workspace-create idempotency is not client-retry-safe.** The key is the newly generated workspace ID, so a repeated POST creates a second workspace and run.
4. **The web manual-run client sends no `Idempotency-Key`.** A retry from the Workflows page can create duplicate dynamic/static runs.
5. **No one-active-run policy.** `WorkflowService.create_run` does not reject a second run for a workspace with a pending/running run.
6. **Hybrid is less adaptive than its name.** `route_workflow_request`'s `selected_workflow` is advisory and unused for graph selection. The hybrid planner node creates a fixed three-task research plan and does not call the LLM planner. `workflow_mode` is also not a runtime selector; `workflow_id` selects the graph.
7. **Dynamic revision is rule-based.** `revise_node` removes duplicate optional work and adds missing agent coverage; it does not ask the planner to review findings.
8. **Unknown workspace mode silently becomes hybrid.** Any `workflow_mode` other than `static` or `dynamic` maps to `interview-preparation-hybrid` rather than returning `400`.
9. **No data-readiness gate.** `POST /api/v1/workflows/runs` accepts a workspace with no resume/JD and can fail later in normalization or job analysis. A missing resume falls back to a generic description; an empty JD is not rejected.
10. **`PATCH /api/v1/workspaces/{id}/objective` is not a run-creation path.** It updates the objective, and when a latest run exists it increments that run's `plan_revision`, emits `plan.revised`, and enqueues a job on that run. It returns no run ID, ignores `resume_plan` errors (logs a warning and still returns `200`), and the worker does not branch on the `resume_plan` payload flag. Only hybrid consumes the accompanying `plan_revision` in its state.
11. **Cancellation is not immediate for in-flight work.** A cancelled run may keep executing agent tasks until the current job boundary.
12. **Single gRPC control stream.** `GrpcClient` connects once at API startup and multiplexes commands over one `StreamControl` stream. There is no visible reconnect loop if the response stream terminates; subsequent commands can fail or wait for the 30 s command timeout.
13. **Initial task counts are estimates.** Static reports 9, hybrid reports 6, and dynamic reports 0 until the planner produces a DAG. They are progress hints, not exact ledger task counts.
14. **List-runs results depend on Intelligence.** The Rust API selects owned run IDs, then asks Intelligence for those IDs and drops any extra runs. A run missing from the Intelligence response is omitted even if Rust still has its `workflow_run` row.
15. **Hybrid question target is 105, not 100.** The code checks both a total of at least 100 and at least 15 questions in each of 7 categories; the category check dominates.

---

## 12. Source map

| Concern | Source |
|---|---|
| Workspace routes, create, trigger, LLM config resolution | `server/api/src/interfaces/http/workspaces.rs` |
| Manual run trigger, run read/stream/cancel routes | `server/api/src/interfaces/http/workflows.rs` |
| Request/response DTOs | `server/api/src/contexts/workspace/application/dtos.rs`, `server/api/src/interfaces/http/workflow_dtos.rs` |
| Workspace mode default and mapping | `server/api/src/contexts/workspace/domain/mod.rs`, `server/api/src/interfaces/http/workspaces.rs` |
| Run ownership and listing | `server/api/src/contexts/workflow/application/mod.rs`, `server/api/src/contexts/workflow/infrastructure/repository.rs` |
| Rust gRPC client and SSE bridge | `server/api/src/interfaces/grpc/client.rs` |
| gRPC contract | `contracts/grpc/proto/v1/control.proto`, `contracts/grpc/proto/v1/streaming.proto` |
| gRPC control dispatch | `server/intelligence/src/grpc/control_servicer.py` |
| gRPC authentication | `server/intelligence/src/grpc/auth_interceptor.py` |
| Run creation, idempotency, queue publication | `server/intelligence/src/orchestration/service.py` |
| Idempotent ledger transaction | `server/intelligence/src/orchestration/ledger/runs.py` |
| Redis stream, dedupe, leases | `server/intelligence/src/orchestration/redis_queue.py` |
| Worker claim, heartbeat, graph selection | `server/intelligence/src/orchestration/runtime/worker.py` |
| Static graph and tasks | `server/intelligence/src/orchestration/static_workflow.py`, `static_input.py`, `static_research.py`, `static_synthesis.py`, `static_curriculum.py`, `static_questions.py`, `static_outputs.py` |
| Hybrid graph and tasks | `server/intelligence/src/orchestration/hybrid_workflow.py`, `hybrid_nodes.py`, `hybrid_curriculum.py`, `hybrid_outputs.py`, `hybrid_support.py` |
| Dynamic graph, planner validation, revision | `server/intelligence/src/orchestration/dynamic_workflow.py` |
| Workflow definitions | `server/intelligence/src/orchestration/shared.py` |
| Model routing and provider fallback | `server/intelligence/src/agents/model_router.py` |
| Streaming assembly | `server/intelligence/src/grpc/streaming_servicer.py` |
| Workspace progress SSE | `server/api/src/interfaces/http/workspaces/event_handlers.rs` |
| Web trigger call sites | `app/web/pages/interviews/new.vue`, `app/web/components/workflow/CreateWorkflowRunDialog.vue`, `app/web/services/workflow.service.ts`, `app/web/pages/interviews/[id]/research.vue` |
