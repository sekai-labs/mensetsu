#!/usr/bin/env bash
set -euo pipefail

SURREAL_HOST="${SURREAL_HOST:-127.0.0.1:8000}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
TEST_NS="mensetsu_test"
TEST_DB="unit_tests"

echo "==> Running Database Schema Test on ${TEST_NS}/${TEST_DB}..."

curl -s -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${TEST_NS}" \
  -H "surreal-db: ${TEST_DB}" \
  -H "Accept: application/json" \
  --data-binary "@infra/migrations/001_initial_schema.surql" > /dev/null

QUERY="
USE NS ${TEST_NS} DB ${TEST_DB};
CREATE skill:rust SET name = 'Rust Systems Programming', slug = 'rust', category = 'Language';
CREATE skill:async SET name = 'Asynchronous Concurrency', slug = 'async', category = 'Concurrency';
RELATE skill:rust->rel_depends_on->skill:async SET order = 1;
SELECT ->rel_depends_on->skill.name AS prerequisites FROM skill:rust;
"

RESULT=$(curl -s -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${TEST_NS}" \
  -H "surreal-db: ${TEST_DB}" \
  -H "Accept: application/json" \
  --data "${QUERY}")

echo "Query result:"
echo "${RESULT}" | grep -q "Asynchronous Concurrency" && echo "Graph traversal test PASSED." || {
  echo "Graph traversal test FAILED: ${RESULT}"
  exit 1
}

curl -s -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${TEST_NS}" \
  -H "surreal-db: ${TEST_DB}" \
  --data "REMOVE NAMESPACE ${TEST_NS};" > /dev/null

echo "==> Schema and Graph Traversal Verified Successfully."
