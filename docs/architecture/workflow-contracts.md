# Workflow Contracts Specification (Public v1 Contract)

This document specifies the authoritative Public v1 contract exposed by the Python Intelligence Service (`server/intelligence`), proxied by the Rust Core API (`server/api`), and consumed by the Nuxt Web Client (`app/web`).

---

## 1. Architectural Boundaries & Authentication

### 1.1 Internal Service Auth & Network Isolation
1. **Network Boundary**:
   - The intelligence service runs as an internal microservice on the private Docker network. Port 8001 is **not published** to host interfaces in `docker-compose.yml`.
   - Only the Rust API gateway (`server/api`) and durable background worker communicate with the intelligence HTTP server.
2. **Bearer Token Authentication**:
   - Every route on the intelligence service (with the explicit exception of `/health/live` and unauthenticated status on `/health/ready`) resides under `/internal/v1/*` and requires:
     ```http
     Authorization: Bearer <INTERNAL_SERVICE_TOKEN>
     ```
   - Tokens are validated in **constant time** using `hmac.compare_digest`.
   - **Fail-Closed Startup Invariant**: If `INTERNAL_SERVICE_TOKEN` is unset or empty in the process environment, the service refuses to start and raises an unrecoverable exception.
3. **Health Probes**:
   - `/health/live`: Unauthenticated liveness probe. Returns `200 {"status": "alive"}`.
   - `/health/ready`: Without authorization, returns only basic status strings (`200 {"status": "ready"}` or `503 {"status": "unhealthy"}`) without exposing database DSNs, pool statistics, or internal stack traces. Detailed system diagnostics require the Bearer token.
4. **Rust Gateway Authority**:
   - Rust Axum endpoints at `/api/v1/workflows/*` mirror the internal `/internal/v1/workflows/*` endpoints 1:1.
   - Rust verifies session authenticity, enforces workspace access control via SurrealDB, and injects `workspace_id` and `user_id` into intelligence requests.
   - Intelligence trusts Rust for scoping and audit, but never relies on user-supplied parameters for tenant authorization.

---

## 2. Core Domain Schemas

The canonical source of truth for these schemas is defined as Pydantic v2 models in `src/contracts/v1.py` and exported to `server/api/contracts/schema/workflows.v1.json`.

### 2.1 `WorkflowDefinition`
Represents the structural specification of an available workflow type.

```json
{
  "id": "interview-preparation-static",
  "name": "Static Interview Preparation Workflow",
  "type": "static",
  "version": "1.0.0",
  "description": "Deterministic fan-out/fan-in research and question generation pipeline",
  "default_max_retries": 3,
  "supported_agents": [
    "company-research",
    "job-analysis",
    "technical-research",
    "interview-generation",
    "evidence-evaluator"
  ]
}
```

- `id` (string, required): Unique workflow template identifier.
- `name` (string, required): Human-readable name.
- `type` (string enum, required): `"static"` | `"dynamic"`.
- `version` (string, required): Semantic version string (e.g. `"1.0.0"`).
- `description` (string, required): Detailed workflow behavior description.
- `default_max_retries` (integer, required): Default retry count for constituent tasks.
- `supported_agents` (array of strings, required): Agent roles supported by this definition.

---

### 2.2 `WorkflowRun` (Snapshot)
Snapshot of a workflow execution instance.

```json
{
  "id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "workflow_id": "interview-preparation-static",
  "workflow_name": "Static Interview Preparation Workflow",
  "workspace_id": "workspace_123",
  "user_id": "usr_456",
  "type": "static",
  "status": "running",
  "current_phase": "research",
  "plan_revision": 1,
  "progress": {
    "total_tasks": 7,
    "pending": 0,
    "queued": 0,
    "running": 3,
    "completed": 2,
    "failed": 0,
    "retrying": 0,
    "blocked": 0,
    "cancelled": 0,
    "skipped": 0,
    "percentage": 28,
    "estimate": false,
    "phase_index": 1,
    "phase_count": 4
  },
  "last_seq": 42,
  "started_at": "2026-10-12T10:00:00Z",
  "completed_at": null,
  "duration_ms": null,
  "created_at": "2026-10-12T10:00:00Z",
  "updated_at": "2026-10-12T10:01:15Z",
  "error_message": null
}
```

- `id` (string UUID, required): Globally unique run identifier.
- `workflow_id` (string, required): Identifier of the workflow template.
- `workflow_name` (string, required): Human-readable name of the workflow.
- `workspace_id` (string, required): Associated workspace scope (from Rust authority).
- `user_id` (string, required): Owner user ID (from Rust authority).
- `type` (string enum, required): `"static"` | `"dynamic"`.
- `status` (string enum, required): `"pending"` | `"running"` | `"completed"` | `"failed"` | `"cancelled"`.
- `current_phase` (string, required): Human-readable execution stage.
- `plan_revision` (integer, required): Monotonic count of dynamic plan adaptations (starts at 1).
- `progress` (object, required):
  - `total_tasks` (integer): Total active tasks in current plan revision.
  - `pending`, `queued`, `running`, `completed`, `failed`, `retrying`, `blocked`, `cancelled`, `skipped` (integers): State tallies.
  - `percentage` (integer 0–100 or null): Null when not meaningful (e.g. dynamic runs whose plan is still expanding); numeric percentage for static non-expanding plans.
  - `estimate` (boolean): Flag indicating whether progress percentage is an approximation.
  - `phase_index` (integer or null): 0-based index of the current phase in static workflows.
  - `phase_count` (integer or null): Total phase count in static workflows.
- `last_seq` (integer, required): Highest monotonic sequence number emitted to date.
- `started_at` (string ISO-8601 or null).
- `completed_at` (string ISO-8601 or null).
- `duration_ms` (number or null).
- `created_at` (string ISO-8601, required).
- `updated_at` (string ISO-8601, required).
- `error_message` (string or null).

---

### 2.3 `WorkflowTask`
Represents an individual discrete step in the workflow graph.

```json
{
  "id": "task_research_company",
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "task_type": "research",
  "name": "Research Target Company Culture & Tech",
  "assigned_agent": "company-research",
  "dependencies": ["task_normalize_input"],
  "status": "running",
  "attempt": 1,
  "required": true,
  "added_in_revision": 1,
  "execution_ids": ["exec_8f293b71c0de"],
  "input_data": {
    "company_name": "Acme Corp",
    "company_urls": ["https://acme.example.com"]
  },
  "output_data": {},
  "error": null,
  "created_at": "2026-10-12T10:00:05Z",
  "updated_at": "2026-10-12T10:00:10Z"
}
```

- `status` (string enum, required): `"pending"` | `"queued"` | `"running"` | `"completed"` | `"failed"` | `"retrying"` | `"blocked"` | `"cancelled"` | `"skipped"`.
- `execution_ids` (array of strings, required): Associated agent executions.

---

### 2.4 `AgentExecution`
Represents an invocation of an autonomous agent executing a task or delegated subtask.

```json
{
  "id": "exec_8f293b71c0de",
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "task_id": "task_research_company",
  "parent_execution_id": null,
  "child_execution_ids": ["exec_998877665544"],
  "agent_id": "company-research",
  "objective": "Scrape public docs and verify company culture and tech stack",
  "status": "completed",
  "delegation_depth": 0,
  "retries": 0,
  "current_activity": null,
  "model_provider": "anthropic",
  "model_name": "claude-3-7-sonnet-20250219",
  "budgets": {
    "model_calls": 10,
    "tool_calls": 20,
    "tokens_in": 50000,
    "tokens_out": 10000,
    "wall_time_ms": 120000.0,
    "external_requests": 25
  },
  "budgets_used": {
    "model_calls": 3,
    "tool_calls": 4,
    "tokens_in": 3200,
    "tokens_out": 1050,
    "wall_time_ms": 12840.5,
    "external_requests": 4
  },
  "tool_call_count": 4,
  "notes": [
    {
      "ts": "2026-10-12T10:00:15Z",
      "text": "Identified Kubernetes and Rust architectural patterns from engineering blog."
    }
  ],
  "artifact_ids": ["art_aabbccddeeff"],
  "public_summary": "Extracted 14 verified claims regarding Acme engineering architecture and core product values.",
  "error": null,
  "started_at": "2026-10-12T10:00:10Z",
  "completed_at": "2026-10-12T10:00:23Z"
}
```

- `notes`: Strictly public decision summaries; never includes chain-of-thought scratchpads.

---

### 2.5 `ToolInvocation`
Audit record of a single tool invocation made by an agent.

```json
{
  "id": "tool_0a1b2c3d4e5f",
  "execution_id": "exec_8f293b71c0de",
  "tool_name": "read_source",
  "status": "completed",
  "duration_ms": 412.3,
  "sanitized_input": {
    "url": "https://acme.example.com/about"
  },
  "output_summary": "Retrieved 3500 bytes of clean text; HTTP 200.",
  "output_ref": "surreal:source:src_8f293b71c0de",
  "error": null,
  "started_at": "2026-10-12T10:00:12Z",
  "completed_at": "2026-10-12T10:00:13Z"
}
```

---

### 2.6 `Delegation`
Record of hierarchical sub-agent delegation.

```json
{
  "id": "del_112233445566",
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "parent_execution_id": "exec_8f293b71c0de",
  "child_execution_id": "exec_998877665544",
  "parent_agent_id": "company-research",
  "child_agent_id": "technical-research",
  "delegation_task_id": "task_sub_verify_blog",
  "objective": "Scrape and evaluate engineering blog posts for architecture signals",
  "reason": "Delegate deep engineering blog scraping to technical-research specialist",
  "depth": 1,
  "status": "completed",
  "input_artifact_ids": [],
  "output_artifact_ids": ["art_123456"],
  "error": null,
  "created_at": "2026-10-12T10:00:15Z",
  "completed_at": "2026-10-12T10:00:20Z"
}
```

- `status` (string enum, required): `"requested"` | `"accepted"` | `"running"` | `"completed"` | `"failed"` | `"rejected"`.

---

### 2.7 `Artifact` & Evidence Provenance
First-class output produced by tasks or agents.

```json
{
  "id": "art_aabbccddeeff",
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "task_id": "task_research_company",
  "agent_execution_id": "exec_8f293b71c0de",
  "type": "evidence",
  "title": "Official About Page Architecture Claims",
  "version": 1,
  "surreal_record_id": "source:src_workspace_123",
  "evidence_ids": ["claim_987654"],
  "source_links": [
    { "title": "Acme Engineering Blog", "url": "https://acme.example.com/blog" }
  ],
  "evidence": {
    "source_url": "https://acme.example.com/blog/architecture",
    "title": "Scalable Systems at Acme",
    "retrieved_at": "2026-10-12T10:00:13Z",
    "snippet": "We migrated our core data pipeline to Kafka and SurrealDB for real-time telemetry.",
    "confidence": 0.95
  },
  "content": {
    "summary": "High-velocity distributed systems engineering team...",
    "key_values": ["scalability", "transparency"]
  },
  "created_at": "2026-10-12T10:00:22Z"
}
```

- `type` (string enum, required): `"research_report"` | `"finding"` | `"evidence"` | `"question_set"` | `"recommendation"` | `"validation_result"`.
- When `type == "evidence"`, the `evidence` object is mandatory and adheres strictly to the AGENTS.md provenance rule (`source_url`, `title`, `retrieved_at`, `snippet`, `confidence`).

---

## 3. Discriminated Event Envelope & Dotted Taxonomy

### 3.1 Envelope Schema
All events share the root envelope:

```json
{
  "schema_version": "1.0",
  "event_id": "evt_9876543210ab",
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "seq": 14,
  "ts": "2026-10-12T10:00:15.123Z",
  "type": "task.started",
  "task_id": "task_research_company",
  "execution_id": "exec_8f293b71c0de",
  "parent_execution_id": null,
  "tool_invocation_id": null,
  "correlation_id": "corr_cba987654321",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "payload": {
    "name": "Research Target Company Culture & Tech",
    "assigned_agent": "company-research",
    "attempt": 1
  }
}
```

The `payload` field is a discriminated union keyed on the dotted `type` field.

### 3.2 Exhaustive Dotted Types & Discriminated Payloads

1. `workflow.created`: `{"workflow_id": string, "workspace_id": string, "user_id": string, "type": "static"|"dynamic"}`
2. `workflow.started`: `{"plan_revision": integer, "initial_tasks_count": integer}`
3. `workflow.completed`: `{"duration_ms": number, "total_tasks": integer, "artifacts_count": integer}`
4. `workflow.failed`: `{"error": string, "failed_task_id": string|null}`
5. `workflow.cancelled`: `{"reason": string, "cancelled_by": string}`
6. `plan.created`: `{"revision": integer, "tasks": [{"id": string, "name": string, "assigned_agent": string, "dependencies": [string]}]}`
7. `plan.revised`: `{"revision": integer, "added_tasks": [string], "removed_tasks": [string], "reason": string}`
8. `task.added`: `{"task_id": string, "name": string, "assigned_agent": string, "dependencies": [string]}`
9. `task.removed`: `{"task_id": string, "reason": string}`
10. `task.queued`: `{"task_id": string, "priority": integer}`
11. `task.started`: `{"task_id": string, "attempt": integer, "assigned_agent": string}`
12. `task.completed`: `{"task_id": string, "duration_ms": number, "output_summary": string}`
13. `task.failed`: `{"task_id": string, "attempt": integer, "error": string, "can_retry": boolean}`
14. `task.retrying`: `{"task_id": string, "next_attempt": integer, "delay_ms": integer}`
15. `task.blocked`: `{"task_id": string, "blocked_by_task_id": string}`
16. `agent.started`: `{"execution_id": string, "agent_id": string, "objective": string, "model": string}`
17. `agent.activity`: `{"execution_id": string, "activity_type": string, "description": string}`
18. `agent.completed`: `{"execution_id": string, "summary": string, "tool_calls": integer, "tokens_used": integer}`
19. `agent.failed`: `{"execution_id": string, "error": string}`
20. `delegation.requested`: `{"parent_execution_id": string, "target_agent": string, "reason": string}`
21. `delegation.accepted`: `{"delegation_id": string, "child_execution_id": string}`
22. `delegation.started`: `{"delegation_id": string, "child_execution_id": string}`
23. `delegation.completed`: `{"delegation_id": string, "child_execution_id": string, "summary": string}`
24. `delegation.failed`: `{"delegation_id": string, "error": string}`
25. `tool.started`: `{"tool_invocation_id": string, "tool_name": string, "sanitized_input": object}`
26. `tool.completed`: `{"tool_invocation_id": string, "tool_name": string, "duration_ms": number, "summary": string}`
27. `tool.failed`: `{"tool_invocation_id": string, "tool_name": string, "error": string}`
28. `artifact.created`: `{"artifact_id": string, "type": string, "title": string, "surreal_record_id": string}`
29. `finding.saved`: `{"finding_id": string, "category": string, "claim": string, "confidence": number}`
30. `evidence.saved`: `{"source_id": string, "url": string, "title": string, "chunk_count": integer}`
31. `checkpoint.created`: `{"thread_id": string, "checkpoint_id": string, "step": integer}`
32. `execution.resumed`: `{"thread_id": string, "from_step": integer}`
33. `worker.recovered`: `{"worker_id": string, "run_id": string, "resumed_tasks_count": integer}`

---

## 4. Log Entry & Page Envelopes

### 4.1 `LogEntry`

```json
{
  "id": 1042,
  "run_id": "run_0192e2b3-7649-7cb1-9876-0123456789ab",
  "seq": 14,
  "task_id": "task_research_company",
  "execution_id": "exec_8f293b71c0de",
  "tool_invocation_id": "tool_0a1b2c3d4e5f",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "component": "agent_runtime",
  "severity": "info",
  "message": "Fetched official company about page with status 200",
  "duration_ms": 412.3,
  "error_category": null,
  "attributes": {
    "url": "https://acme.example.com/about",
    "status_code": 200,
    "bytes": 3500
  },
  "ts": "2026-10-12T10:00:13.450Z"
}
```

- `severity` (string enum): `"debug"` | `"info"` | `"warn"` | `"error"`.
- `error_category` (string enum or null): `"network"` | `"rate_limit"` | `"budget_exceeded"` | `"llm_error"` | `"validation_error"` | `"internal_error"`.

### 4.2 Page Envelopes
- **Runs Page**: `GET /internal/v1/workflows/runs?workspace_id=&status=&type=&cursor=&limit=`
  ```json
  {
    "items": [ /* WorkflowRun */ ],
    "next_cursor": "eyJpZCI6ICJydW5fMTIzIn0="
  }
  ```
- **Logs Page**: `GET /internal/v1/workflows/runs/{run_id}/logs?cursor=&limit=&severity=`
  ```json
  {
    "items": [ /* LogEntry */ ],
    "next_cursor": "1050"
  }
  ```
- **Events History Page**: `GET /internal/v1/workflows/runs/{run_id}/events/history?after_seq=0&limit=50`
  ```json
  {
    "events": [ /* ExecutionEvent */ ],
    "next_after_seq": 50,
    "last_seq": 142
  }
  ```
- **Tool Invocations Page**: `GET /internal/v1/workflows/runs/{run_id}/agents/{execution_id}/tools?cursor=&limit=`
  ```json
  {
    "items": [ /* ToolInvocation */ ],
    "next_cursor": "tool_0a1b2c"
  }
  ```
- **Controlled Tool Output Retrieval**: `GET /internal/v1/workflows/runs/{run_id}/tools/{tool_invocation_id}/output?offset=0&limit=1000`
  ```json
  {
    "tool_invocation_id": "tool_0a1b2c3d4e5f",
    "offset": 0,
    "limit": 1000,
    "total_bytes": 3500,
    "has_more": true,
    "content": "..."
  }
  ```

---

## 5. Visual Graph Endpoints

### 5.1 Task Graph: `GET /internal/v1/workflows/runs/{run_id}/graph`
Represents the task dependency DAG for rendering in Vue Flow.

```json
{
  "nodes": [
    {
      "id": "task_research_company",
      "label": "Research Target Company",
      "type": "task",
      "data": {
        "status": "running",
        "assigned_agent": "company-research",
        "attempt": 1,
        "task_type": "research",
        "required": true,
        "added_in_revision": 1,
        "execution_ids": ["exec_8f293b71c0de"],
        "group": "research"
      }
    }
  ],
  "edges": [
    {
      "id": "edge_norm_to_comp",
      "source": "task_normalize_input",
      "target": "task_research_company",
      "kind": "dependency"
    }
  ]
}
```

- All task graph edges MUST have `kind: "dependency"`.
- `group` represents the phase name used for visual collapsing in the frontend.

### 5.2 Delegation Graph: `GET /internal/v1/workflows/runs/{run_id}/delegations/graph`
Constructed **strictly** from `Delegation` rows to display hierarchical sub-agent relationships:

```json
{
  "nodes": [
    {
      "id": "exec_8f293b71c0de",
      "label": "company-research",
      "type": "agent",
      "data": {
        "status": "running",
        "task_id": "task_research_company",
        "agent_id": "company-research"
      }
    },
    {
      "id": "exec_998877665544",
      "label": "technical-research",
      "type": "agent",
      "data": {
        "status": "completed",
        "task_id": "task_sub_verify_blog",
        "agent_id": "technical-research"
      }
    }
  ],
  "edges": [
    {
      "id": "del_edge_112233445566",
      "source": "exec_8f293b71c0de",
      "target": "exec_998877665544",
      "kind": "delegation",
      "data": {
        "delegation_id": "del_112233445566",
        "objective": "Scrape deep engineering blog posts",
        "status": "completed",
        "created_at": "2026-10-12T10:00:15Z",
        "parent_agent_id": "company-research",
        "child_agent_id": "technical-research"
      }
    }
  ]
}
```

---

## 6. Monotonic Event Stream & SSE Framing

- **`GET /internal/v1/workflows/runs/{run_id}/events`**
  - **Sequence Replay Precedence**:
    - If `after_seq` query parameter is provided, it takes precedence.
    - If `after_seq` is omitted, the standard HTTP `Last-Event-ID` header is used.
    - If both are omitted, `after_seq` defaults to `0`.
  - **Framing Invariants**:
    - Standard W3C Server-Sent Events framing:
      ```
      id: <seq>
      event: <dotted_event_type>
      data: <JSON ExecutionEvent envelope>

      ```
    - Keepalive comments (`: ping\n\n`) emitted every 15 seconds.
    - Terminal event (`workflow.completed`, `workflow.failed`, `workflow.cancelled`): emit terminal event, flush, and close connection.
