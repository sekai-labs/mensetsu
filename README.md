# Mensetsu (面接)

AI-driven, evidence-backed technical interview preparation platform. Mensetsu pairs candidate resumes with job descriptions to perform verifiable web research, gap analysis, knowledge graph construction, and personalized curriculum assembly.

---

## Architecture

- **Web Client (`app/web`)**: Nuxt 4, Vue 3, Tailwind CSS, Vue Flow. Uses a Backend-For-Frontend (BFF) proxy with Redis-mapped session tokens and RFC 6238 TOTP 2FA.
- **Core API (`server/api`)**: Rust (Axum, Tokio, SurrealDB, Utoipa OpenAPI 3.1).
- **Intelligence Engine (`server/intelligence`)**: Python 3.12 (`uv`), Temporal workflows, SSRF-guarded research scraper, semantic LLM routing.
- **Mobile & Desktop Client (`app/mobile`)**: Flutter 3+ adaptive client.
- **Data & Cache**: SurrealDB (relational, vector, graph) and Redis 7 (BFF session mapping).

---

## Quickstart

### Prerequisites
- Docker & Docker Compose
- Bun (`curl -fsSL https://bun.sh/install | bash`)
- Rust (`rustup default stable`)
- Python 3.12+ (`uv`)

### Run with Docker Compose
```bash
# Start SurrealDB, Redis, Temporal, and core services
docker compose up -d

# Services will be available at:
# - Web Application:  http://localhost:3001
# - Temporal Web UI:  http://localhost:8088
# - Redis Server:     localhost:6379
```

### Local Development

1. **Backend (Rust)**:
   ```bash
   cd server/api
   cargo run --bin mensetsu-api
   ```

2. **Web Client (Nuxt + Bun)**:
   ```bash
   cd app/web
   bun install
   bun run dev
   ```

3. **Intelligence Engine (Python)**:
   ```bash
   cd server/intelligence
   uv sync
   uv run uvicorn src.service:app --port 8001
   ```

---

## Testing

```bash
# Web tests (Vitest via Bun)
cd app/web && bun run test

# Rust backend tests
cd server/api && cargo test

# Python intelligence tests
cd server/intelligence && uv run pytest
```

---

## Contributor Rules & Guidelines

All contributors and coding agents must follow these guidelines:

### 1. Package Management & Tooling
- **Web**: Always use **Bun** (`bun install`, `bun add`, `bun run test`). Do not use `npm` or commit `package-lock.json`.
- **Rust**: Use standard `cargo` toolchain.
- **Python**: Use `uv` for dependency management.

### 2. TypeScript & Code Standards
- **No `any`**: Strictly forbid `: any` or `as any`. Use typed interfaces, generics, or type guards.
- **Modular Feature Architecture**: Group web features in `app/web/features/<feature>/` with:
  - `pages/`, `services/`, `utils/`, `models/`, `components/`
  - Public shared UI components belong in `app/web/components/ui/`.
  - Public shared utilities belong in `app/web/utils/`.
- **No Trivial Wrappers**: Avoid one-line wrapper functions that only rename existing functions without domain behavior.

### 3. Security Invariants
- **BFF Token Isolation**: Client JavaScript must never receive or store raw JWT access or refresh tokens. All client traffic flows through the server proxy using HttpOnly session ID cookies mapped in Redis.
- **SSRF Guardrails**: All external URLs fetched by scrapers or intelligence activities must pass IP validation (block loopback, link-local, RFC1918, and cloud metadata IPs).
- **Prompt Injection Defense**: User-supplied resumes and job descriptions must be contained and treated as untrusted data.

### 4. Git & Commits
- Use conventional commit messages (`feat(...)`, `fix(...)`, `chore(...)`, `test(...)`).
- Keep commits atomic and scoped by domain (`web`, `api`, `intelligence`, `infra`, `mobile`).
- Do not commit sensitive keys, `.env` files with real credentials, temporary test output, or debug artifacts.

---

## License

Apache-2.0. See [LICENSE](LICENSE) for details.
