use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::shared::record_id::clean_record_id;

pub type JsonObject = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    Static,
    Dynamic,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Queued,
    Running,
    Completed,
    Failed,
    Retrying,
    Blocked,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationStatus {
    Requested,
    Accepted,
    Running,
    Completed,
    Failed,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogSeverity {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Network,
    RateLimit,
    BudgetExceeded,
    LlmError,
    ValidationError,
    InternalError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    ResearchReport,
    Finding,
    Evidence,
    QuestionSet,
    Recommendation,
    ValidationResult,
    RoutingDecision,
}

fn default_one() -> i64 {
    1
}
fn default_task_status() -> TaskStatus {
    TaskStatus::Pending
}
fn default_execution_status() -> ExecutionStatus {
    ExecutionStatus::Pending
}
fn default_tool_status() -> ToolStatus {
    ToolStatus::Pending
}
fn default_delegation_status() -> DelegationStatus {
    DelegationStatus::Requested
}
fn default_severity() -> LogSeverity {
    LogSeverity::Info
}
fn default_true() -> bool {
    true
}
fn default_version() -> String {
    "1.0.0".into()
}
fn default_schema_version() -> String {
    "1.0".into()
}
fn default_component() -> String {
    "orchestration".into()
}
fn default_current_phase() -> String {
    "initialization".into()
}
fn default_group() -> String {
    "default".into()
}
fn default_task_kind() -> String {
    "task".into()
}
fn default_agent_kind() -> String {
    "agent".into()
}
fn default_max_retries() -> i64 {
    3
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields, default)]
pub struct WorkflowProgress {
    pub total_tasks: i64,
    pub pending: i64,
    pub queued: i64,
    pub running: i64,
    pub completed: i64,
    pub failed: i64,
    pub retrying: i64,
    pub blocked: i64,
    pub cancelled: i64,
    pub skipped: i64,
    pub percentage: Option<i64>,
    pub estimate: bool,
    pub phase_index: Option<i64>,
    pub phase_count: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub workflow_type: WorkflowType,
    #[serde(default = "default_version")]
    pub version: String,
    pub description: String,
    #[serde(default = "default_max_retries")]
    pub default_max_retries: i64,
    #[serde(default)]
    pub supported_agents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRun {
    pub id: String,
    pub workflow_id: String,
    pub workflow_name: String,
    pub workspace_id: String,
    pub user_id: String,
    #[serde(rename = "type")]
    pub workflow_type: WorkflowType,
    pub status: RunStatus,
    #[serde(default = "default_current_phase")]
    pub current_phase: String,
    #[serde(default = "default_one")]
    pub plan_revision: i64,
    #[serde(default)]
    pub progress: WorkflowProgress,
    #[serde(default)]
    pub last_seq: i64,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub duration_ms: Option<f64>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub degraded: bool,
    #[serde(default)]
    pub degraded_reasons: Vec<String>,
}

impl WorkflowRun {
    pub fn bare_id(&self) -> &str {
        bare_run_id(&self.id)
    }

    pub fn normalize_record_ids(&mut self) {
        self.id = clean_record_id(&self.id);
        self.workspace_id = clean_record_id(&self.workspace_id);
    }
}

pub fn bare_run_id(id: &str) -> &str {
    let id = id.strip_prefix("workflow_run:").unwrap_or(id);
    id.trim_matches('`')
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowTask {
    pub id: String,
    pub run_id: String,
    pub task_type: String,
    pub name: String,
    pub assigned_agent: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default = "default_task_status")]
    pub status: TaskStatus,
    #[serde(default = "default_one")]
    pub attempt: i64,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default = "default_one")]
    pub added_in_revision: i64,
    #[serde(default)]
    pub execution_ids: Vec<String>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub input_data: JsonObject,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub output_data: JsonObject,
    #[serde(default)]
    pub error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionNote {
    pub ts: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields, default)]
pub struct ExecutionBudgets {
    pub model_calls: i64,
    pub tool_calls: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub wall_time_ms: f64,
    pub external_requests: i64,
}

impl Default for ExecutionBudgets {
    fn default() -> Self {
        Self {
            model_calls: 10,
            tool_calls: 20,
            tokens_in: 50_000,
            tokens_out: 10_000,
            wall_time_ms: 120_000.0,
            external_requests: 25,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields, default)]
pub struct ExecutionBudgetsUsed {
    pub model_calls: i64,
    pub tool_calls: i64,
    pub tokens_in: i64,
    pub tokens_out: i64,
    pub wall_time_ms: f64,
    pub external_requests: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AgentExecution {
    pub id: String,
    pub run_id: String,
    pub task_id: String,
    #[serde(default)]
    pub parent_execution_id: Option<String>,
    #[serde(default)]
    pub child_execution_ids: Vec<String>,
    pub agent_id: String,
    #[serde(default)]
    pub objective: String,
    #[serde(default = "default_execution_status")]
    pub status: ExecutionStatus,
    #[serde(default)]
    pub delegation_depth: i64,
    #[serde(default)]
    pub retries: i64,
    #[serde(default)]
    pub current_activity: Option<String>,
    #[serde(default)]
    pub model_provider: String,
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub budgets: ExecutionBudgets,
    #[serde(default)]
    pub budgets_used: ExecutionBudgetsUsed,
    #[serde(default)]
    pub tool_call_count: i64,
    #[serde(default)]
    pub notes: Vec<ExecutionNote>,
    #[serde(default)]
    pub artifact_ids: Vec<String>,
    #[serde(default)]
    pub public_summary: String,
    #[serde(default)]
    pub error: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocation {
    pub id: String,
    pub execution_id: String,
    pub tool_name: String,
    #[serde(default = "default_tool_status")]
    pub status: ToolStatus,
    #[serde(default)]
    pub duration_ms: f64,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub sanitized_input: JsonObject,
    #[serde(default)]
    pub output_summary: String,
    #[serde(default)]
    pub output_ref: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub started_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Delegation {
    pub id: String,
    pub run_id: String,
    pub parent_execution_id: String,
    pub child_execution_id: String,
    pub parent_agent_id: String,
    pub child_agent_id: String,
    pub delegation_task_id: String,
    pub objective: String,
    pub reason: String,
    #[serde(default = "default_one")]
    pub depth: i64,
    #[serde(default = "default_delegation_status")]
    pub status: DelegationStatus,
    #[serde(default)]
    pub input_artifact_ids: Vec<String>,
    #[serde(default)]
    pub output_artifact_ids: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceLink {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProvenance {
    pub source_url: String,
    pub title: String,
    #[serde(default)]
    pub retrieved_at: Option<String>,
    pub snippet: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum ArtifactContent {
    #[schema(value_type = Object)]
    Object(JsonObject),
    Text(String),
}

impl Default for ArtifactContent {
    fn default() -> Self {
        Self::Object(JsonObject::new())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: String,
    pub run_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub agent_execution_id: Option<String>,
    #[serde(rename = "type")]
    pub artifact_type: ArtifactType,
    pub title: String,
    #[serde(default = "default_one")]
    pub version: i64,
    #[serde(default)]
    pub surreal_record_id: Option<String>,
    #[serde(default)]
    pub evidence_ids: Vec<String>,
    #[serde(default)]
    pub source_links: Vec<SourceLink>,
    #[serde(default)]
    pub evidence: Option<EvidenceProvenance>,
    #[serde(default)]
    pub content: ArtifactContent,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LogEntry {
    pub id: String,
    pub run_id: String,
    #[serde(default)]
    pub seq: Option<i64>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub execution_id: Option<String>,
    #[serde(default)]
    pub tool_invocation_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[serde(default = "default_component")]
    pub component: String,
    #[serde(default = "default_severity")]
    pub severity: LogSeverity,
    pub message: String,
    #[serde(default)]
    pub duration_ms: Option<f64>,
    #[serde(default)]
    pub error_category: Option<ErrorCategory>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub attributes: JsonObject,
    pub ts: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum ExecutionEventType {
    #[serde(rename = "agent.activity")]
    AgentActivity,
    #[serde(rename = "agent.completed")]
    AgentCompleted,
    #[serde(rename = "agent.failed")]
    AgentFailed,
    #[serde(rename = "agent.started")]
    AgentStarted,
    #[serde(rename = "artifact.created")]
    ArtifactCreated,
    #[serde(rename = "checkpoint.created")]
    CheckpointCreated,
    #[serde(rename = "delegation.accepted")]
    DelegationAccepted,
    #[serde(rename = "delegation.completed")]
    DelegationCompleted,
    #[serde(rename = "delegation.failed")]
    DelegationFailed,
    #[serde(rename = "delegation.requested")]
    DelegationRequested,
    #[serde(rename = "delegation.started")]
    DelegationStarted,
    #[serde(rename = "evidence.saved")]
    EvidenceSaved,
    #[serde(rename = "execution.resumed")]
    ExecutionResumed,
    #[serde(rename = "finding.saved")]
    FindingSaved,
    #[serde(rename = "plan.created")]
    PlanCreated,
    #[serde(rename = "plan.revised")]
    PlanRevised,
    #[serde(rename = "task.added")]
    TaskAdded,
    #[serde(rename = "task.blocked")]
    TaskBlocked,
    #[serde(rename = "task.completed")]
    TaskCompleted,
    #[serde(rename = "task.failed")]
    TaskFailed,
    #[serde(rename = "task.queued")]
    TaskQueued,
    #[serde(rename = "task.removed")]
    TaskRemoved,
    #[serde(rename = "task.retrying")]
    TaskRetrying,
    #[serde(rename = "task.started")]
    TaskStarted,
    #[serde(rename = "tool.completed")]
    ToolCompleted,
    #[serde(rename = "tool.failed")]
    ToolFailed,
    #[serde(rename = "tool.started")]
    ToolStarted,
    #[serde(rename = "worker.recovered")]
    WorkerRecovered,
    #[serde(rename = "workflow.cancelled")]
    WorkflowCancelled,
    #[serde(rename = "workflow.completed")]
    WorkflowCompleted,
    #[serde(rename = "workflow.created")]
    WorkflowCreated,
    #[serde(rename = "workflow.failed")]
    WorkflowFailed,
    #[serde(rename = "workflow.retrying")]
    WorkflowRetrying,
    #[serde(rename = "workflow.started")]
    WorkflowStarted,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct WorkflowRetryingPayload {
    pub attempt: i64,
    pub next_attempt: i64,
    pub delay_ms: i64,
    pub error_category: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionEvent {
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
    pub event_id: String,
    pub run_id: String,
    pub seq: i64,
    pub ts: String,
    #[serde(rename = "type")]
    pub event_type: ExecutionEventType,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub execution_id: Option<String>,
    #[serde(default)]
    pub parent_execution_id: Option<String>,
    #[serde(default)]
    pub tool_invocation_id: Option<String>,
    #[serde(default)]
    pub correlation_id: Option<String>,
    #[serde(default)]
    pub trace_id: Option<String>,
    #[schema(value_type = Object)]
    pub payload: JsonObject,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct RunsPageResponse {
    pub items: Vec<WorkflowRun>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct LogsPageResponse {
    pub items: Vec<LogEntry>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct EventsHistoryPageResponse {
    pub events: Vec<ExecutionEvent>,
    #[serde(default)]
    pub next_after_seq: Option<i64>,
    #[serde(default)]
    pub last_seq: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationsPageResponse {
    pub items: Vec<ToolInvocation>,
    #[serde(default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolOutputResponse {
    pub tool_invocation_id: String,
    pub offset: i64,
    pub limit: i64,
    pub total_bytes: i64,
    pub has_more: bool,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskGraphNodeData {
    pub status: TaskStatus,
    pub assigned_agent: String,
    pub attempt: i64,
    pub task_type: String,
    pub required: bool,
    pub added_in_revision: i64,
    #[serde(default)]
    pub execution_ids: Vec<String>,
    #[serde(default = "default_group")]
    pub group: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskGraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type", default = "default_task_kind")]
    pub node_type: String,
    pub data: TaskGraphNodeData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DependencyEdgeKind {
    Dependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(default = "dependency_kind")]
    pub kind: DependencyEdgeKind,
}

fn dependency_kind() -> DependencyEdgeKind {
    DependencyEdgeKind::Dependency
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct TaskGraphResponse {
    pub nodes: Vec<TaskGraphNode>,
    pub edges: Vec<TaskGraphEdge>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelegationGraphNodeData {
    pub status: ExecutionStatus,
    pub task_id: String,
    pub agent_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelegationGraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type", default = "default_agent_kind")]
    pub node_type: String,
    pub data: DelegationGraphNodeData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelegationGraphEdgeData {
    pub delegation_id: String,
    pub objective: String,
    pub status: DelegationStatus,
    pub created_at: String,
    pub parent_agent_id: String,
    pub child_agent_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DelegationEdgeKind {
    Delegation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelegationGraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(default = "delegation_kind")]
    pub kind: DelegationEdgeKind,
    pub data: DelegationGraphEdgeData,
}

fn delegation_kind() -> DelegationEdgeKind {
    DelegationEdgeKind::Delegation
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct DelegationGraphResponse {
    pub nodes: Vec<DelegationGraphNode>,
    pub edges: Vec<DelegationGraphEdge>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn fixture(name: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("contracts/fixtures/workflows.v1")
            .join(name);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn strict<T: serde::de::DeserializeOwned>(name: &str) -> T {
        serde_json::from_str(&fixture(name)).unwrap_or_else(|e| panic!("{name} violates DTO: {e}"))
    }

    #[test]
    fn every_fixture_deserializes_into_its_dto() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/fixtures/workflows.v1");
        let mut seen: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        seen.sort();

        let run: WorkflowRun = strict("run.json");
        assert!(run.bare_id().starts_with("run_"), "unexpected run id {}", run.id);
        let _: Vec<AgentExecution> = strict("agents.json");
        let artifacts: Vec<Artifact> = strict("artifacts.json");
        assert!(!artifacts.is_empty());
        let _: DelegationGraphResponse = strict("delegations-graph.json");
        let graph: TaskGraphResponse = strict("graph.json");
        assert!(!graph.nodes.is_empty());

        let events: Vec<ExecutionEvent> = fixture("events.ndjson")
            .lines()
            .filter(|l| !l.trim().is_empty())
            .enumerate()
            .map(|(i, l)| {
                serde_json::from_str(l).unwrap_or_else(|e| panic!("events.ndjson line {}: {e}", i + 1))
            })
            .collect();
        assert!(!events.is_empty());
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq), "seq must be monotonic");
        assert!(events.iter().all(|e| e.run_id == run.bare_id()), "events from another run");
        assert_eq!(events.last().unwrap().seq, run.last_seq);

        let mut parser = crate::interfaces::http::sse_parser::SseParser::new();
        let frames = parser.feed(fixture("sse.txt").as_bytes());
        assert_eq!(frames.len(), events.len());
        for (frame, event) in frames.iter().zip(&events) {
            let parsed: ExecutionEvent = serde_json::from_str(&frame.data).unwrap();
            assert_eq!(&parsed, event);
            assert_eq!(frame.id.as_deref(), Some(event.seq.to_string().as_str()));
            let framed: ExecutionEventType =
                serde_json::from_value(serde_json::Value::String(frame.event.clone().unwrap())).unwrap();
            assert_eq!(framed, event.event_type);
        }

        assert_eq!(
            seen,
            [
                "agents.json",
                "artifacts.json",
                "delegations-graph.json",
                "events.ndjson",
                "graph.json",
                "run.json",
                "sse.txt"
            ],
            "unclaimed fixture: add a DTO assertion for it above"
        );
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let mut run: serde_json::Value = serde_json::from_str(&fixture("run.json")).unwrap();
        run["surprise"] = serde_json::json!(1);
        assert!(serde_json::from_value::<WorkflowRun>(run).is_err());
    }

    #[test]
    fn event_payload_must_be_an_object() {
        let line = fixture("events.ndjson").lines().next().unwrap().to_string();
        let mut ev: serde_json::Value = serde_json::from_str(&line).unwrap();
        ev["payload"] = serde_json::json!("not-an-object");
        assert!(serde_json::from_value::<ExecutionEvent>(ev).is_err());
    }

    #[test]
    fn event_types_match_schema() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("contracts/schema/workflows.v1.json");
        let schema: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut from_schema: Vec<String> = schema["$defs"]
            .as_object()
            .unwrap()
            .iter()
            .filter(|(name, _)| name.ends_with("Event"))
            .map(|(_, def)| def["properties"]["type"]["const"].as_str().unwrap().to_string())
            .collect();
        from_schema.sort();

        let mut accepted: Vec<String> = from_schema
            .iter()
            .filter(|t| serde_json::from_value::<ExecutionEventType>(serde_json::Value::String((*t).clone())).is_ok())
            .cloned()
            .collect();
        accepted.sort();
        assert_eq!(accepted, from_schema, "schema event types missing from ExecutionEventType");

        let openapi = serde_json::to_value(<ExecutionEventType as utoipa::PartialSchema>::schema()).unwrap();
        let mut from_rust: Vec<String> =
            openapi["enum"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        from_rust.sort();
        assert_eq!(from_rust, from_schema, "ExecutionEventType has variants the contract lacks");
    }

    #[test]
    fn workflow_retrying_event_round_trips() {
        let raw = serde_json::json!({
            "schema_version": "1.0", "event_id": "evt_1", "run_id": "run_x", "seq": 7,
            "ts": "2026-10-13T00:00:00Z", "type": "workflow.retrying",
            "task_id": null, "execution_id": null, "parent_execution_id": null,
            "tool_invocation_id": null, "correlation_id": null, "trace_id": null,
            "payload": {"attempt": 1, "next_attempt": 2, "delay_ms": 5000, "error_category": "network"}
        });
        let ev: ExecutionEvent = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(ev.event_type, ExecutionEventType::WorkflowRetrying);
        let payload: WorkflowRetryingPayload =
            serde_json::from_value(serde_json::Value::Object(ev.payload.clone())).unwrap();
        assert_eq!((payload.attempt, payload.next_attempt, payload.delay_ms), (1, 2, 5000));
        assert_eq!(serde_json::to_value(&ev).unwrap(), raw);

        let mut bad = raw["payload"].clone();
        bad["extra"] = serde_json::json!(true);
        assert!(serde_json::from_value::<WorkflowRetryingPayload>(bad).is_err());
        let mut unknown = raw;
        unknown["type"] = serde_json::json!("workflow.teleported");
        assert!(serde_json::from_value::<ExecutionEvent>(unknown).is_err());
    }
}
