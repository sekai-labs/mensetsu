USE NS mensetsu DB production;

DEFINE TABLE IF NOT EXISTS application_dispatch SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE application_dispatch TYPE record<workspace>;
DEFINE FIELD OVERWRITE user_id ON TABLE application_dispatch TYPE record<user>;
DEFINE FIELD OVERWRITE application_id ON TABLE application_dispatch TYPE record<job_application>;
DEFINE FIELD OVERWRITE state ON TABLE application_dispatch TYPE string ASSERT $value IN ['pending', 'leased', 'delivered'] DEFAULT 'pending';
DEFINE FIELD OVERWRITE attempts ON TABLE application_dispatch TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE due_at ON TABLE application_dispatch TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE lease_until ON TABLE application_dispatch TYPE option<datetime>;
DEFINE FIELD OVERWRITE lease_token ON TABLE application_dispatch TYPE option<string>;
DEFINE FIELD OVERWRITE run_id ON TABLE application_dispatch TYPE option<string>;
DEFINE FIELD OVERWRITE last_error ON TABLE application_dispatch TYPE option<string>;
DEFINE FIELD OVERWRITE created_at ON TABLE application_dispatch TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE updated_at ON TABLE application_dispatch TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_application_dispatch_due ON TABLE application_dispatch COLUMNS state, due_at;

DEFINE FIELD OVERWRITE id ON TABLE saved_job TYPE string;
DEFINE INDEX IF NOT EXISTS idx_saved_job_user_created ON TABLE saved_job COLUMNS user_id, created_at, id;

LET $backfill_now = time::now();
LET $submitted_applications = (
    SELECT id, workspace_id, user_id
    FROM job_application
    WHERE status = 'submitted' AND workspace_id != NONE
);
FOR $application IN $submitted_applications {
    LET $workspace = SELECT id, user_id, status FROM ONLY $application.workspace_id;
    LET $existing_dispatch = SELECT VALUE id FROM application_dispatch WHERE workspace_id = $application.workspace_id LIMIT 1;
    LET $existing_run = SELECT VALUE id FROM workflow_run WHERE workspace_id = $application.workspace_id LIMIT 1;
    IF $workspace IS NOT NONE
        AND $workspace.user_id = $application.user_id
        AND array::len($existing_dispatch) = 0
        AND array::len($existing_run) = 0 {
        CREATE type::record('application_dispatch', record::id($application.workspace_id)) SET
            workspace_id = $application.workspace_id,
            user_id = $application.user_id,
            application_id = $application.id,
            state = 'pending',
            attempts = 0,
            due_at = $backfill_now,
            created_at = $backfill_now,
            updated_at = $backfill_now;
        IF $workspace.status = 'failed' {
            UPDATE $application.workspace_id SET
                status = 'pending',
                current_phase = 'preparing_input',
                progress_percent = 0,
                updated_at = $backfill_now;
        };
    };
};
