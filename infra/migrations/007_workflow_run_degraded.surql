USE NS mensetsu DB production;

DEFINE FIELD OVERWRITE degraded ON TABLE workflow_run TYPE bool DEFAULT false;
DEFINE FIELD OVERWRITE degraded_reasons ON TABLE workflow_run TYPE array<string> DEFAULT [];
