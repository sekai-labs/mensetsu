.PHONY: help infra-up infra-down up down build dev-api dev-ai dev-worker dev-web test-api test-ai test-web build-web test check contracts contracts-check schema-parity api-target-owner

CONTRACTS_DIR := server/api/contracts

help:
	@echo "Mensetsu Commands:"
	@echo "  make up              - Start all services (surrealdb, redis, api, intelligence, worker, web) via Docker Compose"
	@echo "  make down            - Stop all containers"
	@echo "  make infra-up        - Start only infrastructure (SurrealDB, Redis) in background"
	@echo "  make infra-down      - Stop infrastructure containers"
	@echo "  make dev-api         - Run Rust API service locally"
	@echo "  make dev-ai          - Run Python intelligence HTTP API locally"
	@echo "  make dev-worker      - Run Python intelligence durable worker locally"
	@echo "  make dev-web         - Run Nuxt web client locally"
	@echo "  make contracts       - Export Rust OpenAPI and regenerate TypeScript contracts"
	@echo "  make contracts-check - Fail if committed contracts differ from a fresh regeneration"
	@echo "  make schema-parity   - Fail if schema.surql and infra/migrations produce different schemas (needs SurrealDB)"
	@echo "  make test            - Run all test suites"
	@echo "  make check           - Typecheck, lint, and verify contracts and schema parity"

api-target-owner:
	@bad=$$(find server/api/target -maxdepth 4 ! -user "$$(id -u)" -print -quit 2>/dev/null); \
	if [ -n "$$bad" ]; then \
		echo "Error: server/api/target contains files not owned by $$(id -un) (e.g. $$bad)." >&2; \
		echo "A root container bind-mounted the tree (docker run -v ./server/api:/app ... cargo). Build images with 'docker compose build api' instead." >&2; \
		echo "Fix: sudo chown -R $$(id -u):$$(id -g) server/api/target" >&2; \
		exit 1; \
	fi

up:
	docker compose up -d

down:
	docker compose down

infra-up:
	docker compose up -d surrealdb redis

infra-down:
	docker compose stop surrealdb redis

build:
	docker compose build

dev-api: api-target-owner
	cd server/api && cargo run

dev-ai:
	cd server/intelligence && uv run uvicorn src.service:app --host 0.0.0.0 --port 8001

dev-worker:
	cd server/intelligence && uv run python -m src.worker

dev-web:
	cd app/web && bun run dev

build-web:
	cd app/web && bun run build

test-api: api-target-owner
	cd server/api && cargo test

test-ai:
	cd server/intelligence && uv run pytest

test-web:
	cd app/web && bun x vitest run

test: test-api test-ai test-web

contracts: api-target-owner
	cd server/api && cargo run --quiet --bin export-openapi
	cd $(CONTRACTS_DIR) && bun install --frozen-lockfile && bun run generate

contracts-check:
	@set -e; tmp=$$(mktemp -d); trap 'rm -rf "$$tmp"' EXIT; \
	cp -R $(CONTRACTS_DIR)/openapi.json $(CONTRACTS_DIR)/src "$$tmp/"; \
	$(MAKE) --no-print-directory contracts; \
	diff -u "$$tmp/openapi.json" $(CONTRACTS_DIR)/openapi.json; \
	diff -ru "$$tmp/src" $(CONTRACTS_DIR)/src; \
	echo "contracts up to date"

schema-parity:
	./scripts/check-schema-parity.py $${SURREAL_HTTP_URL:-http://127.0.0.1:8000}

e2e-reset:
	@if [ "$${MENSETSU_ALLOW_RESET:-0}" != "1" ]; then \
		echo "Error: Reset refused. You must explicitly export MENSETSU_ALLOW_RESET=1" >&2; \
		echo "Usage: MENSETSU_ALLOW_RESET=1 make e2e" >&2; \
		exit 1; \
	fi
	./scripts/e2e-reset.sh

e2e: e2e-reset
	bunx playwright test

check: api-target-owner contracts-check schema-parity
	cd server/api && cargo check
	cd server/intelligence && uv run ruff check .
