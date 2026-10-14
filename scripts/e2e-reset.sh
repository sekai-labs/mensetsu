#!/usr/bin/env bash
set -euo pipefail

FAILURE_LOG="tests/e2e/.state/reset-failure.log"

dump_failure_diagnostics() {
  local exit_code=$?
  if [[ ${exit_code} -ne 0 ]]; then
    echo "[e2e-reset] Encountered failure (exit ${exit_code}). Dumping diagnostics to ${FAILURE_LOG}..." >&2
    mkdir -p "$(dirname "${FAILURE_LOG}")"
    {
      echo "=== Docker Compose Status ==="
      docker compose ps 2>&1 || true
      echo ""
      echo "=== Last 200 Logs: API ==="
      docker compose logs --tail=200 api 2>&1 || true
      echo ""
      echo "=== Last 200 Logs: Intelligence ==="
      docker compose logs --tail=200 intelligence 2>&1 || true
      echo ""
      echo "=== Last 200 Logs: Worker ==="
      docker compose logs --tail=200 worker 2>&1 || true
      echo ""
      echo "=== Last 200 Logs: Web ==="
      docker compose logs --tail=200 web 2>&1 || true
    } > "${FAILURE_LOG}"
  fi
}

trap dump_failure_diagnostics EXIT

if [[ "${MENSETSU_ALLOW_RESET:-0}" != "1" ]]; then
  echo "Error: e2e-reset refused because MENSETSU_ALLOW_RESET=1 is not set." >&2
  echo "Usage: MENSETSU_ALLOW_RESET=1 make e2e-reset" >&2
  exit 1
fi

COMPOSE_CONFIG="$(docker compose config --format json)"
COMPOSE_PROJECT="$(echo "${COMPOSE_CONFIG}" | jq -r '.name // "mensetsu"')"

DEFAULT_SURREAL_NS="$(echo "${COMPOSE_CONFIG}" | jq -r '.services.api.environment.SURREAL_NS // "mensetsu"')"
DEFAULT_SURREAL_DB="$(echo "${COMPOSE_CONFIG}" | jq -r '.services.api.environment.SURREAL_DB // "production"')"
SURREAL_USER="$(echo "${COMPOSE_CONFIG}" | jq -r '.services.api.environment.SURREAL_USER // "root"')"
SURREAL_PASS="$(echo "${COMPOSE_CONFIG}" | jq -r '.services.api.environment.SURREAL_PASS // "root"')"

SURREAL_NS="${RESET_SURREAL_NS:-${DEFAULT_SURREAL_NS}}"
SURREAL_DB="${RESET_SURREAL_DB:-${DEFAULT_SURREAL_DB}}"

if [[ "${RESET_DRY_RUN:-0}" == "1" ]]; then
  echo "[e2e-reset:DRY_RUN] Mode active. Simulating actions without executing changes."
  echo "[e2e-reset:DRY_RUN] Would stop services: api intelligence worker web"
  echo "[e2e-reset:DRY_RUN] Checking for orphan postgres container:"
  ORPHAN_CONTAINERS=$(docker ps -a --filter "name=mensetsu-postgres" --format '{{.ID}} {{.Names}} (status: {{.Status}})' || true)
  if [[ -n "${ORPHAN_CONTAINERS}" ]]; then
    echo "[e2e-reset:DRY_RUN] Found orphan container that would be removed by 'up --remove-orphans':"
    echo "${ORPHAN_CONTAINERS}" | sed 's/^/  - /'
  else
    echo "[e2e-reset:DRY_RUN] No orphan container named 'mensetsu-postgres' found."
  fi

  MATCHING_VOLUMES=$(docker volume ls --format '{{.Name}}' | grep -E "^${COMPOSE_PROJECT}.*postgres" || true)
  if [[ -n "${MATCHING_VOLUMES}" ]]; then
    echo "[e2e-reset:DRY_RUN] Would remove postgres volumes matching '^${COMPOSE_PROJECT}.*postgres':"
    echo "${MATCHING_VOLUMES}" | sed 's/^/  - /'
  else
    echo "[e2e-reset:DRY_RUN] No postgres volumes matching '^${COMPOSE_PROJECT}.*postgres' found."
  fi

  echo "[e2e-reset:DRY_RUN] Would purge Redis via FLUSHALL and verify DBSIZE == 0"
  echo "[e2e-reset:DRY_RUN] Would execute SurrealQL: REMOVE DATABASE ${SURREAL_DB} on ns=${SURREAL_NS}"
  echo "[e2e-reset:DRY_RUN] Would run: docker compose up -d --build --force-recreate api intelligence worker web"
  echo "[e2e-reset:DRY_RUN] Would verify worker heartbeat matching container hostname with TTL > 0"
  echo "[e2e-reset:DRY_RUN] Dry run simulation finished successfully."
  exit 0
fi

echo "[e2e-reset] Halting core application services (api, intelligence, worker, web)..."
docker compose stop api intelligence worker web

echo "[e2e-reset] Upstream refresh & orphan container cleanup..."
docker compose up -d --remove-orphans surrealdb redis

STILL_ORPHAN=$(docker ps -a --filter "name=mensetsu-postgres" -q)
if [[ -n "${STILL_ORPHAN}" ]]; then
  echo "Error: Orphan postgres container is still present after --remove-orphans: ${STILL_ORPHAN}" >&2
  exit 1
fi

POSTGRES_VOLS=$(docker volume ls --filter "label=com.docker.compose.project=mensetsu" --filter "label=com.docker.compose.project=docker" --format '{{.Name}}' | grep -E '(postgres|_postgres_data$)' || docker volume ls --format '{{.Name}}' | grep -E '(^mensetsu.*postgres|^docker.*postgres|_postgres_data$)' || true)
if [[ -n "${POSTGRES_VOLS}" ]]; then
  echo "[e2e-reset] Removing orphan postgres volume(s):"
  while IFS= read -r vol; do
    [[ -z "${vol}" ]] && continue
    echo "  - Removing volume: ${vol}"
    docker volume rm "${vol}"
  done <<< "${POSTGRES_VOLS}"
fi

echo "[e2e-reset] Purging Redis cache..."
docker compose exec -T redis redis-cli FLUSHALL >/dev/null
REDIS_SIZE="$(docker compose exec -T redis redis-cli DBSIZE | tr -d '\r\n[:space:]')"
if [[ "${REDIS_SIZE}" != "0" && "${REDIS_SIZE}" != ":0" ]]; then
  echo "Error: Redis DBSIZE is not 0 (found: ${REDIS_SIZE})" >&2
  exit 1
fi
echo "[e2e-reset] Redis verified empty (DBSIZE: 0)."

echo "[e2e-reset] Resetting SurrealDB database '${SURREAL_DB}' in namespace '${SURREAL_NS}'..."
docker compose exec -T surrealdb /surreal sql \
  --endpoint http://127.0.0.1:8000 \
  --username "${SURREAL_USER}" \
  --password "${SURREAL_PASS}" \
  --ns "${SURREAL_NS}" \
  --db "${SURREAL_DB}" <<EOF
REMOVE DATABASE ${SURREAL_DB};
EOF

echo "[e2e-reset] Recreating and building services (api, intelligence, worker, web)..."
docker compose up -d --build --force-recreate api intelligence worker web

echo "[e2e-reset] Health-checking all services..."
docker compose exec -T redis redis-cli ping >/dev/null
docker compose exec -T surrealdb /surreal is-ready >/dev/null

MAX_WAIT=90
INTERVAL=2
ELAPSED=0

API_READY=0
WEB_READY=0
INTELLIGENCE_READY=0
WORKER_READY=0

WORKER_HOSTNAME="$(docker compose exec -T worker hostname 2>/dev/null | tr -d '\r\n[:space:]' || echo "")"

while [[ ${ELAPSED} -lt ${MAX_WAIT} ]]; do
  if [[ -z "${WORKER_HOSTNAME}" ]]; then
    WORKER_HOSTNAME="$(docker compose exec -T worker hostname 2>/dev/null | tr -d '\r\n[:space:]' || echo "")"
  fi

  if [[ ${INTELLIGENCE_READY} -eq 0 ]]; then
    if docker compose exec -T intelligence python -c "import urllib.request,sys; sys.exit(0 if urllib.request.urlopen('http://127.0.0.1:8001/health/ready').status==200 else 1)" 2>/dev/null; then
      INTELLIGENCE_READY=1
    fi
  fi

  if [[ ${WORKER_READY} -eq 0 && -n "${WORKER_HOSTNAME}" ]]; then
    ALIVE_KEYS=$(docker compose exec -T redis redis-cli --scan --pattern "mensetsu:worker:*${WORKER_HOSTNAME}*:alive" | tr -d '\r' || true)

    if [[ -n "${ALIVE_KEYS}" ]]; then
      VALID_HEARTBEAT=0
      while IFS= read -r key; do
        if [[ -n "${key}" ]]; then
          KEY_TTL=$(docker compose exec -T redis redis-cli TTL "${key}" | tr -d '\r\n[:space:]')
          if [[ "${KEY_TTL}" =~ ^[0-9]+$ ]] && [[ "${KEY_TTL}" -gt 0 ]]; then
            VALID_HEARTBEAT=1
            break
          fi
        fi
      done <<< "${ALIVE_KEYS}"

      if [[ ${VALID_HEARTBEAT} -eq 1 ]]; then
        WORKER_READY=1
      fi
    fi
  fi

  if [[ ${API_READY} -eq 0 ]]; then
    HTTP_CODE="$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3000/health/ready || echo "000")"
    if [[ "${HTTP_CODE}" == "200" ]]; then
      API_READY=1
    fi
  fi

  if [[ ${WEB_READY} -eq 0 ]]; then
    HTTP_CODE="$(curl -s -o /dev/null -w "%{http_code}" http://127.0.0.1:3001/login || echo "000")"
    if [[ "${HTTP_CODE}" == "200" ]]; then
      WEB_READY=1
    fi
  fi

  if [[ ${INTELLIGENCE_READY} -eq 1 && ${WORKER_READY} -eq 1 && ${API_READY} -eq 1 && ${WEB_READY} -eq 1 ]]; then
    echo "[e2e-reset] All services are healthy and running with verified worker heartbeat and positive TTL."
    break
  fi

  sleep ${INTERVAL}
  ELAPSED=$((ELAPSED + INTERVAL))
done

if [[ ${API_READY} -ne 1 || ${WEB_READY} -ne 1 || ${INTELLIGENCE_READY} -ne 1 || ${WORKER_READY} -ne 1 ]]; then
  echo "Error: Service health readiness timed out after ${MAX_WAIT}s." >&2
  echo "Status - API: ${API_READY}, Web: ${WEB_READY}, Intelligence: ${INTELLIGENCE_READY}, Worker: ${WORKER_READY}" >&2
  exit 1
fi

echo "[e2e-reset] Verifying database tables via INFO FOR DB..."
SURREAL_INFO="$(docker compose exec -T surrealdb /surreal sql \
  --endpoint http://127.0.0.1:8000 \
  --username "${SURREAL_USER}" \
  --password "${SURREAL_PASS}" \
  --ns "${SURREAL_NS}" \
  --db "${SURREAL_DB}" <<'EOF'
INFO FOR DB;
EOF
)"

for tbl in "workflow_run" "execution_event" "agent_execution"; do
  if [[ "${SURREAL_INFO}" != *"${tbl}"* ]]; then
    echo "Error: Table '${tbl}' missing from SurrealDB schema after API boot." >&2
    exit 1
  fi
done

echo "[e2e-reset] Verified ledger tables exist (workflow_run, execution_event, agent_execution)."
echo "[e2e-reset] Stack reset complete and verified."
exit 0
