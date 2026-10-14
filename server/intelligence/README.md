# Mensetsu Intelligence Service

Asynchronous research and extraction background engine for the Mensetsu platform.

## Architecture & Responsibilities

- **Autonomous Multi-Agent Workflows**: Orchestrates dual-mode static and dynamic LangGraph agent workflows with durable SurrealDB ledger and Redis checkpointer.
- **Document Parsing**: Extracts structured skills and competencies from PDF, DOCX, and raw text resumes with prompt injection containment wrappers.
- **SSRF-Safe Web Research**: Enforces strict DNS resolution and CIDR filtering (blocks RFC1918, link-local, and cloud metadata IPs) before fetching external URLs.
- **SurrealDB Knowledge Graph**: Assembles and persists company domains, technologies, skills, match analysis, and prerequisite-ordered learning paths into SurrealDB.
- **LLM Access**: Agents call an OpenAI-compatible endpoint (`LLM_OPENAI_BASE_URL`, `LLM_OPENAI_API_KEY`, `LLM_DEFAULT_MODEL`) through LangChain, with bounded retries for timeouts, 429 and 5xx/524 gateway errors.
- **Concurrency Limits** (`src/orchestration/limits.py`): each worker process caps agent executions globally (`MAX_CONCURRENT_AGENTS`, default 8), per run (`MAX_CONCURRENT_AGENTS_PER_RUN`, default 4) and per agent (`max_concurrency` in `agents/<id>/agent.yaml`, default 3), and caps in-flight LLM calls per provider endpoint (`LLM_MAX_INFLIGHT`, default 6). Delegated child executions take only their agent slot, so a parent never deadlocks waiting on its own child. Waits longer than 1 s emit an `agent.activity` "Waiting for execution slot (...)" event.

## Local Development

### Prerequisites
- Python 3.12+
- `uv` package manager
- SurrealDB running on port 8000

### Run Service
```bash
# Run HTTP trigger service
uv run uvicorn src.service:app --port 8001 --host 127.0.0.1
# Run Python background worker
uv run python -m src.worker
```

### Testing & Linting
```bash
# Run test suite
uv run pytest

# Run linter
uv run ruff check .
```
