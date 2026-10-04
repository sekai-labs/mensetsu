# Mensetsu Intelligence Service

Asynchronous research and extraction background engine for the Mensetsu platform.

## Architecture & Responsibilities

- **Temporal Durable Workflows**: Orchestrates the multi-stage research and interview preparation pipeline (`InterviewPreparationWorkflow`).
- **Document Parsing**: Extracts structured skills and competencies from PDF, DOCX, and raw text resumes with prompt injection containment wrappers.
- **SSRF-Safe Web Research**: Enforces strict DNS resolution and CIDR filtering (blocks RFC1918, link-local, and cloud metadata IPs) before fetching external URLs.
- **SurrealDB Knowledge Graph**: Assembles and persists company domains, technologies, skills, match analysis, and prerequisite-ordered learning paths into SurrealDB.
- **Deterministic Fallback & LLM Routing**: Supports dynamic multi-model LLM generation (Anthropic, OpenAI) with deterministic heuristic fallbacks for offline testing and development.

## Local Development

### Prerequisites
- Python 3.12+
- `uv` package manager
- SurrealDB running on port 8000

### Run Service
```bash
# Run HTTP trigger service
uv run uvicorn src.service:app --port 8001 --host 127.0.0.1

# Run Temporal background worker
uv run python -m src.worker
```

### Testing & Linting
```bash
# Run test suite
uv run pytest

# Run linter
uv run ruff check .
```
