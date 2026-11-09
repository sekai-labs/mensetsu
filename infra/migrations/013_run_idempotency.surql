USE NS mensetsu DB production;

DEFINE TABLE IF NOT EXISTS run_idempotency SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS request_hash ON TABLE run_idempotency TYPE string;
DEFINE FIELD IF NOT EXISTS run_id ON TABLE run_idempotency TYPE record<workflow_run>;
DEFINE FIELD IF NOT EXISTS user_id ON TABLE run_idempotency TYPE string;
DEFINE FIELD IF NOT EXISTS workspace_id ON TABLE run_idempotency TYPE record<workspace>;
DEFINE FIELD IF NOT EXISTS operation ON TABLE run_idempotency TYPE string;
DEFINE FIELD IF NOT EXISTS created_at ON TABLE run_idempotency TYPE datetime DEFAULT time::now();
