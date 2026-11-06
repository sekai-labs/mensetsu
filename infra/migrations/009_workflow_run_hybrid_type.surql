
DEFINE FIELD OVERWRITE type ON TABLE workflow_run TYPE string ASSERT $value IN ['static', 'dynamic', 'hybrid'];
