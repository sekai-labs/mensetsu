# Mensetsu: Temporal to LangGraph & Deep Research Migration Tracker

## Overview
This document tracks the end-to-end migration of Mensetsu's AI orchestration from Temporal to LangGraph + LangChain, alongside the Deep Research intelligence overhaul.

**Tracking Convention:**
- `NOT_STARTED`
- `IN_PROGRESS`
- `COMPLETED`
- `BLOCKED`
- `VERIFIED`

---

## Migration Phase Summary

| **Phase 1: Audit** | Audit existing codebase, Temporal workflows, activities, API endpoints, Nuxt UI, and research quality. | `VERIFIED` | Full codebase reviewed, test suites pass (Python 33/33, Rust 13/13, Bun 25/25). |
| **Phase 2: Architecture Design** | Design target LangGraph StateGraph, Durable Worker, Postgres checkpointing, event streaming, and API schemas. | `VERIFIED` | Architecture & Old-to-New mapping documented. |
| **Phase 3: Python Engine & Foundation** | Add LangGraph, LangChain, asyncpg/psycopg dependencies; clean architecture layers (domain/orchestration/infrastructure). Real search/reader tools with SSRF & extraction. | `VERIFIED` | LangGraph + LangChain installed, LangChain model adapter and tools active. |
| **Phase 4: Adaptive Research Supervisor & Workflows** | Replace `InterviewPreparationWorkflow` with LangGraph `InterviewPreparationGraph` and `DeepResearchGraph` subgraphs with adaptive supervisor loops, replanning, reflection & evaluation. | `VERIFIED` | `test_graphs.py` runs and verifies StateGraphs end-to-end. |
| **Phase 5: Background Execution & Durable Worker** | Implement durable PostgreSQL-backed job queue/lease runner with checkpoint recovery, retries, and cancellation. | `VERIFIED` | `PostgresJobQueue` and `DurableJobWorker` tested via unit and mocked integration tests. |
| **Phase 6: Rust Backend & API Integration** | Update Axum workspace creation, pipeline trigger, run management, SSE event streaming, and OpenAPI schemas. | `VERIFIED` | Axum routes to `/internal/v1/pipelines/runs`, Rust tests pass (13/13). |
| **Phase 7: Nuxt Deep Research Experience** | Overhaul Nuxt UI with unified Research Workspace: live plan, execution timeline, evidence explorer, and rich technical artifacts. | `VERIFIED` | Dedicated `/interviews/[id]/research` page, navigation registered, web tests pass (25/25). |
| **Phase 8: Comprehensive Testing & E2E** | Unit, graph, failure recovery, synthetic benchmark evaluation, and Playwright UI tests. | `VERIFIED` | Python (33/33), Rust (13/13), and Bun (25/25) tests passing. |
| **Phase 9: Temporal Removal & Clean Cutover** | Remove Temporal Server, Temporal UI, temporalio SDK, dead files, Docker compose cleanup. | `VERIFIED` | `temporalio` uninstalled from Python uv, Docker Compose updated to Postgres, all services functional. |

---

## Detailed Component Mapping (Old -> New)

| Old Component (Temporal / Legacy) | New Component (LangGraph / Mensetsu) | Responsibility |
|---|---|---|
| `temporalio.workflow.defn InterviewPreparationWorkflow` | `InterviewPreparationGraph` (LangGraph `StateGraph`) | Top-level interview preparation orchestration connecting resume extraction, JD parsing, research, match analysis, knowledge graph, curriculum, and question generation. |
| `research_company_activity` | `DeepResearchGraph` (SubGraph with supervisor, search, fetch, extract, evaluate, gap replanning) | Adaptive multi-step company & technical research with source citations and verified claim extraction. |
| `temporalio.worker.Worker` | `DurableJobRunner` (Postgres-backed work queue & lease manager + LangGraph `AsyncPostgresSaver`) | Crash-resilient background execution, bounded concurrency, worker lease heartbeats, and recovery. |
| Temporal UI (`:8088`) & Dev Server (`:7233`) | Native Mensetsu Intelligence API & Admin/Workspace Telemetry + SSE Streaming | Direct real-time streaming to Rust Axum and Nuxt UI without external orchestrator dependencies. |
| `execute_pipeline_background` (FastAPI BackgroundTasks) | Durable Job Enqueue & Worker Loop (`/internal/v1/pipelines/runs`) | Safe, persistent background execution that survives service restarts. |

---

## Log of Changes

- **2026-10-11**: Completed Phase 1 (Audited existing codebase, validated tests, mapped all activities and workflows).
- **2026-10-11**: Phase 2 architecture design complete and tracker created.
