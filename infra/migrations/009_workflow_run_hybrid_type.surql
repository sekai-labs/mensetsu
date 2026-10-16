-- Migration: 009_workflow_run_hybrid_type.surql
-- Allow 'hybrid' workflow type in workflow_run table

DEFINE FIELD OVERWRITE type ON TABLE workflow_run TYPE string ASSERT $value IN ['static', 'dynamic', 'hybrid'];
