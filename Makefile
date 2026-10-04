.PHONY: help infra-up infra-down up down build dev-api dev-ai dev-web test-api test-ai test-web build-web test check

help:
	@echo "Mensetsu Commands:"
	@echo "  make up          - Start all services (surrealdb, temporal, api, worker, web) via Docker Compose"
	@echo "  make down        - Stop all containers"
	@echo "  make infra-up    - Start only infrastructure (SurrealDB, Temporal) in background"
	@echo "  make infra-down  - Stop infrastructure containers"
	@echo "  make dev-api     - Run Rust API service locally"
	@echo "  make dev-ai      - Run Python Intelligence worker/service locally"
	@echo "  make dev-web     - Run Nuxt 3 web client locally"
	@echo "  make test        - Run all test suites"
	@echo "  make check       - Typecheck and lint packages"

up:
	docker compose up -d

down:
	docker compose down

infra-up:
	docker compose up -d surrealdb temporal-server temporal-ui

infra-down:
	docker compose down

build:
	docker compose build

dev-api:
	cd server/api && cargo run

dev-ai:
	cd server/intelligence && uv run python -m src.worker

dev-web:
	cd app/web && bun run dev

build-web:
	cd app/web && bun run build

test-api:
	cd server/api && cargo test

test-ai:
	cd server/intelligence && uv run pytest

test-web:
	cd app/web && bun x vitest run

test: test-api test-ai test-web

check:
	cd server/api && cargo check
	cd server/intelligence && uv run ruff check .
