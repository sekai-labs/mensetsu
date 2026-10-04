# Mensetsu (面接)

An AI-powered, evidence-backed interview preparation platform. Mensetsu replaces generic chatbot interactions with a structured, durable, research-backed workspace tailored to a candidate's actual resume, target role, and target company.

---

## 1. System Overview & Architecture

```
                    +------------------------------------+
                    |              Clients               |
                    |  - Nuxt 3 Web (Vue Flow, SSR/SPA)  |
                    |  - Flutter Adaptive Mobile/Desktop |
                    +-----------------+------------------+
                                      |
                              HTTPS / SSE (OpenAPI 3.1)
                                      |
                    +-----------------v------------------+
                    |        server/api (Rust)           |
                    |  - Axum & Tokio Runtime            |
                    |  - Domain-Driven Design (DDD)      |
                    |  - Native SurrealDB v3 Driver      |
                    +-----------------+------------------+
                                      |
                               HTTP Trigger
                                      |
                    +-----------------v------------------+
                    |   server/intelligence (Python)     |
                    |  - uv Workspace & Pydantic AI      |
                    |  - Temporal Durable Workflows      |
                    |  - SSRF-Safe Research Fetcher      |
                    |  - Prompt Injection Defense Layer  |
                    +-----------------+------------------+
                                      |
             +------------------------+------------------------+
             |                                                 |
+------------v------------+                       +------------v------------+
|  SurrealDB v3 (Docker)  |                       |     Temporal Server     |
| - Relational & Graph    |                       | - Durable Workflow Exec |
| - Vectors & Schemas     |                       | - Event History Store   |
+-------------------------+                       +-------------------------+
```

---

## 2. Key Features

- **Asynchronous Durable Workflow**: Uploading a resume and role description immediately returns a workspace ID while research executes in the background across resilient activities.
- **Server-Sent Events (SSE)**: Live streaming pipeline progress updates from `preparing_input` to `ready`.
- **Explainable Match & Gap Analysis**: Transparent decomposition of mandatory vs preferred requirements, candidate strengths, evidence quotes, and targeted gap remediation.
- **SSRF & Injection Defense**: Automated DNS hostname resolution with strict denylist checking (RFC1918, link-local, cloud metadata IP blocking) and prompt containment wrappers.
- **Interactive Knowledge Graph**: Rendered with `@vue-flow/core` showing concept dependencies, required technologies, and candidate mastery levels.
- **Curated Learning Units**: Prerequisite-ordered technical syllabus with structured architecture and flow diagrams.
- **Targeted Question Book**: Questions mapped to real company pillars with answer outlines, senior evaluation rubrics, and common candidate pitfalls.

---

## 3. Monorepo Layout

```
mensetsu/
├── app/
│   ├── web/               # Nuxt 3 TypeScript web client
│   └── mobile/            # Flutter 3+ adaptive mobile & desktop client
├── server/
│   ├── api/               # Rust Axum backend with SurrealDB & OpenAPI
│   │   └── contracts/     # Generated OpenAPI spec and type definitions
│   └── intelligence/      # Python uv research engine with Temporal
├── infra/
│   ├── docker-compose.yml # SurrealDB & Temporal compose definitions
│   ├── migrations/        # SurrealQL schema definitions
│   └── verify_e2e.sh      # End-to-end integration verification script
└── fixtures/              # Synthetic test profiles and job descriptions

---

## 4. Local Development Quickstart

### Prerequisites
- Docker & Docker Compose
- Rust 1.80+ (`cargo`)
- Python 3.12+ (`uv`)
- Node.js 20+ (`npm`)
- Flutter 3.20+ (for mobile/desktop client)

### Setup & Run

1. **Start Local Dependencies (SurrealDB)**:
   ```bash
   make infra-up
   ./infra/verify_and_migrate.sh
   ```

2. **Run Rust Core API Service**:
   ```bash
   cd server/api
   cargo run --bin mensetsu-api
   # API listening at http://127.0.0.1:3000
   # Swagger UI at http://127.0.0.1:3000/swagger-ui
   ```

3. **Run Python Intelligence Engine**:
   ```bash
   cd server/intelligence
   uv run uvicorn src.service:app --port 8001 --host 127.0.0.1
   ```

4. **Run Nuxt Web Client**:
   ```bash
   cd app/web
   bun run dev
   # Web workspace accessible at http://localhost:3000 (or http://localhost:3001)
   ```

5. **Run Flutter Mobile & Desktop Client**:
   ```bash
   cd app/mobile
   flutter run -d linux # or macos / windows / chrome
   ```

---

## 5. Testing & Verification

Run all test suites across the stack:

```bash
# Rust Unit and Integration Tests
cd server/api && cargo test

# Python Unit, Activity, SSRF & Eval Tests
cd server/intelligence && uv run pytest

# Web Client Vitest Tests
cd app/web && bun x vitest run

# Flutter Model & Unit Tests
cd app/mobile && flutter test

# End-to-End Automated Pipeline Verification
./infra/verify_e2e.sh
```

---

## 6. License
MIT
