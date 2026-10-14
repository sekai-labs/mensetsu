USE NS mensetsu DB production;

DEFINE FIELD OVERWRITE totp_enabled ON TABLE user TYPE bool DEFAULT false;

DEFINE FIELD OVERWRITE workspace_id ON TABLE skill TYPE option<record<workspace>>;
DEFINE FIELD OVERWRITE candidate_status ON TABLE skill TYPE option<string>;
DEFINE FIELD OVERWRITE importance ON TABLE skill TYPE option<string>;
DEFINE INDEX IF NOT EXISTS idx_skill_ws ON TABLE skill COLUMNS workspace_id;

DEFINE TABLE OVERWRITE skill_relation SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE skill_relation TYPE option<record<workspace>>;
DEFINE FIELD OVERWRITE source ON TABLE skill_relation TYPE string;
DEFINE FIELD OVERWRITE target ON TABLE skill_relation TYPE string;
DEFINE FIELD OVERWRITE relation ON TABLE skill_relation TYPE string;
DEFINE INDEX IF NOT EXISTS idx_skill_rel_ws ON TABLE skill_relation COLUMNS workspace_id;

DEFINE FIELD OVERWRITE chapters.* ON TABLE learning_unit TYPE any;
DEFINE FIELD OVERWRITE diagram.* ON TABLE learning_unit TYPE any;

DEFINE TABLE OVERWRITE pipeline_execution SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE pipeline_execution TYPE record<workspace>;
DEFINE FIELD OVERWRITE status ON TABLE pipeline_execution TYPE string;
DEFINE FIELD OVERWRITE current_step ON TABLE pipeline_execution TYPE string;
DEFINE FIELD OVERWRITE total_steps ON TABLE pipeline_execution TYPE int DEFAULT 8;
DEFINE FIELD OVERWRITE completed_steps ON TABLE pipeline_execution TYPE int DEFAULT 0;
DEFINE FIELD OVERWRITE error_message ON TABLE pipeline_execution TYPE option<string>;
DEFINE FIELD OVERWRITE started_at ON TABLE pipeline_execution TYPE datetime DEFAULT time::now();
DEFINE FIELD OVERWRITE completed_at ON TABLE pipeline_execution TYPE option<datetime>;
DEFINE INDEX IF NOT EXISTS idx_pipeline_exec_ws ON TABLE pipeline_execution COLUMNS workspace_id;
DEFINE FIELD OVERWRITE details.* ON TABLE pipeline_execution TYPE any;

DEFINE TABLE OVERWRITE pipeline_log SCHEMAFULL;
DEFINE FIELD OVERWRITE workspace_id ON TABLE pipeline_log TYPE record<workspace>;
DEFINE FIELD OVERWRITE execution_id ON TABLE pipeline_log TYPE string;
DEFINE FIELD OVERWRITE step ON TABLE pipeline_log TYPE string;
DEFINE FIELD OVERWRITE level ON TABLE pipeline_log TYPE string ASSERT $value IN ['info', 'warn', 'error', 'debug'];
DEFINE FIELD OVERWRITE message ON TABLE pipeline_log TYPE string;
DEFINE FIELD OVERWRITE details ON TABLE pipeline_log TYPE option<object>;
DEFINE FIELD OVERWRITE timestamp ON TABLE pipeline_log TYPE datetime DEFAULT time::now();
DEFINE INDEX IF NOT EXISTS idx_pipeline_log_ws ON TABLE pipeline_log COLUMNS workspace_id, timestamp;
DEFINE FIELD OVERWRITE details.* ON TABLE pipeline_log TYPE any;
