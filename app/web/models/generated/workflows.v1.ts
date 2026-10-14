export type CorrelationId = string | null
export type EventId = string
export type ExecutionId = string | null
export type ParentExecutionId = string | null
export type ActivityType = string
export type Description = string
export type ExecutionId1 = string
export type RunId = string
export type SchemaVersion = string
export type Seq = number
export type TaskId = string | null
export type ToolInvocationId = string | null
export type TraceId = string | null
export type Ts = string
export type Type = 'agent.activity'
export type CorrelationId1 = string | null
export type EventId1 = string
export type ExecutionId2 = string | null
export type ParentExecutionId1 = string | null
export type ExecutionId3 = string
export type Summary = string
export type TokensUsed = number
export type ToolCalls = number
export type RunId1 = string
export type SchemaVersion1 = string
export type Seq1 = number
export type TaskId1 = string | null
export type ToolInvocationId1 = string | null
export type TraceId1 = string | null
export type Ts1 = string
export type Type1 = 'agent.completed'
export type AgentId = string
export type ArtifactIds = string[]
export type ExternalRequests = number
export type ModelCalls = number
export type TokensIn = number
export type TokensOut = number
export type ToolCalls1 = number
export type WallTimeMs = number
export type ExternalRequests1 = number
export type ModelCalls1 = number
export type TokensIn1 = number
export type TokensOut1 = number
export type ToolCalls2 = number
export type WallTimeMs1 = number
export type ChildExecutionIds = string[]
export type CompletedAt = string | null
export type CurrentActivity = string | null
export type DelegationDepth = number
export type Error = string | null
export type Id = string
export type ModelName = string
export type ModelProvider = string
export type Text = string
export type Ts2 = string
export type Notes = ExecutionNote[]
export type Objective = string
export type ParentExecutionId2 = string | null
export type PublicSummary = string
export type Retries = number
export type RunId2 = string
export type StartedAt = string
export type Status = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type TaskId2 = string
export type ToolCallCount = number
export type CorrelationId2 = string | null
export type EventId2 = string
export type ExecutionId4 = string | null
export type ParentExecutionId3 = string | null
export type Error1 = string
export type ExecutionId5 = string
export type RunId3 = string
export type SchemaVersion2 = string
export type Seq2 = number
export type TaskId3 = string | null
export type ToolInvocationId2 = string | null
export type TraceId2 = string | null
export type Ts3 = string
export type Type2 = 'agent.failed'
export type CorrelationId3 = string | null
export type EventId3 = string
export type ExecutionId6 = string | null
export type ParentExecutionId4 = string | null
export type AgentId1 = string
export type ExecutionId7 = string
export type Model = string
export type Objective1 = string
export type RunId4 = string
export type SchemaVersion3 = string
export type Seq3 = number
export type TaskId4 = string | null
export type ToolInvocationId3 = string | null
export type TraceId3 = string | null
export type Ts4 = string
export type Type3 = 'agent.started'
export type AgentExecutionId = string | null
export type Content =
  | {
      [k: string]: unknown
    }
  | string
export type CreatedAt = string
export type Confidence = number
export type RetrievedAt = string
export type Snippet = string
export type SourceUrl = string
export type Title = string
export type EvidenceIds = string[]
export type Id1 = string
export type RunId5 = string
export type Title1 = string
export type Url = string
export type SourceLinks = SourceLink[]
export type SurrealRecordId = string | null
export type TaskId5 = string | null
export type Title2 = string
export type Type4 = 'research_report' | 'finding' | 'evidence' | 'question_set' | 'recommendation' | 'validation_result'
export type Version = number
export type CorrelationId4 = string | null
export type EventId4 = string
export type ExecutionId8 = string | null
export type ParentExecutionId5 = string | null
export type ArtifactId = string
export type SurrealRecordId1 = string | null
export type Title3 = string
export type Type5 = 'research_report' | 'finding' | 'evidence' | 'question_set' | 'recommendation' | 'validation_result'
export type RunId6 = string
export type SchemaVersion4 = string
export type Seq4 = number
export type TaskId6 = string | null
export type ToolInvocationId4 = string | null
export type TraceId4 = string | null
export type Ts5 = string
export type Type6 = 'artifact.created'
export type CorrelationId5 = string | null
export type EventId5 = string
export type ExecutionId9 = string | null
export type ParentExecutionId6 = string | null
export type CheckpointId = string
export type Step = number
export type ThreadId = string
export type RunId7 = string
export type SchemaVersion5 = string
export type Seq5 = number
export type TaskId7 = string | null
export type ToolInvocationId5 = string | null
export type TraceId5 = string | null
export type Ts6 = string
export type Type7 = 'checkpoint.created'
export type ChildAgentId = string
export type ChildExecutionId = string
export type CompletedAt1 = string | null
export type CreatedAt1 = string
export type DelegationTaskId = string
export type Depth = number
export type Error2 = string | null
export type Id2 = string
export type InputArtifactIds = string[]
export type Objective2 = string
export type OutputArtifactIds = string[]
export type ParentAgentId = string
export type ParentExecutionId7 = string
export type Reason = string
export type RunId8 = string
export type Status1 = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type CorrelationId6 = string | null
export type EventId6 = string
export type ExecutionId10 = string | null
export type ParentExecutionId8 = string | null
export type ChildExecutionId1 = string
export type DelegationId = string
export type RunId9 = string
export type SchemaVersion6 = string
export type Seq6 = number
export type TaskId8 = string | null
export type ToolInvocationId6 = string | null
export type TraceId6 = string | null
export type Ts7 = string
export type Type8 = 'delegation.accepted'
export type CorrelationId7 = string | null
export type EventId7 = string
export type ExecutionId11 = string | null
export type ParentExecutionId9 = string | null
export type ChildExecutionId2 = string
export type DelegationId1 = string
export type Summary1 = string
export type RunId10 = string
export type SchemaVersion7 = string
export type Seq7 = number
export type TaskId9 = string | null
export type ToolInvocationId7 = string | null
export type TraceId7 = string | null
export type Ts8 = string
export type Type9 = 'delegation.completed'
export type CorrelationId8 = string | null
export type EventId8 = string
export type ExecutionId12 = string | null
export type ParentExecutionId10 = string | null
export type DelegationId2 = string
export type Error3 = string
export type RunId11 = string
export type SchemaVersion8 = string
export type Seq8 = number
export type TaskId10 = string | null
export type ToolInvocationId8 = string | null
export type TraceId8 = string | null
export type Ts9 = string
export type Type10 = 'delegation.failed'
export type ChildAgentId1 = string
export type CreatedAt2 = string
export type DelegationId3 = string
export type Objective3 = string
export type ParentAgentId1 = string
export type Status2 = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type Id3 = string
export type Kind = 'delegation'
export type Source = string
export type Target = string
export type AgentId2 = string
export type Status3 = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type TaskId11 = string
export type Id4 = string
export type Label = string
export type Type11 = string
export type Edges = DelegationGraphEdge[]
export type Nodes = DelegationGraphNode[]
export type CorrelationId9 = string | null
export type EventId9 = string
export type ExecutionId13 = string | null
export type ParentExecutionId11 = string | null
export type ParentExecutionId12 = string
export type Reason1 = string
export type TargetAgent = string
export type RunId12 = string
export type SchemaVersion9 = string
export type Seq9 = number
export type TaskId12 = string | null
export type ToolInvocationId9 = string | null
export type TraceId9 = string | null
export type Ts10 = string
export type Type12 = 'delegation.requested'
export type CorrelationId10 = string | null
export type EventId10 = string
export type ExecutionId14 = string | null
export type ParentExecutionId13 = string | null
export type ChildExecutionId3 = string
export type DelegationId4 = string
export type RunId13 = string
export type SchemaVersion10 = string
export type Seq10 = number
export type TaskId13 = string | null
export type ToolInvocationId10 = string | null
export type TraceId10 = string | null
export type Ts11 = string
export type Type13 = 'delegation.started'
export type CorrelationId11 = string | null
export type EventId11 = string
export type ExecutionId15 = string | null
export type ParentExecutionId14 = string | null
export type Type14 = 'static' | 'dynamic'
export type UserId = string
export type WorkflowId = string
export type WorkspaceId = string
export type RunId14 = string
export type SchemaVersion11 = string
export type Seq11 = number
export type TaskId14 = string | null
export type ToolInvocationId11 = string | null
export type TraceId11 = string | null
export type Ts12 = string
export type Type15 = 'workflow.created'
export type CorrelationId12 = string | null
export type EventId12 = string
export type ExecutionId16 = string | null
export type ParentExecutionId15 = string | null
export type InitialTasksCount = number
export type PlanRevision = number
export type RunId15 = string
export type SchemaVersion12 = string
export type Seq12 = number
export type TaskId15 = string | null
export type ToolInvocationId12 = string | null
export type TraceId12 = string | null
export type Ts13 = string
export type Type16 = 'workflow.started'
export type CorrelationId13 = string | null
export type EventId13 = string
export type ExecutionId17 = string | null
export type ParentExecutionId16 = string | null
export type ArtifactsCount = number
export type DurationMs = number
export type TotalTasks = number
export type RunId16 = string
export type SchemaVersion13 = string
export type Seq13 = number
export type TaskId16 = string | null
export type ToolInvocationId13 = string | null
export type TraceId13 = string | null
export type Ts14 = string
export type Type17 = 'workflow.completed'
export type CorrelationId14 = string | null
export type EventId14 = string
export type ExecutionId18 = string | null
export type ParentExecutionId17 = string | null
export type Error4 = string
export type FailedTaskId = string | null
export type RunId17 = string
export type SchemaVersion14 = string
export type Seq14 = number
export type TaskId17 = string | null
export type ToolInvocationId14 = string | null
export type TraceId14 = string | null
export type Ts15 = string
export type Type18 = 'workflow.failed'
export type CorrelationId15 = string | null
export type EventId15 = string
export type ExecutionId19 = string | null
export type ParentExecutionId18 = string | null
export type CancelledBy = string
export type Reason2 = string
export type RunId18 = string
export type SchemaVersion15 = string
export type Seq15 = number
export type TaskId18 = string | null
export type ToolInvocationId15 = string | null
export type TraceId15 = string | null
export type Ts16 = string
export type Type19 = 'workflow.cancelled'
export type CorrelationId16 = string | null
export type EventId16 = string
export type ExecutionId20 = string | null
export type ParentExecutionId19 = string | null
export type Attempt = number
export type DelayMs = number
export type ErrorCategory = string
export type NextAttempt = number
export type RunId19 = string
export type SchemaVersion16 = string
export type Seq16 = number
export type TaskId19 = string | null
export type ToolInvocationId16 = string | null
export type TraceId16 = string | null
export type Ts17 = string
export type Type20 = 'workflow.retrying'
export type CorrelationId17 = string | null
export type EventId17 = string
export type ExecutionId21 = string | null
export type ParentExecutionId20 = string | null
export type Revision = number
export type AssignedAgent = string
export type Dependencies = string[]
export type Id5 = string
export type Name = string
export type Required = boolean
export type Tasks = PlanTaskItem[]
export type RunId20 = string
export type SchemaVersion17 = string
export type Seq17 = number
export type TaskId20 = string | null
export type ToolInvocationId17 = string | null
export type TraceId17 = string | null
export type Ts18 = string
export type Type21 = 'plan.created'
export type CorrelationId18 = string | null
export type EventId18 = string
export type ExecutionId22 = string | null
export type ParentExecutionId21 = string | null
export type AddedTasks = string[]
export type Reason3 = string
export type RemovedTasks = string[]
export type Revision1 = number
export type RunId21 = string
export type SchemaVersion18 = string
export type Seq18 = number
export type TaskId21 = string | null
export type ToolInvocationId18 = string | null
export type TraceId18 = string | null
export type Ts19 = string
export type Type22 = 'plan.revised'
export type CorrelationId19 = string | null
export type EventId19 = string
export type ExecutionId23 = string | null
export type ParentExecutionId22 = string | null
export type AddedInRevision = number
export type AssignedAgent1 = string
export type Dependencies1 = string[]
export type Name1 = string
export type TaskId22 = string
export type RunId22 = string
export type SchemaVersion19 = string
export type Seq19 = number
export type TaskId23 = string | null
export type ToolInvocationId19 = string | null
export type TraceId19 = string | null
export type Ts20 = string
export type Type23 = 'task.added'
export type CorrelationId20 = string | null
export type EventId20 = string
export type ExecutionId24 = string | null
export type ParentExecutionId23 = string | null
export type Reason4 = string
export type TaskId24 = string
export type RunId23 = string
export type SchemaVersion20 = string
export type Seq20 = number
export type TaskId25 = string | null
export type ToolInvocationId20 = string | null
export type TraceId20 = string | null
export type Ts21 = string
export type Type24 = 'task.removed'
export type CorrelationId21 = string | null
export type EventId21 = string
export type ExecutionId25 = string | null
export type ParentExecutionId24 = string | null
export type Priority = number
export type TaskId26 = string
export type RunId24 = string
export type SchemaVersion21 = string
export type Seq21 = number
export type TaskId27 = string | null
export type ToolInvocationId21 = string | null
export type TraceId21 = string | null
export type Ts22 = string
export type Type25 = 'task.queued'
export type CorrelationId22 = string | null
export type EventId22 = string
export type ExecutionId26 = string | null
export type ParentExecutionId25 = string | null
export type AssignedAgent2 = string
export type Attempt1 = number
export type TaskId28 = string
export type RunId25 = string
export type SchemaVersion22 = string
export type Seq22 = number
export type TaskId29 = string | null
export type ToolInvocationId22 = string | null
export type TraceId22 = string | null
export type Ts23 = string
export type Type26 = 'task.started'
export type CorrelationId23 = string | null
export type EventId23 = string
export type ExecutionId27 = string | null
export type ParentExecutionId26 = string | null
export type DurationMs1 = number
export type OutputSummary = string
export type TaskId30 = string
export type RunId26 = string
export type SchemaVersion23 = string
export type Seq23 = number
export type TaskId31 = string | null
export type ToolInvocationId23 = string | null
export type TraceId23 = string | null
export type Ts24 = string
export type Type27 = 'task.completed'
export type CorrelationId24 = string | null
export type EventId24 = string
export type ExecutionId28 = string | null
export type ParentExecutionId27 = string | null
export type Attempt2 = number
export type CanRetry = boolean
export type Error5 = string
export type TaskId32 = string
export type RunId27 = string
export type SchemaVersion24 = string
export type Seq24 = number
export type TaskId33 = string | null
export type ToolInvocationId24 = string | null
export type TraceId24 = string | null
export type Ts25 = string
export type Type28 = 'task.failed'
export type CorrelationId25 = string | null
export type EventId25 = string
export type ExecutionId29 = string | null
export type ParentExecutionId28 = string | null
export type DelayMs1 = number
export type NextAttempt1 = number
export type TaskId34 = string
export type RunId28 = string
export type SchemaVersion25 = string
export type Seq25 = number
export type TaskId35 = string | null
export type ToolInvocationId25 = string | null
export type TraceId25 = string | null
export type Ts26 = string
export type Type29 = 'task.retrying'
export type CorrelationId26 = string | null
export type EventId26 = string
export type ExecutionId30 = string | null
export type ParentExecutionId29 = string | null
export type BlockedByTaskId = string
export type TaskId36 = string
export type RunId29 = string
export type SchemaVersion26 = string
export type Seq26 = number
export type TaskId37 = string | null
export type ToolInvocationId26 = string | null
export type TraceId26 = string | null
export type Ts27 = string
export type Type30 = 'task.blocked'
export type CorrelationId27 = string | null
export type EventId27 = string
export type ExecutionId31 = string | null
export type ParentExecutionId30 = string | null
export type ToolInvocationId27 = string
export type ToolName = string
export type RunId30 = string
export type SchemaVersion27 = string
export type Seq27 = number
export type TaskId38 = string | null
export type ToolInvocationId28 = string | null
export type TraceId27 = string | null
export type Ts28 = string
export type Type31 = 'tool.started'
export type CorrelationId28 = string | null
export type EventId28 = string
export type ExecutionId32 = string | null
export type ParentExecutionId31 = string | null
export type DurationMs2 = number
export type Summary2 = string
export type ToolInvocationId29 = string
export type ToolName1 = string
export type RunId31 = string
export type SchemaVersion28 = string
export type Seq28 = number
export type TaskId39 = string | null
export type ToolInvocationId30 = string | null
export type TraceId28 = string | null
export type Ts29 = string
export type Type32 = 'tool.completed'
export type CorrelationId29 = string | null
export type EventId29 = string
export type ExecutionId33 = string | null
export type ParentExecutionId32 = string | null
export type Error6 = string
export type ToolInvocationId31 = string
export type ToolName2 = string
export type RunId32 = string
export type SchemaVersion29 = string
export type Seq29 = number
export type TaskId40 = string | null
export type ToolInvocationId32 = string | null
export type TraceId29 = string | null
export type Ts30 = string
export type Type33 = 'tool.failed'
export type CorrelationId30 = string | null
export type EventId30 = string
export type ExecutionId34 = string | null
export type ParentExecutionId33 = string | null
export type Category = string
export type Claim = string
export type Confidence1 = number
export type FindingId = string
export type RunId33 = string
export type SchemaVersion30 = string
export type Seq30 = number
export type TaskId41 = string | null
export type ToolInvocationId33 = string | null
export type TraceId30 = string | null
export type Ts31 = string
export type Type34 = 'finding.saved'
export type CorrelationId31 = string | null
export type EventId31 = string
export type ExecutionId35 = string | null
export type ParentExecutionId34 = string | null
export type ChunkCount = number
export type SourceId = string
export type Title4 = string
export type Url1 = string
export type RunId34 = string
export type SchemaVersion31 = string
export type Seq31 = number
export type TaskId42 = string | null
export type ToolInvocationId34 = string | null
export type TraceId31 = string | null
export type Ts32 = string
export type Type35 = 'evidence.saved'
export type CorrelationId32 = string | null
export type EventId32 = string
export type ExecutionId36 = string | null
export type ParentExecutionId35 = string | null
export type FromStep = number
export type ThreadId1 = string
export type RunId35 = string
export type SchemaVersion32 = string
export type Seq32 = number
export type TaskId43 = string | null
export type ToolInvocationId35 = string | null
export type TraceId32 = string | null
export type Ts33 = string
export type Type36 = 'execution.resumed'
export type CorrelationId33 = string | null
export type EventId33 = string
export type ExecutionId37 = string | null
export type ParentExecutionId36 = string | null
export type ResumedTasksCount = number
export type RunId36 = string
export type WorkerId = string
export type RunId37 = string
export type SchemaVersion33 = string
export type Seq33 = number
export type TaskId44 = string | null
export type ToolInvocationId36 = string | null
export type TraceId33 = string | null
export type Ts34 = string
export type Type37 = 'worker.recovered'
export type Events = (
  | WorkflowCreatedEvent
  | WorkflowStartedEvent
  | WorkflowCompletedEvent
  | WorkflowFailedEvent
  | WorkflowCancelledEvent
  | WorkflowRetryingEvent
  | PlanCreatedEvent
  | PlanRevisedEvent
  | TaskAddedEvent
  | TaskRemovedEvent
  | TaskQueuedEvent
  | TaskStartedEvent
  | TaskCompletedEvent
  | TaskFailedEvent
  | TaskRetryingEvent
  | TaskBlockedEvent
  | AgentStartedEvent
  | AgentActivityEvent
  | AgentCompletedEvent
  | AgentFailedEvent
  | DelegationRequestedEvent
  | DelegationAcceptedEvent
  | DelegationStartedEvent
  | DelegationCompletedEvent
  | DelegationFailedEvent
  | ToolStartedEvent
  | ToolCompletedEvent
  | ToolFailedEvent
  | ArtifactCreatedEvent
  | FindingSavedEvent
  | EvidenceSavedEvent
  | CheckpointCreatedEvent
  | ExecutionResumedEvent
  | WorkerRecoveredEvent
)[]
export type LastSeq = number
export type NextAfterSeq = number | null
export type Component = string
export type DurationMs3 = number | null
export type ErrorCategory1 =
  ('network' | 'rate_limit' | 'budget_exceeded' | 'llm_error' | 'validation_error' | 'internal_error') | null
export type ExecutionId38 = string | null
export type Id6 = string
export type Message = string
export type RunId38 = string
export type Seq34 = number | null
export type Severity = 'debug' | 'info' | 'warn' | 'error'
export type TaskId45 = string | null
export type ToolInvocationId37 = string | null
export type TraceId34 = string | null
export type Ts35 = string
export type Items = LogEntry[]
export type NextCursor = string | null
export type CompletedAt2 = string | null
export type CreatedAt3 = string
export type CurrentPhase = string
export type DurationMs4 = number | null
export type ErrorMessage = string | null
export type Id7 = string
export type LastSeq1 = number
export type PlanRevision1 = number
export type Blocked = number
export type Cancelled = number
export type Completed = number
export type Estimate = boolean
export type Failed = number
export type Pending = number
export type Percentage = number | null
export type PhaseCount = number | null
export type PhaseIndex = number | null
export type Queued = number
export type Retrying = number
export type Running = number
export type Skipped = number
export type TotalTasks1 = number
export type StartedAt1 = string | null
export type Status4 = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type Type38 = 'static' | 'dynamic'
export type UpdatedAt = string
export type UserId1 = string
export type WorkflowId1 = string
export type WorkflowName = string
export type WorkspaceId1 = string
export type Items1 = WorkflowRun[]
export type NextCursor1 = string | null
export type Id8 = string
export type Kind1 = 'dependency'
export type Source1 = string
export type Target1 = string
export type AddedInRevision1 = number
export type AssignedAgent3 = string
export type Attempt3 = number
export type ExecutionIds = string[]
export type Group = string
export type Required1 = boolean
export type Status5 =
  'pending' | 'queued' | 'running' | 'completed' | 'failed' | 'retrying' | 'blocked' | 'cancelled' | 'skipped'
export type TaskType = string
export type Id9 = string
export type Label1 = string
export type Type39 = string
export type Edges1 = TaskGraphEdge[]
export type Nodes1 = TaskGraphNode[]
export type CompletedAt3 = string | null
export type DurationMs5 = number
export type Error7 = string | null
export type ExecutionId39 = string
export type Id10 = string
export type OutputRef = string | null
export type OutputSummary1 = string
export type StartedAt2 = string
export type Status6 = 'pending' | 'running' | 'completed' | 'failed'
export type ToolName3 = string
export type Items2 = ToolInvocation[]
export type NextCursor2 = string | null
export type Content1 = string
export type HasMore = boolean
export type Limit = number
export type Offset = number
export type ToolInvocationId38 = string
export type TotalBytes = number
export type DefaultMaxRetries = number
export type Description1 = string
export type Id11 = string
export type Name2 = string
export type SupportedAgents = string[]
export type Type40 = 'static' | 'dynamic'
export type Version1 = string
export type AddedInRevision2 = number
export type AssignedAgent4 = string
export type Attempt4 = number
export type CreatedAt4 = string
export type Dependencies2 = string[]
export type Error8 = string | null
export type ExecutionIds1 = string[]
export type Id12 = string
export type Name3 = string
export type Required2 = boolean
export type RunId39 = string
export type Status7 =
  'pending' | 'queued' | 'running' | 'completed' | 'failed' | 'retrying' | 'blocked' | 'cancelled' | 'skipped'
export type TaskType1 = string
export type UpdatedAt1 = string

export interface MensetsuWorkflowsV1Schemas {
  AgentActivityEvent?: AgentActivityEvent
  AgentActivityPayload?: AgentActivityPayload
  AgentCompletedEvent?: AgentCompletedEvent
  AgentCompletedPayload?: AgentCompletedPayload
  AgentExecution?: AgentExecution
  AgentFailedEvent?: AgentFailedEvent
  AgentFailedPayload?: AgentFailedPayload
  AgentStartedEvent?: AgentStartedEvent
  AgentStartedPayload?: AgentStartedPayload
  Artifact?: Artifact
  ArtifactCreatedEvent?: ArtifactCreatedEvent
  ArtifactCreatedPayload?: ArtifactCreatedPayload
  CheckpointCreatedEvent?: CheckpointCreatedEvent
  CheckpointCreatedPayload?: CheckpointCreatedPayload
  Delegation?: Delegation
  DelegationAcceptedEvent?: DelegationAcceptedEvent
  DelegationAcceptedPayload?: DelegationAcceptedPayload
  DelegationCompletedEvent?: DelegationCompletedEvent
  DelegationCompletedPayload?: DelegationCompletedPayload
  DelegationFailedEvent?: DelegationFailedEvent
  DelegationFailedPayload?: DelegationFailedPayload
  DelegationGraphEdge?: DelegationGraphEdge
  DelegationGraphEdgeData?: DelegationGraphEdgeData
  DelegationGraphNode?: DelegationGraphNode
  DelegationGraphNodeData?: DelegationGraphNodeData
  DelegationGraphResponse?: DelegationGraphResponse
  DelegationRequestedEvent?: DelegationRequestedEvent
  DelegationRequestedPayload?: DelegationRequestedPayload
  DelegationStartedEvent?: DelegationStartedEvent
  DelegationStartedPayload?: DelegationStartedPayload
  EventsHistoryPageResponse?: EventsHistoryPageResponse
  EvidenceProvenance?: EvidenceProvenance
  EvidenceSavedEvent?: EvidenceSavedEvent
  EvidenceSavedPayload?: EvidenceSavedPayload
  ExecutionBudgets?: ExecutionBudgets
  ExecutionBudgetsUsed?: ExecutionBudgetsUsed
  ExecutionNote?: ExecutionNote
  ExecutionResumedEvent?: ExecutionResumedEvent
  ExecutionResumedPayload?: ExecutionResumedPayload
  FindingSavedEvent?: FindingSavedEvent
  FindingSavedPayload?: FindingSavedPayload
  LogEntry?: LogEntry
  LogsPageResponse?: LogsPageResponse
  PlanCreatedEvent?: PlanCreatedEvent
  PlanCreatedPayload?: PlanCreatedPayload
  PlanRevisedEvent?: PlanRevisedEvent
  PlanRevisedPayload?: PlanRevisedPayload
  PlanTaskItem?: PlanTaskItem
  RunsPageResponse?: RunsPageResponse
  SourceLink?: SourceLink
  TaskAddedEvent?: TaskAddedEvent
  TaskAddedPayload?: TaskAddedPayload
  TaskBlockedEvent?: TaskBlockedEvent
  TaskBlockedPayload?: TaskBlockedPayload
  TaskCompletedEvent?: TaskCompletedEvent
  TaskCompletedPayload?: TaskCompletedPayload
  TaskFailedEvent?: TaskFailedEvent
  TaskFailedPayload?: TaskFailedPayload
  TaskGraphEdge?: TaskGraphEdge
  TaskGraphNode?: TaskGraphNode
  TaskGraphNodeData?: TaskGraphNodeData
  TaskGraphResponse?: TaskGraphResponse
  TaskQueuedEvent?: TaskQueuedEvent
  TaskQueuedPayload?: TaskQueuedPayload
  TaskRemovedEvent?: TaskRemovedEvent
  TaskRemovedPayload?: TaskRemovedPayload
  TaskRetryingEvent?: TaskRetryingEvent
  TaskRetryingPayload?: TaskRetryingPayload
  TaskStartedEvent?: TaskStartedEvent
  TaskStartedPayload?: TaskStartedPayload
  ToolCompletedEvent?: ToolCompletedEvent
  ToolCompletedPayload?: ToolCompletedPayload
  ToolFailedEvent?: ToolFailedEvent
  ToolFailedPayload?: ToolFailedPayload
  ToolInvocation?: ToolInvocation
  ToolInvocationsPageResponse?: ToolInvocationsPageResponse
  ToolOutputResponse?: ToolOutputResponse
  ToolStartedEvent?: ToolStartedEvent
  ToolStartedPayload?: ToolStartedPayload
  WorkerRecoveredEvent?: WorkerRecoveredEvent
  WorkerRecoveredPayload?: WorkerRecoveredPayload
  WorkflowCancelledEvent?: WorkflowCancelledEvent
  WorkflowCancelledPayload?: WorkflowCancelledPayload
  WorkflowCompletedEvent?: WorkflowCompletedEvent
  WorkflowCompletedPayload?: WorkflowCompletedPayload
  WorkflowCreatedEvent?: WorkflowCreatedEvent
  WorkflowCreatedPayload?: WorkflowCreatedPayload
  WorkflowDefinition?: WorkflowDefinition
  WorkflowFailedEvent?: WorkflowFailedEvent
  WorkflowFailedPayload?: WorkflowFailedPayload
  WorkflowProgress?: WorkflowProgress
  WorkflowRetryingEvent?: WorkflowRetryingEvent
  WorkflowRetryingPayload?: WorkflowRetryingPayload
  WorkflowRun?: WorkflowRun
  WorkflowStartedEvent?: WorkflowStartedEvent
  WorkflowStartedPayload?: WorkflowStartedPayload
  WorkflowTask?: WorkflowTask
  [k: string]: unknown
}

export interface AgentActivityEvent {
  correlation_id?: CorrelationId
  event_id: EventId
  execution_id?: ExecutionId
  parent_execution_id?: ParentExecutionId
  payload: AgentActivityPayload
  run_id: RunId
  schema_version?: SchemaVersion
  seq: Seq
  task_id?: TaskId
  tool_invocation_id?: ToolInvocationId
  trace_id?: TraceId
  ts?: Ts
  type?: Type
}

export interface AgentActivityPayload {
  activity_type: ActivityType
  description: Description
  execution_id: ExecutionId1
}

export interface AgentCompletedEvent {
  correlation_id?: CorrelationId1
  event_id: EventId1
  execution_id?: ExecutionId2
  parent_execution_id?: ParentExecutionId1
  payload: AgentCompletedPayload
  run_id: RunId1
  schema_version?: SchemaVersion1
  seq: Seq1
  task_id?: TaskId1
  tool_invocation_id?: ToolInvocationId1
  trace_id?: TraceId1
  ts?: Ts1
  type?: Type1
}

export interface AgentCompletedPayload {
  execution_id: ExecutionId3
  summary: Summary
  tokens_used: TokensUsed
  tool_calls: ToolCalls
}

export interface AgentExecution {
  agent_id: AgentId
  artifact_ids?: ArtifactIds
  budgets?: ExecutionBudgets
  budgets_used?: ExecutionBudgetsUsed
  child_execution_ids?: ChildExecutionIds
  completed_at?: CompletedAt
  current_activity?: CurrentActivity
  delegation_depth?: DelegationDepth
  error?: Error
  id: Id
  model_name?: ModelName
  model_provider?: ModelProvider
  notes?: Notes
  objective?: Objective
  parent_execution_id?: ParentExecutionId2
  public_summary?: PublicSummary
  retries?: Retries
  run_id: RunId2
  started_at?: StartedAt
  status?: Status
  task_id: TaskId2
  tool_call_count?: ToolCallCount
}

export interface ExecutionBudgets {
  external_requests?: ExternalRequests
  model_calls?: ModelCalls
  tokens_in?: TokensIn
  tokens_out?: TokensOut
  tool_calls?: ToolCalls1
  wall_time_ms?: WallTimeMs
}

export interface ExecutionBudgetsUsed {
  external_requests?: ExternalRequests1
  model_calls?: ModelCalls1
  tokens_in?: TokensIn1
  tokens_out?: TokensOut1
  tool_calls?: ToolCalls2
  wall_time_ms?: WallTimeMs1
}

export interface ExecutionNote {
  text: Text
  ts?: Ts2
}

export interface AgentFailedEvent {
  correlation_id?: CorrelationId2
  event_id: EventId2
  execution_id?: ExecutionId4
  parent_execution_id?: ParentExecutionId3
  payload: AgentFailedPayload
  run_id: RunId3
  schema_version?: SchemaVersion2
  seq: Seq2
  task_id?: TaskId3
  tool_invocation_id?: ToolInvocationId2
  trace_id?: TraceId2
  ts?: Ts3
  type?: Type2
}

export interface AgentFailedPayload {
  error: Error1
  execution_id: ExecutionId5
}

export interface AgentStartedEvent {
  correlation_id?: CorrelationId3
  event_id: EventId3
  execution_id?: ExecutionId6
  parent_execution_id?: ParentExecutionId4
  payload: AgentStartedPayload
  run_id: RunId4
  schema_version?: SchemaVersion3
  seq: Seq3
  task_id?: TaskId4
  tool_invocation_id?: ToolInvocationId3
  trace_id?: TraceId3
  ts?: Ts4
  type?: Type3
}

export interface AgentStartedPayload {
  agent_id: AgentId1
  execution_id: ExecutionId7
  model: Model
  objective: Objective1
}

export interface Artifact {
  agent_execution_id?: AgentExecutionId
  content?: Content
  created_at?: CreatedAt
  evidence?: EvidenceProvenance | null
  evidence_ids?: EvidenceIds
  id: Id1
  run_id: RunId5
  source_links?: SourceLinks
  surreal_record_id?: SurrealRecordId
  task_id?: TaskId5
  title: Title2
  type: Type4
  version?: Version
}

export interface EvidenceProvenance {
  confidence: Confidence
  retrieved_at?: RetrievedAt
  snippet: Snippet
  source_url: SourceUrl
  title: Title
}

export interface SourceLink {
  title: Title1
  url: Url
}

export interface ArtifactCreatedEvent {
  correlation_id?: CorrelationId4
  event_id: EventId4
  execution_id?: ExecutionId8
  parent_execution_id?: ParentExecutionId5
  payload: ArtifactCreatedPayload
  run_id: RunId6
  schema_version?: SchemaVersion4
  seq: Seq4
  task_id?: TaskId6
  tool_invocation_id?: ToolInvocationId4
  trace_id?: TraceId4
  ts?: Ts5
  type?: Type6
}

export interface ArtifactCreatedPayload {
  artifact_id: ArtifactId
  surreal_record_id?: SurrealRecordId1
  title: Title3
  type: Type5
}

export interface CheckpointCreatedEvent {
  correlation_id?: CorrelationId5
  event_id: EventId5
  execution_id?: ExecutionId9
  parent_execution_id?: ParentExecutionId6
  payload: CheckpointCreatedPayload
  run_id: RunId7
  schema_version?: SchemaVersion5
  seq: Seq5
  task_id?: TaskId7
  tool_invocation_id?: ToolInvocationId5
  trace_id?: TraceId5
  ts?: Ts6
  type?: Type7
}

export interface CheckpointCreatedPayload {
  checkpoint_id: CheckpointId
  step: Step
  thread_id: ThreadId
}

export interface Delegation {
  child_agent_id: ChildAgentId
  child_execution_id: ChildExecutionId
  completed_at?: CompletedAt1
  created_at?: CreatedAt1
  delegation_task_id: DelegationTaskId
  depth?: Depth
  error?: Error2
  id: Id2
  input_artifact_ids?: InputArtifactIds
  objective: Objective2
  output_artifact_ids?: OutputArtifactIds
  parent_agent_id: ParentAgentId
  parent_execution_id: ParentExecutionId7
  reason: Reason
  run_id: RunId8
  status?: Status1
}

export interface DelegationAcceptedEvent {
  correlation_id?: CorrelationId6
  event_id: EventId6
  execution_id?: ExecutionId10
  parent_execution_id?: ParentExecutionId8
  payload: DelegationAcceptedPayload
  run_id: RunId9
  schema_version?: SchemaVersion6
  seq: Seq6
  task_id?: TaskId8
  tool_invocation_id?: ToolInvocationId6
  trace_id?: TraceId6
  ts?: Ts7
  type?: Type8
}

export interface DelegationAcceptedPayload {
  child_execution_id: ChildExecutionId1
  delegation_id: DelegationId
}

export interface DelegationCompletedEvent {
  correlation_id?: CorrelationId7
  event_id: EventId7
  execution_id?: ExecutionId11
  parent_execution_id?: ParentExecutionId9
  payload: DelegationCompletedPayload
  run_id: RunId10
  schema_version?: SchemaVersion7
  seq: Seq7
  task_id?: TaskId9
  tool_invocation_id?: ToolInvocationId7
  trace_id?: TraceId7
  ts?: Ts8
  type?: Type9
}

export interface DelegationCompletedPayload {
  child_execution_id: ChildExecutionId2
  delegation_id: DelegationId1
  summary: Summary1
}

export interface DelegationFailedEvent {
  correlation_id?: CorrelationId8
  event_id: EventId8
  execution_id?: ExecutionId12
  parent_execution_id?: ParentExecutionId10
  payload: DelegationFailedPayload
  run_id: RunId11
  schema_version?: SchemaVersion8
  seq: Seq8
  task_id?: TaskId10
  tool_invocation_id?: ToolInvocationId8
  trace_id?: TraceId8
  ts?: Ts9
  type?: Type10
}

export interface DelegationFailedPayload {
  delegation_id: DelegationId2
  error: Error3
}

export interface DelegationGraphEdge {
  data: DelegationGraphEdgeData
  id: Id3
  kind?: Kind
  source: Source
  target: Target
}

export interface DelegationGraphEdgeData {
  child_agent_id: ChildAgentId1
  created_at: CreatedAt2
  delegation_id: DelegationId3
  objective: Objective3
  parent_agent_id: ParentAgentId1
  status: Status2
}

export interface DelegationGraphNode {
  data: DelegationGraphNodeData
  id: Id4
  label: Label
  type?: Type11
}

export interface DelegationGraphNodeData {
  agent_id: AgentId2
  status: Status3
  task_id: TaskId11
}

export interface DelegationGraphResponse {
  edges: Edges
  nodes: Nodes
}

export interface DelegationRequestedEvent {
  correlation_id?: CorrelationId9
  event_id: EventId9
  execution_id?: ExecutionId13
  parent_execution_id?: ParentExecutionId11
  payload: DelegationRequestedPayload
  run_id: RunId12
  schema_version?: SchemaVersion9
  seq: Seq9
  task_id?: TaskId12
  tool_invocation_id?: ToolInvocationId9
  trace_id?: TraceId9
  ts?: Ts10
  type?: Type12
}

export interface DelegationRequestedPayload {
  parent_execution_id: ParentExecutionId12
  reason: Reason1
  target_agent: TargetAgent
}

export interface DelegationStartedEvent {
  correlation_id?: CorrelationId10
  event_id: EventId10
  execution_id?: ExecutionId14
  parent_execution_id?: ParentExecutionId13
  payload: DelegationStartedPayload
  run_id: RunId13
  schema_version?: SchemaVersion10
  seq: Seq10
  task_id?: TaskId13
  tool_invocation_id?: ToolInvocationId10
  trace_id?: TraceId10
  ts?: Ts11
  type?: Type13
}

export interface DelegationStartedPayload {
  child_execution_id: ChildExecutionId3
  delegation_id: DelegationId4
}

export interface EventsHistoryPageResponse {
  events: Events
  last_seq?: LastSeq
  next_after_seq?: NextAfterSeq
}

export interface WorkflowCreatedEvent {
  correlation_id?: CorrelationId11
  event_id: EventId11
  execution_id?: ExecutionId15
  parent_execution_id?: ParentExecutionId14
  payload: WorkflowCreatedPayload
  run_id: RunId14
  schema_version?: SchemaVersion11
  seq: Seq11
  task_id?: TaskId14
  tool_invocation_id?: ToolInvocationId11
  trace_id?: TraceId11
  ts?: Ts12
  type?: Type15
}

export interface WorkflowCreatedPayload {
  type: Type14
  user_id: UserId
  workflow_id: WorkflowId
  workspace_id: WorkspaceId
}

export interface WorkflowStartedEvent {
  correlation_id?: CorrelationId12
  event_id: EventId12
  execution_id?: ExecutionId16
  parent_execution_id?: ParentExecutionId15
  payload: WorkflowStartedPayload
  run_id: RunId15
  schema_version?: SchemaVersion12
  seq: Seq12
  task_id?: TaskId15
  tool_invocation_id?: ToolInvocationId12
  trace_id?: TraceId12
  ts?: Ts13
  type?: Type16
}

export interface WorkflowStartedPayload {
  initial_tasks_count: InitialTasksCount
  plan_revision: PlanRevision
}

export interface WorkflowCompletedEvent {
  correlation_id?: CorrelationId13
  event_id: EventId13
  execution_id?: ExecutionId17
  parent_execution_id?: ParentExecutionId16
  payload: WorkflowCompletedPayload
  run_id: RunId16
  schema_version?: SchemaVersion13
  seq: Seq13
  task_id?: TaskId16
  tool_invocation_id?: ToolInvocationId13
  trace_id?: TraceId13
  ts?: Ts14
  type?: Type17
}

export interface WorkflowCompletedPayload {
  artifacts_count: ArtifactsCount
  duration_ms: DurationMs
  total_tasks: TotalTasks
}

export interface WorkflowFailedEvent {
  correlation_id?: CorrelationId14
  event_id: EventId14
  execution_id?: ExecutionId18
  parent_execution_id?: ParentExecutionId17
  payload: WorkflowFailedPayload
  run_id: RunId17
  schema_version?: SchemaVersion14
  seq: Seq14
  task_id?: TaskId17
  tool_invocation_id?: ToolInvocationId14
  trace_id?: TraceId14
  ts?: Ts15
  type?: Type18
}

export interface WorkflowFailedPayload {
  error: Error4
  failed_task_id?: FailedTaskId
}

export interface WorkflowCancelledEvent {
  correlation_id?: CorrelationId15
  event_id: EventId15
  execution_id?: ExecutionId19
  parent_execution_id?: ParentExecutionId18
  payload: WorkflowCancelledPayload
  run_id: RunId18
  schema_version?: SchemaVersion15
  seq: Seq15
  task_id?: TaskId18
  tool_invocation_id?: ToolInvocationId15
  trace_id?: TraceId15
  ts?: Ts16
  type?: Type19
}

export interface WorkflowCancelledPayload {
  cancelled_by: CancelledBy
  reason: Reason2
}

export interface WorkflowRetryingEvent {
  correlation_id?: CorrelationId16
  event_id: EventId16
  execution_id?: ExecutionId20
  parent_execution_id?: ParentExecutionId19
  payload: WorkflowRetryingPayload
  run_id: RunId19
  schema_version?: SchemaVersion16
  seq: Seq16
  task_id?: TaskId19
  tool_invocation_id?: ToolInvocationId16
  trace_id?: TraceId16
  ts?: Ts17
  type?: Type20
}

export interface WorkflowRetryingPayload {
  attempt: Attempt
  delay_ms: DelayMs
  error_category: ErrorCategory
  next_attempt: NextAttempt
}

export interface PlanCreatedEvent {
  correlation_id?: CorrelationId17
  event_id: EventId17
  execution_id?: ExecutionId21
  parent_execution_id?: ParentExecutionId20
  payload: PlanCreatedPayload
  run_id: RunId20
  schema_version?: SchemaVersion17
  seq: Seq17
  task_id?: TaskId20
  tool_invocation_id?: ToolInvocationId17
  trace_id?: TraceId17
  ts?: Ts18
  type?: Type21
}

export interface PlanCreatedPayload {
  revision: Revision
  tasks: Tasks
}

export interface PlanTaskItem {
  assigned_agent: AssignedAgent
  dependencies?: Dependencies
  id: Id5
  name: Name
  required: Required
}

export interface PlanRevisedEvent {
  correlation_id?: CorrelationId18
  event_id: EventId18
  execution_id?: ExecutionId22
  parent_execution_id?: ParentExecutionId21
  payload: PlanRevisedPayload
  run_id: RunId21
  schema_version?: SchemaVersion18
  seq: Seq18
  task_id?: TaskId21
  tool_invocation_id?: ToolInvocationId18
  trace_id?: TraceId18
  ts?: Ts19
  type?: Type22
}

export interface PlanRevisedPayload {
  added_tasks?: AddedTasks
  reason: Reason3
  removed_tasks?: RemovedTasks
  revision: Revision1
}

export interface TaskAddedEvent {
  correlation_id?: CorrelationId19
  event_id: EventId19
  execution_id?: ExecutionId23
  parent_execution_id?: ParentExecutionId22
  payload: TaskAddedPayload
  run_id: RunId22
  schema_version?: SchemaVersion19
  seq: Seq19
  task_id?: TaskId23
  tool_invocation_id?: ToolInvocationId19
  trace_id?: TraceId19
  ts?: Ts20
  type?: Type23
}

export interface TaskAddedPayload {
  added_in_revision: AddedInRevision
  assigned_agent: AssignedAgent1
  dependencies?: Dependencies1
  name: Name1
  task_id: TaskId22
}

export interface TaskRemovedEvent {
  correlation_id?: CorrelationId20
  event_id: EventId20
  execution_id?: ExecutionId24
  parent_execution_id?: ParentExecutionId23
  payload: TaskRemovedPayload
  run_id: RunId23
  schema_version?: SchemaVersion20
  seq: Seq20
  task_id?: TaskId25
  tool_invocation_id?: ToolInvocationId20
  trace_id?: TraceId20
  ts?: Ts21
  type?: Type24
}

export interface TaskRemovedPayload {
  reason: Reason4
  task_id: TaskId24
}

export interface TaskQueuedEvent {
  correlation_id?: CorrelationId21
  event_id: EventId21
  execution_id?: ExecutionId25
  parent_execution_id?: ParentExecutionId24
  payload: TaskQueuedPayload
  run_id: RunId24
  schema_version?: SchemaVersion21
  seq: Seq21
  task_id?: TaskId27
  tool_invocation_id?: ToolInvocationId21
  trace_id?: TraceId21
  ts?: Ts22
  type?: Type25
}

export interface TaskQueuedPayload {
  priority?: Priority
  task_id: TaskId26
}

export interface TaskStartedEvent {
  correlation_id?: CorrelationId22
  event_id: EventId22
  execution_id?: ExecutionId26
  parent_execution_id?: ParentExecutionId25
  payload: TaskStartedPayload
  run_id: RunId25
  schema_version?: SchemaVersion22
  seq: Seq22
  task_id?: TaskId29
  tool_invocation_id?: ToolInvocationId22
  trace_id?: TraceId22
  ts?: Ts23
  type?: Type26
}

export interface TaskStartedPayload {
  assigned_agent: AssignedAgent2
  attempt: Attempt1
  task_id: TaskId28
}

export interface TaskCompletedEvent {
  correlation_id?: CorrelationId23
  event_id: EventId23
  execution_id?: ExecutionId27
  parent_execution_id?: ParentExecutionId26
  payload: TaskCompletedPayload
  run_id: RunId26
  schema_version?: SchemaVersion23
  seq: Seq23
  task_id?: TaskId31
  tool_invocation_id?: ToolInvocationId23
  trace_id?: TraceId23
  ts?: Ts24
  type?: Type27
}

export interface TaskCompletedPayload {
  duration_ms: DurationMs1
  output_summary?: OutputSummary
  task_id: TaskId30
}

export interface TaskFailedEvent {
  correlation_id?: CorrelationId24
  event_id: EventId24
  execution_id?: ExecutionId28
  parent_execution_id?: ParentExecutionId27
  payload: TaskFailedPayload
  run_id: RunId27
  schema_version?: SchemaVersion24
  seq: Seq24
  task_id?: TaskId33
  tool_invocation_id?: ToolInvocationId24
  trace_id?: TraceId24
  ts?: Ts25
  type?: Type28
}

export interface TaskFailedPayload {
  attempt: Attempt2
  can_retry?: CanRetry
  error: Error5
  task_id: TaskId32
}

export interface TaskRetryingEvent {
  correlation_id?: CorrelationId25
  event_id: EventId25
  execution_id?: ExecutionId29
  parent_execution_id?: ParentExecutionId28
  payload: TaskRetryingPayload
  run_id: RunId28
  schema_version?: SchemaVersion25
  seq: Seq25
  task_id?: TaskId35
  tool_invocation_id?: ToolInvocationId25
  trace_id?: TraceId25
  ts?: Ts26
  type?: Type29
}

export interface TaskRetryingPayload {
  delay_ms: DelayMs1
  next_attempt: NextAttempt1
  task_id: TaskId34
}

export interface TaskBlockedEvent {
  correlation_id?: CorrelationId26
  event_id: EventId26
  execution_id?: ExecutionId30
  parent_execution_id?: ParentExecutionId29
  payload: TaskBlockedPayload
  run_id: RunId29
  schema_version?: SchemaVersion26
  seq: Seq26
  task_id?: TaskId37
  tool_invocation_id?: ToolInvocationId26
  trace_id?: TraceId26
  ts?: Ts27
  type?: Type30
}

export interface TaskBlockedPayload {
  blocked_by_task_id: BlockedByTaskId
  task_id: TaskId36
}

export interface ToolStartedEvent {
  correlation_id?: CorrelationId27
  event_id: EventId27
  execution_id?: ExecutionId31
  parent_execution_id?: ParentExecutionId30
  payload: ToolStartedPayload
  run_id: RunId30
  schema_version?: SchemaVersion27
  seq: Seq27
  task_id?: TaskId38
  tool_invocation_id?: ToolInvocationId28
  trace_id?: TraceId27
  ts?: Ts28
  type?: Type31
}

export interface ToolStartedPayload {
  sanitized_input?: SanitizedInput
  tool_invocation_id: ToolInvocationId27
  tool_name: ToolName
}
export interface SanitizedInput {
  [k: string]: unknown
}

export interface ToolCompletedEvent {
  correlation_id?: CorrelationId28
  event_id: EventId28
  execution_id?: ExecutionId32
  parent_execution_id?: ParentExecutionId31
  payload: ToolCompletedPayload
  run_id: RunId31
  schema_version?: SchemaVersion28
  seq: Seq28
  task_id?: TaskId39
  tool_invocation_id?: ToolInvocationId30
  trace_id?: TraceId28
  ts?: Ts29
  type?: Type32
}

export interface ToolCompletedPayload {
  duration_ms: DurationMs2
  summary: Summary2
  tool_invocation_id: ToolInvocationId29
  tool_name: ToolName1
}

export interface ToolFailedEvent {
  correlation_id?: CorrelationId29
  event_id: EventId29
  execution_id?: ExecutionId33
  parent_execution_id?: ParentExecutionId32
  payload: ToolFailedPayload
  run_id: RunId32
  schema_version?: SchemaVersion29
  seq: Seq29
  task_id?: TaskId40
  tool_invocation_id?: ToolInvocationId32
  trace_id?: TraceId29
  ts?: Ts30
  type?: Type33
}

export interface ToolFailedPayload {
  error: Error6
  tool_invocation_id: ToolInvocationId31
  tool_name: ToolName2
}

export interface FindingSavedEvent {
  correlation_id?: CorrelationId30
  event_id: EventId30
  execution_id?: ExecutionId34
  parent_execution_id?: ParentExecutionId33
  payload: FindingSavedPayload
  run_id: RunId33
  schema_version?: SchemaVersion30
  seq: Seq30
  task_id?: TaskId41
  tool_invocation_id?: ToolInvocationId33
  trace_id?: TraceId30
  ts?: Ts31
  type?: Type34
}

export interface FindingSavedPayload {
  category: Category
  claim: Claim
  confidence: Confidence1
  finding_id: FindingId
}

export interface EvidenceSavedEvent {
  correlation_id?: CorrelationId31
  event_id: EventId31
  execution_id?: ExecutionId35
  parent_execution_id?: ParentExecutionId34
  payload: EvidenceSavedPayload
  run_id: RunId34
  schema_version?: SchemaVersion31
  seq: Seq31
  task_id?: TaskId42
  tool_invocation_id?: ToolInvocationId34
  trace_id?: TraceId31
  ts?: Ts32
  type?: Type35
}

export interface EvidenceSavedPayload {
  chunk_count: ChunkCount
  source_id: SourceId
  title: Title4
  url: Url1
}

export interface ExecutionResumedEvent {
  correlation_id?: CorrelationId32
  event_id: EventId32
  execution_id?: ExecutionId36
  parent_execution_id?: ParentExecutionId35
  payload: ExecutionResumedPayload
  run_id: RunId35
  schema_version?: SchemaVersion32
  seq: Seq32
  task_id?: TaskId43
  tool_invocation_id?: ToolInvocationId35
  trace_id?: TraceId32
  ts?: Ts33
  type?: Type36
}

export interface ExecutionResumedPayload {
  from_step: FromStep
  thread_id: ThreadId1
}

export interface WorkerRecoveredEvent {
  correlation_id?: CorrelationId33
  event_id: EventId33
  execution_id?: ExecutionId37
  parent_execution_id?: ParentExecutionId36
  payload: WorkerRecoveredPayload
  run_id: RunId37
  schema_version?: SchemaVersion33
  seq: Seq33
  task_id?: TaskId44
  tool_invocation_id?: ToolInvocationId36
  trace_id?: TraceId33
  ts?: Ts34
  type?: Type37
}

export interface WorkerRecoveredPayload {
  resumed_tasks_count: ResumedTasksCount
  run_id: RunId36
  worker_id: WorkerId
}

export interface LogEntry {
  attributes?: Attributes
  component?: Component
  duration_ms?: DurationMs3
  error_category?: ErrorCategory1
  execution_id?: ExecutionId38
  id: Id6
  message: Message
  run_id: RunId38
  seq?: Seq34
  severity?: Severity
  task_id?: TaskId45
  tool_invocation_id?: ToolInvocationId37
  trace_id?: TraceId34
  ts?: Ts35
}
export interface Attributes {
  [k: string]: unknown
}

export interface LogsPageResponse {
  items: Items
  next_cursor?: NextCursor
}

export interface RunsPageResponse {
  items: Items1
  next_cursor?: NextCursor1
}

export interface WorkflowRun {
  completed_at?: CompletedAt2
  created_at?: CreatedAt3
  current_phase?: CurrentPhase
  duration_ms?: DurationMs4
  error_message?: ErrorMessage
  id: Id7
  last_seq?: LastSeq1
  plan_revision?: PlanRevision1
  progress?: WorkflowProgress
  started_at?: StartedAt1
  status: Status4
  type: Type38
  updated_at?: UpdatedAt
  user_id: UserId1
  workflow_id: WorkflowId1
  workflow_name: WorkflowName
  workspace_id: WorkspaceId1
}

export interface WorkflowProgress {
  blocked?: Blocked
  cancelled?: Cancelled
  completed?: Completed
  estimate?: Estimate
  failed?: Failed
  pending?: Pending
  percentage?: Percentage
  phase_count?: PhaseCount
  phase_index?: PhaseIndex
  queued?: Queued
  retrying?: Retrying
  running?: Running
  skipped?: Skipped
  total_tasks?: TotalTasks1
}

export interface TaskGraphEdge {
  id: Id8
  kind?: Kind1
  source: Source1
  target: Target1
}

export interface TaskGraphNode {
  data: TaskGraphNodeData
  id: Id9
  label: Label1
  type?: Type39
}

export interface TaskGraphNodeData {
  added_in_revision: AddedInRevision1
  assigned_agent: AssignedAgent3
  attempt: Attempt3
  execution_ids?: ExecutionIds
  group?: Group
  required: Required1
  status: Status5
  task_type: TaskType
}

export interface TaskGraphResponse {
  edges: Edges1
  nodes: Nodes1
}

export interface ToolInvocation {
  completed_at?: CompletedAt3
  duration_ms?: DurationMs5
  error?: Error7
  execution_id: ExecutionId39
  id: Id10
  output_ref?: OutputRef
  output_summary?: OutputSummary1
  sanitized_input?: SanitizedInput1
  started_at?: StartedAt2
  status?: Status6
  tool_name: ToolName3
}
export interface SanitizedInput1 {
  [k: string]: unknown
}

export interface ToolInvocationsPageResponse {
  items: Items2
  next_cursor?: NextCursor2
}

export interface ToolOutputResponse {
  content: Content1
  has_more: HasMore
  limit: Limit
  offset: Offset
  tool_invocation_id: ToolInvocationId38
  total_bytes: TotalBytes
}

export interface WorkflowDefinition {
  default_max_retries?: DefaultMaxRetries
  description: Description1
  id: Id11
  name: Name2
  supported_agents?: SupportedAgents
  type: Type40
  version?: Version1
}

export interface WorkflowTask {
  added_in_revision?: AddedInRevision2
  assigned_agent: AssignedAgent4
  attempt?: Attempt4
  created_at?: CreatedAt4
  dependencies?: Dependencies2
  error?: Error8
  execution_ids?: ExecutionIds1
  id: Id12
  input_data?: InputData
  name: Name3
  output_data?: OutputData
  required?: Required2
  run_id: RunId39
  status?: Status7
  task_type: TaskType1
  updated_at?: UpdatedAt1
}
export interface InputData {
  [k: string]: unknown
}
export interface OutputData {
  [k: string]: unknown
}


export type WorkflowType = 'static' | 'dynamic'
export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type TaskStatus =
  | 'pending'
  | 'queued'
  | 'running'
  | 'completed'
  | 'failed'
  | 'retrying'
  | 'blocked'
  | 'cancelled'
  | 'skipped'
export type ExecutionStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type ToolStatus = 'pending' | 'running' | 'completed' | 'failed'
export type DelegationStatus = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type LogSeverity = 'debug' | 'info' | 'warn' | 'error'
export type ArtifactType =
  | 'research_report'
  | 'finding'
  | 'evidence'
  | 'question_set'
  | 'recommendation'
  | 'validation_result'

export type WorkflowEventType =
  | 'agent.activity'
  | 'agent.completed'
  | 'agent.failed'
  | 'agent.started'
  | 'artifact.created'
  | 'checkpoint.created'
  | 'delegation.accepted'
  | 'delegation.completed'
  | 'delegation.failed'
  | 'delegation.requested'
  | 'delegation.started'
  | 'evidence.saved'
  | 'execution.resumed'
  | 'finding.saved'
  | 'plan.created'
  | 'plan.revised'
  | 'task.added'
  | 'task.blocked'
  | 'task.completed'
  | 'task.failed'
  | 'task.queued'
  | 'task.removed'
  | 'task.retrying'
  | 'task.started'
  | 'tool.completed'
  | 'tool.failed'
  | 'tool.started'
  | 'worker.recovered'
  | 'workflow.cancelled'
  | 'workflow.completed'
  | 'workflow.created'
  | 'workflow.failed'
  | 'workflow.retrying'
  | 'workflow.started'

export const WORKFLOW_EVENT_TYPES: readonly WorkflowEventType[] = [
  'agent.activity',
  'agent.completed',
  'agent.failed',
  'agent.started',
  'artifact.created',
  'checkpoint.created',
  'delegation.accepted',
  'delegation.completed',
  'delegation.failed',
  'delegation.requested',
  'delegation.started',
  'evidence.saved',
  'execution.resumed',
  'finding.saved',
  'plan.created',
  'plan.revised',
  'task.added',
  'task.blocked',
  'task.completed',
  'task.failed',
  'task.queued',
  'task.removed',
  'task.retrying',
  'task.started',
  'tool.completed',
  'tool.failed',
  'tool.started',
  'worker.recovered',
  'workflow.cancelled',
  'workflow.completed',
  'workflow.created',
  'workflow.failed',
  'workflow.retrying',
  'workflow.started',
] as const

export type WorkflowEvent =
  | AgentActivityEvent
  | AgentCompletedEvent
  | AgentFailedEvent
  | AgentStartedEvent
  | ArtifactCreatedEvent
  | CheckpointCreatedEvent
  | DelegationAcceptedEvent
  | DelegationCompletedEvent
  | DelegationFailedEvent
  | DelegationRequestedEvent
  | DelegationStartedEvent
  | EvidenceSavedEvent
  | ExecutionResumedEvent
  | FindingSavedEvent
  | PlanCreatedEvent
  | PlanRevisedEvent
  | TaskAddedEvent
  | TaskBlockedEvent
  | TaskCompletedEvent
  | TaskFailedEvent
  | TaskQueuedEvent
  | TaskRemovedEvent
  | TaskRetryingEvent
  | TaskStartedEvent
  | ToolCompletedEvent
  | ToolFailedEvent
  | ToolStartedEvent
  | WorkerRecoveredEvent
  | WorkflowCancelledEvent
  | WorkflowCompletedEvent
  | WorkflowCreatedEvent
  | WorkflowFailedEvent
  | WorkflowRetryingEvent
  | WorkflowStartedEvent
