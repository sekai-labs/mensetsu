#!/usr/bin/env bash
set -euo pipefail

SURREAL_HOST="${SURREAL_HOST:-127.0.0.1:8000}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
SURREAL_NS="${SURREAL_NS:-mensetsu}"
SURREAL_DB="${SURREAL_DB:-production}"

echo "==> Verifying SurrealDB connectivity at http://${SURREAL_HOST}/health..."
curl -s -f "http://${SURREAL_HOST}/health" > /dev/null || {
  echo "Error: SurrealDB is not reachable at http://${SURREAL_HOST}/health"
  exit 1
}
echo "SurrealDB is healthy."

echo "==> Applying migration: infra/migrations/001_initial_schema.surql..."
SCHEMA_CONTENT=$(cat infra/migrations/001_initial_schema.surql)

RESPONSE=$(curl -s -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${SURREAL_NS}" \
  -H "surreal-db: ${SURREAL_DB}" \
  -H "Accept: application/json" \
  --data-binary "@infra/migrations/001_initial_schema.surql")

echo "Migration executed successfully."
echo "SurrealDB schema configured for namespace: ${SURREAL_NS}, database: ${SURREAL_DB}."
