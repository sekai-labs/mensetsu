from datetime import datetime, timezone
from typing import Annotated, Any, Literal, Union

from pydantic import BaseModel, ConfigDict, Field


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


WorkflowType = Literal["static", "dynamic"]
RunStatus = Literal["pending", "running", "completed", "failed", "cancelled"]
TaskStatus = Literal[
    "pending",
    "queued",
    "running",
    "completed",
    "failed",
    "retrying",
    "blocked",
    "cancelled",
    "skipped",
]
ExecutionStatus = Literal["pending", "running", "completed", "failed", "cancelled"]
ToolStatus = Literal["pending", "running", "completed", "failed"]
DelegationStatus = Literal["requested", "accepted", "running", "completed", "failed", "rejected"]
LogSeverity = Literal["debug", "info", "warn", "error"]
ErrorCategory = Literal[
    "network", "rate_limit", "budget_exceeded", "llm_error", "validation_error", "internal_error"
]
ArtifactType = Literal[
    "research_report", "finding", "evidence", "question_set", "recommendation", "validation_result"
]


class WorkflowProgress(BaseModel):
    model_config = ConfigDict(extra="forbid")

    total_tasks: int = 0
    pending: int = 0
    queued: int = 0
    running: int = 0
    completed: int = 0
    failed: int = 0
    retrying: int = 0
    blocked: int = 0
    cancelled: int = 0
    skipped: int = 0
    percentage: int | None = None
    estimate: bool = False
    phase_index: int | None = None
    phase_count: int | None = None


class WorkflowDefinition(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    name: str
    type: WorkflowType
    version: str = "1.0.0"
    description: str
    default_max_retries: int = 3
    supported_agents: list[str] = Field(default_factory=list)


class WorkflowRun(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    workflow_id: str
    workflow_name: str
    workspace_id: str
    user_id: str
    type: WorkflowType
    status: RunStatus
    current_phase: str = "initialization"
    plan_revision: int = 1
    progress: WorkflowProgress = Field(default_factory=WorkflowProgress)
    last_seq: int = 0
    started_at: datetime | None = None
    completed_at: datetime | None = None
    duration_ms: float | None = None
    created_at: datetime = Field(default_factory=utc_now)
    updated_at: datetime = Field(default_factory=utc_now)
    error_message: str | None = None
    degraded: bool = False
    degraded_reasons: list[str] = Field(default_factory=list)


class WorkflowTask(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    run_id: str
    task_type: str
    name: str
    assigned_agent: str
    dependencies: list[str] = Field(default_factory=list)
    status: TaskStatus = "pending"
    attempt: int = 1
    required: bool = True
    added_in_revision: int = 1
    execution_ids: list[str] = Field(default_factory=list)
    input_data: dict[str, Any] = Field(default_factory=dict)
    output_data: dict[str, Any] = Field(default_factory=dict)
    error: str | None = None
    created_at: datetime = Field(default_factory=utc_now)
    updated_at: datetime = Field(default_factory=utc_now)


class ExecutionNote(BaseModel):
    model_config = ConfigDict(extra="forbid")

    ts: datetime = Field(default_factory=utc_now)
    text: str


class ExecutionBudgets(BaseModel):
    model_config = ConfigDict(extra="forbid")

    model_calls: int = 10
    tool_calls: int = 20
    tokens_in: int = 50000
    tokens_out: int = 10000
    wall_time_ms: float = 120000.0
    external_requests: int = 25


class ExecutionBudgetsUsed(BaseModel):
    model_config = ConfigDict(extra="forbid")

    model_calls: int = 0
    tool_calls: int = 0
    tokens_in: int = 0
    tokens_out: int = 0
    wall_time_ms: float = 0.0
    external_requests: int = 0


class ToolInvocation(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    execution_id: str
    tool_name: str
    status: ToolStatus = "pending"
    duration_ms: float = 0.0
    sanitized_input: dict[str, Any] = Field(default_factory=dict)
    output_summary: str = ""
    output_ref: str | None = None
    error: str | None = None
    started_at: datetime = Field(default_factory=utc_now)
    completed_at: datetime | None = None


class Delegation(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    run_id: str
    parent_execution_id: str
    child_execution_id: str
    parent_agent_id: str
    child_agent_id: str
    delegation_task_id: str
    objective: str
    reason: str
    depth: int = 1
    status: DelegationStatus = "requested"
    input_artifact_ids: list[str] = Field(default_factory=list)
    output_artifact_ids: list[str] = Field(default_factory=list)
    error: str | None = None
    created_at: datetime = Field(default_factory=utc_now)
    completed_at: datetime | None = None


class AgentExecution(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    run_id: str
    task_id: str
    parent_execution_id: str | None = None
    child_execution_ids: list[str] = Field(default_factory=list)
    agent_id: str
    objective: str = ""
    status: ExecutionStatus = "pending"
    delegation_depth: int = 0
    retries: int = 0
    current_activity: str | None = None
    model_provider: str = ""
    model_name: str = ""
    budgets: ExecutionBudgets = Field(default_factory=ExecutionBudgets)
    budgets_used: ExecutionBudgetsUsed = Field(default_factory=ExecutionBudgetsUsed)
    tool_call_count: int = 0
    notes: list[ExecutionNote] = Field(default_factory=list)
    artifact_ids: list[str] = Field(default_factory=list)
    public_summary: str = ""
    error: str | None = None
    started_at: datetime = Field(default_factory=utc_now)
    completed_at: datetime | None = None


class SourceLink(BaseModel):
    model_config = ConfigDict(extra="forbid")

    title: str
    url: str


class EvidenceProvenance(BaseModel):
    model_config = ConfigDict(extra="forbid")

    source_url: str
    title: str
    retrieved_at: datetime = Field(default_factory=utc_now)
    snippet: str
    confidence: float = Field(ge=0.0, le=1.0)


class Artifact(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    run_id: str
    task_id: str | None = None
    agent_execution_id: str | None = None
    type: ArtifactType
    title: str
    version: int = 1
    surreal_record_id: str | None = None
    evidence_ids: list[str] = Field(default_factory=list)
    source_links: list[SourceLink] = Field(default_factory=list)
    evidence: EvidenceProvenance | None = None
    content: dict[str, Any] | str = Field(default_factory=dict)
    created_at: datetime = Field(default_factory=utc_now)


class LogEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    run_id: str
    seq: int | None = None
    task_id: str | None = None
    execution_id: str | None = None
    tool_invocation_id: str | None = None
    trace_id: str | None = None
    component: str = "orchestration"
    severity: LogSeverity = "info"
    message: str
    duration_ms: float | None = None
    error_category: ErrorCategory | None = None
    attributes: dict[str, Any] = Field(default_factory=dict)
    ts: datetime = Field(default_factory=utc_now)


class TaskGraphNodeData(BaseModel):
    model_config = ConfigDict(extra="forbid")

    status: TaskStatus
    assigned_agent: str
    attempt: int
    task_type: str
    required: bool
    added_in_revision: int
    execution_ids: list[str] = Field(default_factory=list)
    group: str = "default"


class TaskGraphNode(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    label: str
    type: str = "task"
    data: TaskGraphNodeData


class TaskGraphEdge(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    source: str
    target: str
    kind: Literal["dependency"] = "dependency"


class TaskGraphResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    nodes: list[TaskGraphNode]
    edges: list[TaskGraphEdge]


class DelegationGraphNodeData(BaseModel):
    model_config = ConfigDict(extra="forbid")

    status: ExecutionStatus
    task_id: str
    agent_id: str


class DelegationGraphNode(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    label: str
    type: str = "agent"
    data: DelegationGraphNodeData


class DelegationGraphEdgeData(BaseModel):
    model_config = ConfigDict(extra="forbid")

    delegation_id: str
    objective: str
    status: DelegationStatus
    created_at: datetime
    parent_agent_id: str
    child_agent_id: str


class DelegationGraphEdge(BaseModel):
    model_config = ConfigDict(extra="forbid")

    id: str
    source: str
    target: str
    kind: Literal["delegation"] = "delegation"
    data: DelegationGraphEdgeData


class DelegationGraphResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    nodes: list[DelegationGraphNode]
    edges: list[DelegationGraphEdge]


class RunsPageResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    items: list[WorkflowRun]
    next_cursor: str | None = None


class LogsPageResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    items: list[LogEntry]
    next_cursor: str | None = None


class EventsHistoryPageResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    events: list["ExecutionEvent"]
    next_after_seq: int | None = None
    last_seq: int = 0


class ToolInvocationsPageResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    items: list[ToolInvocation]
    next_cursor: str | None = None


class ToolOutputResponse(BaseModel):
    model_config = ConfigDict(extra="forbid")

    tool_invocation_id: str
    offset: int
    limit: int
    total_bytes: int
    has_more: bool
    content: str


class WorkflowCreatedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    workflow_id: str
    workspace_id: str
    user_id: str
    type: WorkflowType


class WorkflowStartedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    plan_revision: int
    initial_tasks_count: int


class WorkflowCompletedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    duration_ms: float
    total_tasks: int
    artifacts_count: int
    degraded: bool = False
    degraded_reasons: list[str] = Field(default_factory=list)


class WorkflowFailedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    error: str
    failed_task_id: str | None = None


class WorkflowRetryingPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    attempt: int
    next_attempt: int
    delay_ms: int
    error_category: str


class WorkflowCancelledPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    reason: str
    cancelled_by: str


class PlanTaskItem(BaseModel):
    model_config = ConfigDict(extra="forbid")
    id: str
    name: str
    assigned_agent: str
    dependencies: list[str] = Field(default_factory=list)
    required: bool


class PlanCreatedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    revision: int
    tasks: list[PlanTaskItem]


class PlanRevisedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    revision: int
    added_tasks: list[str] = Field(default_factory=list)
    removed_tasks: list[str] = Field(default_factory=list)
    reason: str


class TaskAddedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    name: str
    assigned_agent: str
    dependencies: list[str] = Field(default_factory=list)
    added_in_revision: int


class TaskRemovedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    reason: str


class TaskQueuedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    priority: int = 0


class TaskStartedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    attempt: int
    assigned_agent: str


class TaskCompletedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    duration_ms: float
    output_summary: str = ""


class TaskFailedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    attempt: int
    error: str
    can_retry: bool = False


class TaskRetryingPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    next_attempt: int
    delay_ms: int


class TaskBlockedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    task_id: str
    blocked_by_task_id: str


class AgentStartedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    execution_id: str
    agent_id: str
    objective: str
    model: str


class AgentActivityPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    execution_id: str
    activity_type: str
    description: str


class AgentCompletedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    execution_id: str
    summary: str
    tool_calls: int
    tokens_used: int


class AgentFailedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    execution_id: str
    error: str


class DelegationRequestedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    parent_execution_id: str
    target_agent: str
    reason: str


class DelegationAcceptedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    delegation_id: str
    child_execution_id: str


class DelegationStartedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    delegation_id: str
    child_execution_id: str


class DelegationCompletedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    delegation_id: str
    child_execution_id: str
    summary: str


class DelegationFailedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    delegation_id: str
    error: str


class ToolStartedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    tool_invocation_id: str
    tool_name: str
    sanitized_input: dict[str, Any] = Field(default_factory=dict)


class ToolCompletedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    tool_invocation_id: str
    tool_name: str
    duration_ms: float
    summary: str


class ToolFailedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    tool_invocation_id: str
    tool_name: str
    error: str


class ArtifactCreatedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    artifact_id: str
    type: ArtifactType
    title: str
    surreal_record_id: str | None = None


class FindingSavedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    finding_id: str
    category: str
    claim: str
    confidence: float
    evidence_ids: list[str] = Field(default_factory=list)


class EvidenceSavedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    source_id: str
    url: str
    title: str
    chunk_count: int


class CheckpointCreatedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    thread_id: str
    checkpoint_id: str
    step: int


class ExecutionResumedPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    thread_id: str
    from_step: int


class WorkerRecoveredPayload(BaseModel):
    model_config = ConfigDict(extra="forbid")
    worker_id: str
    run_id: str
    resumed_tasks_count: int


EventPayload = Union[
    WorkflowCreatedPayload,
    WorkflowStartedPayload,
    WorkflowCompletedPayload,
    WorkflowFailedPayload,
    WorkflowCancelledPayload,
    WorkflowRetryingPayload,
    PlanCreatedPayload,
    PlanRevisedPayload,
    TaskAddedPayload,
    TaskRemovedPayload,
    TaskQueuedPayload,
    TaskStartedPayload,
    TaskCompletedPayload,
    TaskFailedPayload,
    TaskRetryingPayload,
    TaskBlockedPayload,
    AgentStartedPayload,
    AgentActivityPayload,
    AgentCompletedPayload,
    AgentFailedPayload,
    DelegationRequestedPayload,
    DelegationAcceptedPayload,
    DelegationStartedPayload,
    DelegationCompletedPayload,
    DelegationFailedPayload,
    ToolStartedPayload,
    ToolCompletedPayload,
    ToolFailedPayload,
    ArtifactCreatedPayload,
    FindingSavedPayload,
    EvidenceSavedPayload,
    CheckpointCreatedPayload,
    ExecutionResumedPayload,
    WorkerRecoveredPayload,
]


class BaseExecutionEvent(BaseModel):
    model_config = ConfigDict(extra="forbid")

    schema_version: str = "1.0"
    event_id: str
    run_id: str
    seq: int
    ts: datetime = Field(default_factory=utc_now)
    task_id: str | None = None
    execution_id: str | None = None
    parent_execution_id: str | None = None
    tool_invocation_id: str | None = None
    correlation_id: str | None = None
    trace_id: str | None = None


class WorkflowCreatedEvent(BaseExecutionEvent):
    type: Literal["workflow.created"] = "workflow.created"
    payload: WorkflowCreatedPayload


class WorkflowStartedEvent(BaseExecutionEvent):
    type: Literal["workflow.started"] = "workflow.started"
    payload: WorkflowStartedPayload


class WorkflowCompletedEvent(BaseExecutionEvent):
    type: Literal["workflow.completed"] = "workflow.completed"
    payload: WorkflowCompletedPayload


class WorkflowFailedEvent(BaseExecutionEvent):
    type: Literal["workflow.failed"] = "workflow.failed"
    payload: WorkflowFailedPayload


class WorkflowRetryingEvent(BaseExecutionEvent):
    type: Literal["workflow.retrying"] = "workflow.retrying"
    payload: WorkflowRetryingPayload


class WorkflowCancelledEvent(BaseExecutionEvent):
    type: Literal["workflow.cancelled"] = "workflow.cancelled"
    payload: WorkflowCancelledPayload


class PlanCreatedEvent(BaseExecutionEvent):
    type: Literal["plan.created"] = "plan.created"
    payload: PlanCreatedPayload


class PlanRevisedEvent(BaseExecutionEvent):
    type: Literal["plan.revised"] = "plan.revised"
    payload: PlanRevisedPayload


class TaskAddedEvent(BaseExecutionEvent):
    type: Literal["task.added"] = "task.added"
    payload: TaskAddedPayload


class TaskRemovedEvent(BaseExecutionEvent):
    type: Literal["task.removed"] = "task.removed"
    payload: TaskRemovedPayload


class TaskQueuedEvent(BaseExecutionEvent):
    type: Literal["task.queued"] = "task.queued"
    payload: TaskQueuedPayload


class TaskStartedEvent(BaseExecutionEvent):
    type: Literal["task.started"] = "task.started"
    payload: TaskStartedPayload


class TaskCompletedEvent(BaseExecutionEvent):
    type: Literal["task.completed"] = "task.completed"
    payload: TaskCompletedPayload


class TaskFailedEvent(BaseExecutionEvent):
    type: Literal["task.failed"] = "task.failed"
    payload: TaskFailedPayload


class TaskRetryingEvent(BaseExecutionEvent):
    type: Literal["task.retrying"] = "task.retrying"
    payload: TaskRetryingPayload


class TaskBlockedEvent(BaseExecutionEvent):
    type: Literal["task.blocked"] = "task.blocked"
    payload: TaskBlockedPayload


class AgentStartedEvent(BaseExecutionEvent):
    type: Literal["agent.started"] = "agent.started"
    payload: AgentStartedPayload


class AgentActivityEvent(BaseExecutionEvent):
    type: Literal["agent.activity"] = "agent.activity"
    payload: AgentActivityPayload


class AgentCompletedEvent(BaseExecutionEvent):
    type: Literal["agent.completed"] = "agent.completed"
    payload: AgentCompletedPayload


class AgentFailedEvent(BaseExecutionEvent):
    type: Literal["agent.failed"] = "agent.failed"
    payload: AgentFailedPayload


class DelegationRequestedEvent(BaseExecutionEvent):
    type: Literal["delegation.requested"] = "delegation.requested"
    payload: DelegationRequestedPayload


class DelegationAcceptedEvent(BaseExecutionEvent):
    type: Literal["delegation.accepted"] = "delegation.accepted"
    payload: DelegationAcceptedPayload


class DelegationStartedEvent(BaseExecutionEvent):
    type: Literal["delegation.started"] = "delegation.started"
    payload: DelegationStartedPayload


class DelegationCompletedEvent(BaseExecutionEvent):
    type: Literal["delegation.completed"] = "delegation.completed"
    payload: DelegationCompletedPayload


class DelegationFailedEvent(BaseExecutionEvent):
    type: Literal["delegation.failed"] = "delegation.failed"
    payload: DelegationFailedPayload


class ToolStartedEvent(BaseExecutionEvent):
    type: Literal["tool.started"] = "tool.started"
    payload: ToolStartedPayload


class ToolCompletedEvent(BaseExecutionEvent):
    type: Literal["tool.completed"] = "tool.completed"
    payload: ToolCompletedPayload


class ToolFailedEvent(BaseExecutionEvent):
    type: Literal["tool.failed"] = "tool.failed"
    payload: ToolFailedPayload


class ArtifactCreatedEvent(BaseExecutionEvent):
    type: Literal["artifact.created"] = "artifact.created"
    payload: ArtifactCreatedPayload


class FindingSavedEvent(BaseExecutionEvent):
    type: Literal["finding.saved"] = "finding.saved"
    payload: FindingSavedPayload


class EvidenceSavedEvent(BaseExecutionEvent):
    type: Literal["evidence.saved"] = "evidence.saved"
    payload: EvidenceSavedPayload


class CheckpointCreatedEvent(BaseExecutionEvent):
    type: Literal["checkpoint.created"] = "checkpoint.created"
    payload: CheckpointCreatedPayload


class ExecutionResumedEvent(BaseExecutionEvent):
    type: Literal["execution.resumed"] = "execution.resumed"
    payload: ExecutionResumedPayload


class WorkerRecoveredEvent(BaseExecutionEvent):
    type: Literal["worker.recovered"] = "worker.recovered"
    payload: WorkerRecoveredPayload


ExecutionEvent = Annotated[
    Union[
        WorkflowCreatedEvent,
        WorkflowStartedEvent,
        WorkflowCompletedEvent,
        WorkflowFailedEvent,
        WorkflowCancelledEvent,
        WorkflowRetryingEvent,
        PlanCreatedEvent,
        PlanRevisedEvent,
        TaskAddedEvent,
        TaskRemovedEvent,
        TaskQueuedEvent,
        TaskStartedEvent,
        TaskCompletedEvent,
        TaskFailedEvent,
        TaskRetryingEvent,
        TaskBlockedEvent,
        AgentStartedEvent,
        AgentActivityEvent,
        AgentCompletedEvent,
        AgentFailedEvent,
        DelegationRequestedEvent,
        DelegationAcceptedEvent,
        DelegationStartedEvent,
        DelegationCompletedEvent,
        DelegationFailedEvent,
        ToolStartedEvent,
        ToolCompletedEvent,
        ToolFailedEvent,
        ArtifactCreatedEvent,
        FindingSavedEvent,
        EvidenceSavedEvent,
        CheckpointCreatedEvent,
        ExecutionResumedEvent,
        WorkerRecoveredEvent,
    ],
    Field(discriminator="type"),
]


EventsHistoryPageResponse.model_rebuild()
