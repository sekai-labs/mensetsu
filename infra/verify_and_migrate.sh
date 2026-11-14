#!/usr/bin/env bash
set -euo pipefail

SURREAL_HOST="${SURREAL_HOST:-127.0.0.1:8000}"
SURREAL_USER="${SURREAL_USER:-root}"
SURREAL_PASS="${SURREAL_PASS:-root}"
SURREAL_NS="${SURREAL_NS:-mensetsu}"
SURREAL_DB="${SURREAL_DB:-production}"
if [[ "$SURREAL_NS" != mensetsu || "$SURREAL_DB" != production ]]; then
  echo "Migrations target mensetsu/production, but SURREAL_NS/SURREAL_DB is $SURREAL_NS/$SURREAL_DB; refusing to run." >&2
  exit 1
fi

apply_sql_file() {
  local path="$1" response
  response="$(curl --fail-with-body --silent --show-error -X POST "http://${SURREAL_HOST}/sql" \
    -u "${SURREAL_USER}:${SURREAL_PASS}" \
    -H "surreal-ns: ${SURREAL_NS}" -H "surreal-db: ${SURREAL_DB}" \
    -H "Accept: application/json" --data-binary "@$path")"
  if ! jq -e 'type == "array" and length > 0 and all(.[]; .status == "OK")' \
    <<< "$response" >/dev/null; then
    echo "SurrealDB rejected migration $path; refusing to continue." >&2
    exit 1
  fi
}


echo "==> Verifying SurrealDB connectivity at http://${SURREAL_HOST}/health..."
curl -s -f "http://${SURREAL_HOST}/health" > /dev/null || {
  echo "Error: SurrealDB is not reachable at http://${SURREAL_HOST}/health"
  exit 1
}
echo "SurrealDB is healthy."
echo "==> Preflighting attached uploads before CV library migration..."
LEGACY_ATTACHED="$(curl --fail-with-body --silent --show-error -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${SURREAL_NS}" -H "surreal-db: ${SURREAL_DB}" \
  -H "Accept: application/json" \
  --data "SELECT id FROM resume_upload WHERE status = 'attached' AND id NOT IN (SELECT VALUE source_upload_id FROM resume WHERE source_upload_id != NONE);")"
UNLINKED_IDS="$(jq -er 'if type == "array" and length == 1 and .[0].status == "OK"
  then (.[0].result | map(.id | tostring) | join("\n"))
  else error("CV library preflight SQL failed") end' <<< "$LEGACY_ATTACHED")"
if [[ -n "$UNLINKED_IDS" ]]; then
  echo "Error: attached upload(s) without resume.source_upload_id: $UNLINKED_IDS; refusing CV library migration." >&2
  exit 1
fi

echo "==> Applying additive resume object migration 010 (without replaying historical migrations)..."
apply_sql_file infra/migrations/010_resume_objects.surql

echo "==> Running legacy resume object backfill..."
cargo run --bin migrate-resume-objects --manifest-path server/api/Cargo.toml || {
  echo "Backfill migration failed or reported unmigratable legacy records. Aborting before dropping base64."
  exit 1
}

echo "==> Preflight check: ensuring zero remaining non-empty source_pdf_base64 records..."
CHECK_RESULT="$(curl --fail-with-body --silent --show-error -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${SURREAL_NS}" -H "surreal-db: ${SURREAL_DB}" \
  -H "Accept: application/json" \
  --data "SELECT count() AS count FROM resume WHERE source_pdf_base64 != NONE AND source_pdf_base64 != NULL AND source_pdf_base64 != '' GROUP ALL;")"
REMAINING="$(jq -er 'if type == "array" and length == 1 and .[0].status == "OK"
  then .[0].result[0].count else error("Preflight SQL failed") end
  | if type == "number" and . >= 0 then . else error("Invalid preflight count") end' \
  <<< "$CHECK_RESULT")"
if [ "$REMAINING" -gt 0 ]; then
  echo "Error: $REMAINING legacy resume record(s) still have non-empty source_pdf_base64! Refusing to drop column." >&2
  exit 1
fi

echo "==> Applying migration 011_drop_resume_base64.surql..."
apply_sql_file infra/migrations/011_drop_resume_base64.surql
echo "==> Applying migration 012_token_usage_event_index.surql..."
apply_sql_file infra/migrations/012_token_usage_event_index.surql
echo "==> Applying migration 013_run_idempotency.surql..."
apply_sql_file infra/migrations/013_run_idempotency.surql
echo "==> Applying CV library migration 014..."
apply_sql_file infra/migrations/014_cv_library.surql
echo "==> Checking submitted applications for inconsistent workspace references..."
ORPHANED_SUBMISSIONS="$(curl --fail-with-body --silent --show-error -X POST "http://${SURREAL_HOST}/sql" \
  -u "${SURREAL_USER}:${SURREAL_PASS}" \
  -H "surreal-ns: ${SURREAL_NS}" -H "surreal-db: ${SURREAL_DB}" \
  -H "Accept: application/json" \
  --data "SELECT id, workspace_id FROM job_application WHERE status = 'submitted' AND (workspace_id = NONE OR workspace_id NOT IN (SELECT VALUE id FROM workspace) OR workspace_id.user_id != user_id);")"
ORPHAN_LINES="$(jq -er 'if type == "array" and length == 1 and .[0].status == "OK"
  then (.[0].result // [] | map((.id | tostring) + " (workspace " + (.workspace_id | tostring) + ")") | join("\n"))
  else error("Submitted application preflight SQL failed") end' <<< "$ORPHANED_SUBMISSIONS")"
if [[ -n "$ORPHAN_LINES" ]]; then
  echo "Warning: submitted application(s) without an owned workspace require manual repair; no workspace or dispatch row will be invented:" >&2
  printf '%s\n' "$ORPHAN_LINES" >&2
fi
echo "==> Applying saved jobs and application flow migration 015..."
apply_sql_file infra/migrations/015_saved_jobs_application_flow.surql
echo "==> Applying application dispatch and saved job pagination index migration 016..."
apply_sql_file infra/migrations/016_application_dispatch_and_saved_job_index.surql

echo "==> Applying final schema..."
apply_sql_file server/api/src/schema.surql

echo "Migration executed successfully."
echo "SurrealDB schema configured for namespace: ${SURREAL_NS}, database: ${SURREAL_DB}."
