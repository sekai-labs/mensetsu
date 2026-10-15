export type SchemaVersion = string
export type EventId = string
export type RunId = string
export type Seq = number
export type Ts = string
export type TaskId = string | null
export type ExecutionId = string | null
export type ParentExecutionId = string | null
export type ToolInvocationId = string | null
export type CorrelationId = string | null
export type TraceId = string | null
export type Type = 'agent.activity'
export type ExecutionId1 = string
export type ActivityType = string
export type Description = string
export type SchemaVersion1 = string
export type EventId1 = string
export type RunId1 = string
export type Seq1 = number
export type Ts1 = string
export type TaskId1 = string | null
export type ExecutionId2 = string | null
export type ParentExecutionId1 = string | null
export type ToolInvocationId1 = string | null
export type CorrelationId1 = string | null
export type TraceId1 = string | null
export type Type1 = 'agent.completed'
export type ExecutionId3 = string
export type Summary = string
export type ToolCalls = number
export type TokensUsed = number
export type Id = string
export type RunId2 = string
export type TaskId2 = string
export type ParentExecutionId2 = string | null
export type ChildExecutionIds = string[]
export type AgentId = string
export type Objective = string
export type Status = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type DelegationDepth = number
export type Retries = number
export type CurrentActivity = string | null
export type ModelProvider = string
export type ModelName = string
export type ModelCalls = number
export type ToolCalls1 = number
export type TokensIn = number
export type TokensOut = number
export type WallTimeMs = number
export type ExternalRequests = number
export type ModelCalls1 = number
export type ToolCalls2 = number
export type TokensIn1 = number
export type TokensOut1 = number
export type WallTimeMs1 = number
export type ExternalRequests1 = number
export type ToolCallCount = number
export type Ts2 = string
export type Text = string
export type Notes = ExecutionNote[]
export type ArtifactIds = string[]
export type PublicSummary = string
export type Error = string | null
export type StartedAt = string
export type CompletedAt = string | null
export type SchemaVersion2 = string
export type EventId2 = string
export type RunId3 = string
export type Seq2 = number
export type Ts3 = string
export type TaskId3 = string | null
export type ExecutionId4 = string | null
export type ParentExecutionId3 = string | null
export type ToolInvocationId2 = string | null
export type CorrelationId2 = string | null
export type TraceId2 = string | null
export type Type2 = 'agent.failed'
export type ExecutionId5 = string
export type Error1 = string
export type SchemaVersion3 = string
export type EventId3 = string
export type RunId4 = string
export type Seq3 = number
export type Ts4 = string
export type TaskId4 = string | null
export type ExecutionId6 = string | null
export type ParentExecutionId4 = string | null
export type ToolInvocationId3 = string | null
export type CorrelationId3 = string | null
export type TraceId3 = string | null
export type Type3 = 'agent.started'
export type ExecutionId7 = string
export type AgentId1 = string
export type Objective1 = string
export type Model = string
export type Id1 = string
export type RunId5 = string
export type TaskId5 = string | null
export type AgentExecutionId = string | null
export type Type4 =
  | 'research_report'
  | 'finding'
  | 'evidence'
  | 'question_set'
  | 'recommendation'
  | 'validation_result'
  | 'routing_decision'
export type Title = string
export type Version = number
export type SurrealRecordId = string | null
export type EvidenceIds = string[]
export type Title1 = string
export type Url = string
export type SourceLinks = SourceLink[]
export type SourceUrl = string
export type Title2 = string
export type RetrievedAt = string
export type Snippet = string
export type Confidence = number
export type Content =
  | {
      [k: string]: unknown
    }
  | string
export type CreatedAt = string
export type SchemaVersion4 = string
export type EventId4 = string
export type RunId6 = string
export type Seq4 = number
export type Ts5 = string
export type TaskId6 = string | null
export type ExecutionId8 = string | null
export type ParentExecutionId5 = string | null
export type ToolInvocationId4 = string | null
export type CorrelationId4 = string | null
export type TraceId4 = string | null
export type Type5 = 'artifact.created'
export type ArtifactId = string
export type Type6 =
  | 'research_report'
  | 'finding'
  | 'evidence'
  | 'question_set'
  | 'recommendation'
  | 'validation_result'
  | 'routing_decision'
export type Title3 = string
export type SurrealRecordId1 = string | null
export type SchemaVersion5 = string
export type EventId5 = string
export type RunId7 = string
export type Seq5 = number
export type Ts6 = string
export type TaskId7 = string | null
export type ExecutionId9 = string | null
export type ParentExecutionId6 = string | null
export type ToolInvocationId5 = string | null
export type CorrelationId5 = string | null
export type TraceId5 = string | null
export type Type7 = 'checkpoint.created'
export type ThreadId = string
export type CheckpointId = string
export type Step = number
export type Id2 = string
export type RunId8 = string
export type ParentExecutionId7 = string
export type ChildExecutionId = string
export type ParentAgentId = string
export type ChildAgentId = string
export type DelegationTaskId = string
export type Objective2 = string
export type Reason = string
export type Depth = number
export type Status1 = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type InputArtifactIds = string[]
export type OutputArtifactIds = string[]
export type Error2 = string | null
export type CreatedAt1 = string
export type CompletedAt1 = string | null
export type SchemaVersion6 = string
export type EventId6 = string
export type RunId9 = string
export type Seq6 = number
export type Ts7 = string
export type TaskId8 = string | null
export type ExecutionId10 = string | null
export type ParentExecutionId8 = string | null
export type ToolInvocationId6 = string | null
export type CorrelationId6 = string | null
export type TraceId6 = string | null
export type Type8 = 'delegation.accepted'
export type DelegationId = string
export type ChildExecutionId1 = string
export type SchemaVersion7 = string
export type EventId7 = string
export type RunId10 = string
export type Seq7 = number
export type Ts8 = string
export type TaskId9 = string | null
export type ExecutionId11 = string | null
export type ParentExecutionId9 = string | null
export type ToolInvocationId7 = string | null
export type CorrelationId7 = string | null
export type TraceId7 = string | null
export type Type9 = 'delegation.completed'
export type DelegationId1 = string
export type ChildExecutionId2 = string
export type Summary1 = string
export type SchemaVersion8 = string
export type EventId8 = string
export type RunId11 = string
export type Seq8 = number
export type Ts9 = string
export type TaskId10 = string | null
export type ExecutionId12 = string | null
export type ParentExecutionId10 = string | null
export type ToolInvocationId8 = string | null
export type CorrelationId8 = string | null
export type TraceId8 = string | null
export type Type10 = 'delegation.failed'
export type DelegationId2 = string
export type Error3 = string
export type Id3 = string
export type Source = string
export type Target = string
export type Kind = 'delegation'
export type DelegationId3 = string
export type Objective3 = string
export type Status2 = 'requested' | 'accepted' | 'running' | 'completed' | 'failed' | 'rejected'
export type CreatedAt2 = string
export type ParentAgentId1 = string
export type ChildAgentId1 = string
export type Id4 = string
export type Label = string
export type Type11 = string
export type Status3 = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type TaskId11 = string
export type AgentId2 = string
export type Nodes = DelegationGraphNode[]
export type Edges = DelegationGraphEdge[]
export type SchemaVersion9 = string
export type EventId9 = string
export type RunId12 = string
export type Seq9 = number
export type Ts10 = string
export type TaskId12 = string | null
export type ExecutionId13 = string | null
export type ParentExecutionId11 = string | null
export type ToolInvocationId9 = string | null
export type CorrelationId9 = string | null
export type TraceId9 = string | null
export type Type12 = 'delegation.requested'
export type ParentExecutionId12 = string
export type TargetAgent = string
export type Reason1 = string
export type SchemaVersion10 = string
export type EventId10 = string
export type RunId13 = string
export type Seq10 = number
export type Ts11 = string
export type TaskId13 = string | null
export type ExecutionId14 = string | null
export type ParentExecutionId13 = string | null
export type ToolInvocationId10 = string | null
export type CorrelationId10 = string | null
export type TraceId10 = string | null
export type Type13 = 'delegation.started'
export type DelegationId4 = string
export type ChildExecutionId3 = string
export type SchemaVersion11 = string
export type EventId11 = string
export type RunId14 = string
export type Seq11 = number
export type Ts12 = string
export type TaskId14 = string | null
export type ExecutionId15 = string | null
export type ParentExecutionId14 = string | null
export type ToolInvocationId11 = string | null
export type CorrelationId11 = string | null
export type TraceId11 = string | null
export type Type14 = 'workflow.created'
export type WorkflowId = string
export type WorkspaceId = string
export type UserId = string
export type Type15 = 'static' | 'dynamic' | 'hybrid'
export type SchemaVersion12 = string
export type EventId12 = string
export type RunId15 = string
export type Seq12 = number
export type Ts13 = string
export type TaskId15 = string | null
export type ExecutionId16 = string | null
export type ParentExecutionId15 = string | null
export type ToolInvocationId12 = string | null
export type CorrelationId12 = string | null
export type TraceId12 = string | null
export type Type16 = 'workflow.started'
export type PlanRevision = number
export type InitialTasksCount = number
export type SchemaVersion13 = string
export type EventId13 = string
export type RunId16 = string
export type Seq13 = number
export type Ts14 = string
export type TaskId16 = string | null
export type ExecutionId17 = string | null
export type ParentExecutionId16 = string | null
export type ToolInvocationId13 = string | null
export type CorrelationId13 = string | null
export type TraceId13 = string | null
export type Type17 = 'workflow.completed'
export type DurationMs = number
export type TotalTasks = number
export type ArtifactsCount = number
export type Degraded = boolean
export type DegradedReasons = string[]
export type SchemaVersion14 = string
export type EventId14 = string
export type RunId17 = string
export type Seq14 = number
export type Ts15 = string
export type TaskId17 = string | null
export type ExecutionId18 = string | null
export type ParentExecutionId17 = string | null
export type ToolInvocationId14 = string | null
export type CorrelationId14 = string | null
export type TraceId14 = string | null
export type Type18 = 'workflow.failed'
export type Error4 = string
export type FailedTaskId = string | null
export type SchemaVersion15 = string
export type EventId15 = string
export type RunId18 = string
export type Seq15 = number
export type Ts16 = string
export type TaskId18 = string | null
export type ExecutionId19 = string | null
export type ParentExecutionId18 = string | null
export type ToolInvocationId15 = string | null
export type CorrelationId15 = string | null
export type TraceId15 = string | null
export type Type19 = 'workflow.cancelled'
export type Reason2 = string
export type CancelledBy = string
export type SchemaVersion16 = string
export type EventId16 = string
export type RunId19 = string
export type Seq16 = number
export type Ts17 = string
export type TaskId19 = string | null
export type ExecutionId20 = string | null
export type ParentExecutionId19 = string | null
export type ToolInvocationId16 = string | null
export type CorrelationId16 = string | null
export type TraceId16 = string | null
export type Type20 = 'workflow.retrying'
export type Attempt = number
export type NextAttempt = number
export type DelayMs = number
export type ErrorCategory = string
export type SchemaVersion17 = string
export type EventId17 = string
export type RunId20 = string
export type Seq17 = number
export type Ts18 = string
export type TaskId20 = string | null
export type ExecutionId21 = string | null
export type ParentExecutionId20 = string | null
export type ToolInvocationId17 = string | null
export type CorrelationId17 = string | null
export type TraceId17 = string | null
export type Type21 = 'plan.created'
export type Revision = number
export type Id5 = string
export type Name = string
export type AssignedAgent = string
export type Dependencies = string[]
export type Required = boolean
export type Tasks = PlanTaskItem[]
export type SchemaVersion18 = string
export type EventId18 = string
export type RunId21 = string
export type Seq18 = number
export type Ts19 = string
export type TaskId21 = string | null
export type ExecutionId22 = string | null
export type ParentExecutionId21 = string | null
export type ToolInvocationId18 = string | null
export type CorrelationId18 = string | null
export type TraceId18 = string | null
export type Type22 = 'plan.revised'
export type Revision1 = number
export type AddedTasks = string[]
export type RemovedTasks = string[]
export type Reason3 = string
export type SchemaVersion19 = string
export type EventId19 = string
export type RunId22 = string
export type Seq19 = number
export type Ts20 = string
export type TaskId22 = string | null
export type ExecutionId23 = string | null
export type ParentExecutionId22 = string | null
export type ToolInvocationId19 = string | null
export type CorrelationId19 = string | null
export type TraceId19 = string | null
export type Type23 = 'task.added'
export type TaskId23 = string
export type Name1 = string
export type AssignedAgent1 = string
export type Dependencies1 = string[]
export type AddedInRevision = number
export type SchemaVersion20 = string
export type EventId20 = string
export type RunId23 = string
export type Seq20 = number
export type Ts21 = string
export type TaskId24 = string | null
export type ExecutionId24 = string | null
export type ParentExecutionId23 = string | null
export type ToolInvocationId20 = string | null
export type CorrelationId20 = string | null
export type TraceId20 = string | null
export type Type24 = 'task.removed'
export type TaskId25 = string
export type Reason4 = string
export type SchemaVersion21 = string
export type EventId21 = string
export type RunId24 = string
export type Seq21 = number
export type Ts22 = string
export type TaskId26 = string | null
export type ExecutionId25 = string | null
export type ParentExecutionId24 = string | null
export type ToolInvocationId21 = string | null
export type CorrelationId21 = string | null
export type TraceId21 = string | null
export type Type25 = 'task.queued'
export type TaskId27 = string
export type Priority = number
export type SchemaVersion22 = string
export type EventId22 = string
export type RunId25 = string
export type Seq22 = number
export type Ts23 = string
export type TaskId28 = string | null
export type ExecutionId26 = string | null
export type ParentExecutionId25 = string | null
export type ToolInvocationId22 = string | null
export type CorrelationId22 = string | null
export type TraceId22 = string | null
export type Type26 = 'task.started'
export type TaskId29 = string
export type Attempt1 = number
export type AssignedAgent2 = string
export type SchemaVersion23 = string
export type EventId23 = string
export type RunId26 = string
export type Seq23 = number
export type Ts24 = string
export type TaskId30 = string | null
export type ExecutionId27 = string | null
export type ParentExecutionId26 = string | null
export type ToolInvocationId23 = string | null
export type CorrelationId23 = string | null
export type TraceId23 = string | null
export type Type27 = 'task.completed'
export type TaskId31 = string
export type DurationMs1 = number
export type OutputSummary = string
export type SchemaVersion24 = string
export type EventId24 = string
export type RunId27 = string
export type Seq24 = number
export type Ts25 = string
export type TaskId32 = string | null
export type ExecutionId28 = string | null
export type ParentExecutionId27 = string | null
export type ToolInvocationId24 = string | null
export type CorrelationId24 = string | null
export type TraceId24 = string | null
export type Type28 = 'task.failed'
export type TaskId33 = string
export type Attempt2 = number
export type Error5 = string
export type CanRetry = boolean
export type SchemaVersion25 = string
export type EventId25 = string
export type RunId28 = string
export type Seq25 = number
export type Ts26 = string
export type TaskId34 = string | null
export type ExecutionId29 = string | null
export type ParentExecutionId28 = string | null
export type ToolInvocationId25 = string | null
export type CorrelationId25 = string | null
export type TraceId25 = string | null
export type Type29 = 'task.retrying'
export type TaskId35 = string
export type NextAttempt1 = number
export type DelayMs1 = number
export type SchemaVersion26 = string
export type EventId26 = string
export type RunId29 = string
export type Seq26 = number
export type Ts27 = string
export type TaskId36 = string | null
export type ExecutionId30 = string | null
export type ParentExecutionId29 = string | null
export type ToolInvocationId26 = string | null
export type CorrelationId26 = string | null
export type TraceId26 = string | null
export type Type30 = 'task.blocked'
export type TaskId37 = string
export type BlockedByTaskId = string
export type SchemaVersion27 = string
export type EventId27 = string
export type RunId30 = string
export type Seq27 = number
export type Ts28 = string
export type TaskId38 = string | null
export type ExecutionId31 = string | null
export type ParentExecutionId30 = string | null
export type ToolInvocationId27 = string | null
export type CorrelationId27 = string | null
export type TraceId27 = string | null
export type Type31 = 'tool.started'
export type ToolInvocationId28 = string
export type ToolName = string
export type SchemaVersion28 = string
export type EventId28 = string
export type RunId31 = string
export type Seq28 = number
export type Ts29 = string
export type TaskId39 = string | null
export type ExecutionId32 = string | null
export type ParentExecutionId31 = string | null
export type ToolInvocationId29 = string | null
export type CorrelationId28 = string | null
export type TraceId28 = string | null
export type Type32 = 'tool.completed'
export type ToolInvocationId30 = string
export type ToolName1 = string
export type DurationMs2 = number
export type Summary2 = string
export type SchemaVersion29 = string
export type EventId29 = string
export type RunId32 = string
export type Seq29 = number
export type Ts30 = string
export type TaskId40 = string | null
export type ExecutionId33 = string | null
export type ParentExecutionId32 = string | null
export type ToolInvocationId31 = string | null
export type CorrelationId29 = string | null
export type TraceId29 = string | null
export type Type33 = 'tool.failed'
export type ToolInvocationId32 = string
export type ToolName2 = string
export type Error6 = string
export type SchemaVersion30 = string
export type EventId30 = string
export type RunId33 = string
export type Seq30 = number
export type Ts31 = string
export type TaskId41 = string | null
export type ExecutionId34 = string | null
export type ParentExecutionId33 = string | null
export type ToolInvocationId33 = string | null
export type CorrelationId30 = string | null
export type TraceId30 = string | null
export type Type34 = 'finding.saved'
export type FindingId = string
export type Category = string
export type Claim = string
export type Confidence1 = number
export type EvidenceIds1 = string[]
export type SchemaVersion31 = string
export type EventId31 = string
export type RunId34 = string
export type Seq31 = number
export type Ts32 = string
export type TaskId42 = string | null
export type ExecutionId35 = string | null
export type ParentExecutionId34 = string | null
export type ToolInvocationId34 = string | null
export type CorrelationId31 = string | null
export type TraceId31 = string | null
export type Type35 = 'evidence.saved'
export type SourceId = string
export type Url1 = string
export type Title4 = string
export type ChunkCount = number
export type SchemaVersion32 = string
export type EventId32 = string
export type RunId35 = string
export type Seq32 = number
export type Ts33 = string
export type TaskId43 = string | null
export type ExecutionId36 = string | null
export type ParentExecutionId35 = string | null
export type ToolInvocationId35 = string | null
export type CorrelationId32 = string | null
export type TraceId32 = string | null
export type Type36 = 'execution.resumed'
export type ThreadId1 = string
export type FromStep = number
export type SchemaVersion33 = string
export type EventId33 = string
export type RunId36 = string
export type Seq33 = number
export type Ts34 = string
export type TaskId44 = string | null
export type ExecutionId37 = string | null
export type ParentExecutionId36 = string | null
export type ToolInvocationId36 = string | null
export type CorrelationId33 = string | null
export type TraceId33 = string | null
export type Type37 = 'worker.recovered'
export type WorkerId = string
export type RunId37 = string
export type ResumedTasksCount = number
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
export type NextAfterSeq = number | null
export type LastSeq = number
export type Id6 = string
export type RunId38 = string
export type Seq34 = number | null
export type TaskId45 = string | null
export type ExecutionId38 = string | null
export type ToolInvocationId37 = string | null
export type TraceId34 = string | null
export type Component = string
export type Severity = 'debug' | 'info' | 'warn' | 'error'
export type Message = string
export type DurationMs3 = number | null
export type ErrorCategory1 =
  ('network' | 'rate_limit' | 'budget_exceeded' | 'llm_error' | 'validation_error' | 'internal_error') | null
export type Ts35 = string
export type Items = LogEntry[]
export type NextCursor = string | null
export type Id7 = string
export type WorkflowId1 = string
export type WorkflowName = string
export type WorkspaceId1 = string
export type UserId1 = string
export type Type38 = 'static' | 'dynamic' | 'hybrid'
export type Status4 = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'
export type CurrentPhase = string
export type PlanRevision1 = number
export type TotalTasks1 = number
export type Pending = number
export type Queued = number
export type Running = number
export type Completed = number
export type Failed = number
export type Retrying = number
export type Blocked = number
export type Cancelled = number
export type Skipped = number
export type Percentage = number | null
export type Estimate = boolean
export type PhaseIndex = number | null
export type PhaseCount = number | null
export type LastSeq1 = number
export type StartedAt1 = string | null
export type CompletedAt2 = string | null
export type DurationMs4 = number | null
export type CreatedAt3 = string
export type UpdatedAt = string
export type ErrorMessage = string | null
export type Degraded1 = boolean
export type DegradedReasons1 = string[]
export type Items1 = WorkflowRun[]
export type NextCursor1 = string | null
export type Id8 = string
export type Source1 = string
export type Target1 = string
export type Kind1 = 'dependency'
export type Id9 = string
export type Label1 = string
export type Type39 = string
export type Status5 =
  'pending' | 'queued' | 'running' | 'completed' | 'failed' | 'retrying' | 'blocked' | 'cancelled' | 'skipped'
export type AssignedAgent3 = string
export type Attempt3 = number
export type TaskType = string
export type Required1 = boolean
export type AddedInRevision1 = number
export type ExecutionIds = string[]
export type Group = string
export type Nodes1 = TaskGraphNode[]
export type Edges1 = TaskGraphEdge[]
export type Id10 = string
export type ExecutionId39 = string
export type ToolName3 = string
export type Status6 = 'pending' | 'running' | 'completed' | 'failed'
export type DurationMs5 = number
export type OutputSummary1 = string
export type OutputRef = string | null
export type Error7 = string | null
export type StartedAt2 = string
export type CompletedAt3 = string | null
export type Items2 = ToolInvocation[]
export type NextCursor2 = string | null
export type ToolInvocationId38 = string
export type Offset = number
export type Limit = number
export type TotalBytes = number
export type HasMore = boolean
export type Content1 = string
export type Id11 = string
export type Name2 = string
export type Type40 = 'static' | 'dynamic' | 'hybrid'
export type Version1 = string
export type Description1 = string
export type DefaultMaxRetries = number
export type SupportedAgents = string[]
export type Id12 = string
export type RunId39 = string
export type TaskType1 = string
export type Name3 = string
export type AssignedAgent4 = string
export type Dependencies2 = string[]
export type Status7 =
  'pending' | 'queued' | 'running' | 'completed' | 'failed' | 'retrying' | 'blocked' | 'cancelled' | 'skipped'
export type Attempt4 = number
export type Required2 = boolean
export type AddedInRevision2 = number
export type ExecutionIds1 = string[]
export type Error8 = string | null
export type CreatedAt4 = string
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
  schema_version?: SchemaVersion
  event_id: EventId
  run_id: RunId
  seq: Seq
  ts?: Ts
  task_id?: TaskId
  execution_id?: ExecutionId
  parent_execution_id?: ParentExecutionId
  tool_invocation_id?: ToolInvocationId
  correlation_id?: CorrelationId
  trace_id?: TraceId
  type?: Type
  payload: AgentActivityPayload
}

export interface AgentActivityPayload {
  execution_id: ExecutionId1
  activity_type: ActivityType
  description: Description
}

export interface AgentCompletedEvent {
  schema_version?: SchemaVersion1
  event_id: EventId1
  run_id: RunId1
  seq: Seq1
  ts?: Ts1
  task_id?: TaskId1
  execution_id?: ExecutionId2
  parent_execution_id?: ParentExecutionId1
  tool_invocation_id?: ToolInvocationId1
  correlation_id?: CorrelationId1
  trace_id?: TraceId1
  type?: Type1
  payload: AgentCompletedPayload
}

export interface AgentCompletedPayload {
  execution_id: ExecutionId3
  summary: Summary
  tool_calls: ToolCalls
  tokens_used: TokensUsed
}

export interface AgentExecution {
  id: Id
  run_id: RunId2
  task_id: TaskId2
  parent_execution_id?: ParentExecutionId2
  child_execution_ids?: ChildExecutionIds
  agent_id: AgentId
  objective?: Objective
  status?: Status
  delegation_depth?: DelegationDepth
  retries?: Retries
  current_activity?: CurrentActivity
  model_provider?: ModelProvider
  model_name?: ModelName
  budgets?: ExecutionBudgets
  budgets_used?: ExecutionBudgetsUsed
  tool_call_count?: ToolCallCount
  notes?: Notes
  artifact_ids?: ArtifactIds
  public_summary?: PublicSummary
  error?: Error
  started_at?: StartedAt
  completed_at?: CompletedAt
}

export interface ExecutionBudgets {
  model_calls?: ModelCalls
  tool_calls?: ToolCalls1
  tokens_in?: TokensIn
  tokens_out?: TokensOut
  wall_time_ms?: WallTimeMs
  external_requests?: ExternalRequests
}

export interface ExecutionBudgetsUsed {
  model_calls?: ModelCalls1
  tool_calls?: ToolCalls2
  tokens_in?: TokensIn1
  tokens_out?: TokensOut1
  wall_time_ms?: WallTimeMs1
  external_requests?: ExternalRequests1
}

export interface ExecutionNote {
  ts?: Ts2
  text: Text
}

export interface AgentFailedEvent {
  schema_version?: SchemaVersion2
  event_id: EventId2
  run_id: RunId3
  seq: Seq2
  ts?: Ts3
  task_id?: TaskId3
  execution_id?: ExecutionId4
  parent_execution_id?: ParentExecutionId3
  tool_invocation_id?: ToolInvocationId2
  correlation_id?: CorrelationId2
  trace_id?: TraceId2
  type?: Type2
  payload: AgentFailedPayload
}

export interface AgentFailedPayload {
  execution_id: ExecutionId5
  error: Error1
}

export interface AgentStartedEvent {
  schema_version?: SchemaVersion3
  event_id: EventId3
  run_id: RunId4
  seq: Seq3
  ts?: Ts4
  task_id?: TaskId4
  execution_id?: ExecutionId6
  parent_execution_id?: ParentExecutionId4
  tool_invocation_id?: ToolInvocationId3
  correlation_id?: CorrelationId3
  trace_id?: TraceId3
  type?: Type3
  payload: AgentStartedPayload
}

export interface AgentStartedPayload {
  execution_id: ExecutionId7
  agent_id: AgentId1
  objective: Objective1
  model: Model
}

export interface Artifact {
  id: Id1
  run_id: RunId5
  task_id?: TaskId5
  agent_execution_id?: AgentExecutionId
  type: Type4
  title: Title
  version?: Version
  surreal_record_id?: SurrealRecordId
  evidence_ids?: EvidenceIds
  source_links?: SourceLinks
  evidence?: EvidenceProvenance | null
  content?: Content
  created_at?: CreatedAt
}

export interface SourceLink {
  title: Title1
  url: Url
}

export interface EvidenceProvenance {
  source_url: SourceUrl
  title: Title2
  retrieved_at?: RetrievedAt
  snippet: Snippet
  confidence: Confidence
}

export interface ArtifactCreatedEvent {
  schema_version?: SchemaVersion4
  event_id: EventId4
  run_id: RunId6
  seq: Seq4
  ts?: Ts5
  task_id?: TaskId6
  execution_id?: ExecutionId8
  parent_execution_id?: ParentExecutionId5
  tool_invocation_id?: ToolInvocationId4
  correlation_id?: CorrelationId4
  trace_id?: TraceId4
  type?: Type5
  payload: ArtifactCreatedPayload
}

export interface ArtifactCreatedPayload {
  artifact_id: ArtifactId
  type: Type6
  title: Title3
  surreal_record_id?: SurrealRecordId1
}

export interface CheckpointCreatedEvent {
  schema_version?: SchemaVersion5
  event_id: EventId5
  run_id: RunId7
  seq: Seq5
  ts?: Ts6
  task_id?: TaskId7
  execution_id?: ExecutionId9
  parent_execution_id?: ParentExecutionId6
  tool_invocation_id?: ToolInvocationId5
  correlation_id?: CorrelationId5
  trace_id?: TraceId5
  type?: Type7
  payload: CheckpointCreatedPayload
}

export interface CheckpointCreatedPayload {
  thread_id: ThreadId
  checkpoint_id: CheckpointId
  step: Step
}

export interface Delegation {
  id: Id2
  run_id: RunId8
  parent_execution_id: ParentExecutionId7
  child_execution_id: ChildExecutionId
  parent_agent_id: ParentAgentId
  child_agent_id: ChildAgentId
  delegation_task_id: DelegationTaskId
  objective: Objective2
  reason: Reason
  depth?: Depth
  status?: Status1
  input_artifact_ids?: InputArtifactIds
  output_artifact_ids?: OutputArtifactIds
  error?: Error2
  created_at?: CreatedAt1
  completed_at?: CompletedAt1
}

export interface DelegationAcceptedEvent {
  schema_version?: SchemaVersion6
  event_id: EventId6
  run_id: RunId9
  seq: Seq6
  ts?: Ts7
  task_id?: TaskId8
  execution_id?: ExecutionId10
  parent_execution_id?: ParentExecutionId8
  tool_invocation_id?: ToolInvocationId6
  correlation_id?: CorrelationId6
  trace_id?: TraceId6
  type?: Type8
  payload: DelegationAcceptedPayload
}

export interface DelegationAcceptedPayload {
  delegation_id: DelegationId
  child_execution_id: ChildExecutionId1
}

export interface DelegationCompletedEvent {
  schema_version?: SchemaVersion7
  event_id: EventId7
  run_id: RunId10
  seq: Seq7
  ts?: Ts8
  task_id?: TaskId9
  execution_id?: ExecutionId11
  parent_execution_id?: ParentExecutionId9
  tool_invocation_id?: ToolInvocationId7
  correlation_id?: CorrelationId7
  trace_id?: TraceId7
  type?: Type9
  payload: DelegationCompletedPayload
}

export interface DelegationCompletedPayload {
  delegation_id: DelegationId1
  child_execution_id: ChildExecutionId2
  summary: Summary1
}

export interface DelegationFailedEvent {
  schema_version?: SchemaVersion8
  event_id: EventId8
  run_id: RunId11
  seq: Seq8
  ts?: Ts9
  task_id?: TaskId10
  execution_id?: ExecutionId12
  parent_execution_id?: ParentExecutionId10
  tool_invocation_id?: ToolInvocationId8
  correlation_id?: CorrelationId8
  trace_id?: TraceId8
  type?: Type10
  payload: DelegationFailedPayload
}

export interface DelegationFailedPayload {
  delegation_id: DelegationId2
  error: Error3
}

export interface DelegationGraphEdge {
  id: Id3
  source: Source
  target: Target
  kind?: Kind
  data: DelegationGraphEdgeData
}

export interface DelegationGraphEdgeData {
  delegation_id: DelegationId3
  objective: Objective3
  status: Status2
  created_at: CreatedAt2
  parent_agent_id: ParentAgentId1
  child_agent_id: ChildAgentId1
}

export interface DelegationGraphNode {
  id: Id4
  label: Label
  type?: Type11
  data: DelegationGraphNodeData
}

export interface DelegationGraphNodeData {
  status: Status3
  task_id: TaskId11
  agent_id: AgentId2
}

export interface DelegationGraphResponse {
  nodes: Nodes
  edges: Edges
}

export interface DelegationRequestedEvent {
  schema_version?: SchemaVersion9
  event_id: EventId9
  run_id: RunId12
  seq: Seq9
  ts?: Ts10
  task_id?: TaskId12
  execution_id?: ExecutionId13
  parent_execution_id?: ParentExecutionId11
  tool_invocation_id?: ToolInvocationId9
  correlation_id?: CorrelationId9
  trace_id?: TraceId9
  type?: Type12
  payload: DelegationRequestedPayload
}

export interface DelegationRequestedPayload {
  parent_execution_id: ParentExecutionId12
  target_agent: TargetAgent
  reason: Reason1
}

export interface DelegationStartedEvent {
  schema_version?: SchemaVersion10
  event_id: EventId10
  run_id: RunId13
  seq: Seq10
  ts?: Ts11
  task_id?: TaskId13
  execution_id?: ExecutionId14
  parent_execution_id?: ParentExecutionId13
  tool_invocation_id?: ToolInvocationId10
  correlation_id?: CorrelationId10
  trace_id?: TraceId10
  type?: Type13
  payload: DelegationStartedPayload
}

export interface DelegationStartedPayload {
  delegation_id: DelegationId4
  child_execution_id: ChildExecutionId3
}

export interface EventsHistoryPageResponse {
  events: Events
  next_after_seq?: NextAfterSeq
  last_seq?: LastSeq
}

export interface WorkflowCreatedEvent {
  schema_version?: SchemaVersion11
  event_id: EventId11
  run_id: RunId14
  seq: Seq11
  ts?: Ts12
  task_id?: TaskId14
  execution_id?: ExecutionId15
  parent_execution_id?: ParentExecutionId14
  tool_invocation_id?: ToolInvocationId11
  correlation_id?: CorrelationId11
  trace_id?: TraceId11
  type?: Type14
  payload: WorkflowCreatedPayload
}

export interface WorkflowCreatedPayload {
  workflow_id: WorkflowId
  workspace_id: WorkspaceId
  user_id: UserId
  type: Type15
}

export interface WorkflowStartedEvent {
  schema_version?: SchemaVersion12
  event_id: EventId12
  run_id: RunId15
  seq: Seq12
  ts?: Ts13
  task_id?: TaskId15
  execution_id?: ExecutionId16
  parent_execution_id?: ParentExecutionId15
  tool_invocation_id?: ToolInvocationId12
  correlation_id?: CorrelationId12
  trace_id?: TraceId12
  type?: Type16
  payload: WorkflowStartedPayload
}

export interface WorkflowStartedPayload {
  plan_revision: PlanRevision
  initial_tasks_count: InitialTasksCount
}

export interface WorkflowCompletedEvent {
  schema_version?: SchemaVersion13
  event_id: EventId13
  run_id: RunId16
  seq: Seq13
  ts?: Ts14
  task_id?: TaskId16
  execution_id?: ExecutionId17
  parent_execution_id?: ParentExecutionId16
  tool_invocation_id?: ToolInvocationId13
  correlation_id?: CorrelationId13
  trace_id?: TraceId13
  type?: Type17
  payload: WorkflowCompletedPayload
}

export interface WorkflowCompletedPayload {
  duration_ms: DurationMs
  total_tasks: TotalTasks
  artifacts_count: ArtifactsCount
  degraded?: Degraded
  degraded_reasons?: DegradedReasons
}

export interface WorkflowFailedEvent {
  schema_version?: SchemaVersion14
  event_id: EventId14
  run_id: RunId17
  seq: Seq14
  ts?: Ts15
  task_id?: TaskId17
  execution_id?: ExecutionId18
  parent_execution_id?: ParentExecutionId17
  tool_invocation_id?: ToolInvocationId14
  correlation_id?: CorrelationId14
  trace_id?: TraceId14
  type?: Type18
  payload: WorkflowFailedPayload
}

export interface WorkflowFailedPayload {
  error: Error4
  failed_task_id?: FailedTaskId
}

export interface WorkflowCancelledEvent {
  schema_version?: SchemaVersion15
  event_id: EventId15
  run_id: RunId18
  seq: Seq15
  ts?: Ts16
  task_id?: TaskId18
  execution_id?: ExecutionId19
  parent_execution_id?: ParentExecutionId18
  tool_invocation_id?: ToolInvocationId15
  correlation_id?: CorrelationId15
  trace_id?: TraceId15
  type?: Type19
  payload: WorkflowCancelledPayload
}

export interface WorkflowCancelledPayload {
  reason: Reason2
  cancelled_by: CancelledBy
}

export interface WorkflowRetryingEvent {
  schema_version?: SchemaVersion16
  event_id: EventId16
  run_id: RunId19
  seq: Seq16
  ts?: Ts17
  task_id?: TaskId19
  execution_id?: ExecutionId20
  parent_execution_id?: ParentExecutionId19
  tool_invocation_id?: ToolInvocationId16
  correlation_id?: CorrelationId16
  trace_id?: TraceId16
  type?: Type20
  payload: WorkflowRetryingPayload
}

export interface WorkflowRetryingPayload {
  attempt: Attempt
  next_attempt: NextAttempt
  delay_ms: DelayMs
  error_category: ErrorCategory
}

export interface PlanCreatedEvent {
  schema_version?: SchemaVersion17
  event_id: EventId17
  run_id: RunId20
  seq: Seq17
  ts?: Ts18
  task_id?: TaskId20
  execution_id?: ExecutionId21
  parent_execution_id?: ParentExecutionId20
  tool_invocation_id?: ToolInvocationId17
  correlation_id?: CorrelationId17
  trace_id?: TraceId17
  type?: Type21
  payload: PlanCreatedPayload
}

export interface PlanCreatedPayload {
  revision: Revision
  tasks: Tasks
}

export interface PlanTaskItem {
  id: Id5
  name: Name
  assigned_agent: AssignedAgent
  dependencies?: Dependencies
  required: Required
}

export interface PlanRevisedEvent {
  schema_version?: SchemaVersion18
  event_id: EventId18
  run_id: RunId21
  seq: Seq18
  ts?: Ts19
  task_id?: TaskId21
  execution_id?: ExecutionId22
  parent_execution_id?: ParentExecutionId21
  tool_invocation_id?: ToolInvocationId18
  correlation_id?: CorrelationId18
  trace_id?: TraceId18
  type?: Type22
  payload: PlanRevisedPayload
}

export interface PlanRevisedPayload {
  revision: Revision1
  added_tasks?: AddedTasks
  removed_tasks?: RemovedTasks
  reason: Reason3
}

export interface TaskAddedEvent {
  schema_version?: SchemaVersion19
  event_id: EventId19
  run_id: RunId22
  seq: Seq19
  ts?: Ts20
  task_id?: TaskId22
  execution_id?: ExecutionId23
  parent_execution_id?: ParentExecutionId22
  tool_invocation_id?: ToolInvocationId19
  correlation_id?: CorrelationId19
  trace_id?: TraceId19
  type?: Type23
  payload: TaskAddedPayload
}

export interface TaskAddedPayload {
  task_id: TaskId23
  name: Name1
  assigned_agent: AssignedAgent1
  dependencies?: Dependencies1
  added_in_revision: AddedInRevision
}

export interface TaskRemovedEvent {
  schema_version?: SchemaVersion20
  event_id: EventId20
  run_id: RunId23
  seq: Seq20
  ts?: Ts21
  task_id?: TaskId24
  execution_id?: ExecutionId24
  parent_execution_id?: ParentExecutionId23
  tool_invocation_id?: ToolInvocationId20
  correlation_id?: CorrelationId20
  trace_id?: TraceId20
  type?: Type24
  payload: TaskRemovedPayload
}

export interface TaskRemovedPayload {
  task_id: TaskId25
  reason: Reason4
}

export interface TaskQueuedEvent {
  schema_version?: SchemaVersion21
  event_id: EventId21
  run_id: RunId24
  seq: Seq21
  ts?: Ts22
  task_id?: TaskId26
  execution_id?: ExecutionId25
  parent_execution_id?: ParentExecutionId24
  tool_invocation_id?: ToolInvocationId21
  correlation_id?: CorrelationId21
  trace_id?: TraceId21
  type?: Type25
  payload: TaskQueuedPayload
}

export interface TaskQueuedPayload {
  task_id: TaskId27
  priority?: Priority
}

export interface TaskStartedEvent {
  schema_version?: SchemaVersion22
  event_id: EventId22
  run_id: RunId25
  seq: Seq22
  ts?: Ts23
  task_id?: TaskId28
  execution_id?: ExecutionId26
  parent_execution_id?: ParentExecutionId25
  tool_invocation_id?: ToolInvocationId22
  correlation_id?: CorrelationId22
  trace_id?: TraceId22
  type?: Type26
  payload: TaskStartedPayload
}

export interface TaskStartedPayload {
  task_id: TaskId29
  attempt: Attempt1
  assigned_agent: AssignedAgent2
}

export interface TaskCompletedEvent {
  schema_version?: SchemaVersion23
  event_id: EventId23
  run_id: RunId26
  seq: Seq23
  ts?: Ts24
  task_id?: TaskId30
  execution_id?: ExecutionId27
  parent_execution_id?: ParentExecutionId26
  tool_invocation_id?: ToolInvocationId23
  correlation_id?: CorrelationId23
  trace_id?: TraceId23
  type?: Type27
  payload: TaskCompletedPayload
}

export interface TaskCompletedPayload {
  task_id: TaskId31
  duration_ms: DurationMs1
  output_summary?: OutputSummary
}

export interface TaskFailedEvent {
  schema_version?: SchemaVersion24
  event_id: EventId24
  run_id: RunId27
  seq: Seq24
  ts?: Ts25
  task_id?: TaskId32
  execution_id?: ExecutionId28
  parent_execution_id?: ParentExecutionId27
  tool_invocation_id?: ToolInvocationId24
  correlation_id?: CorrelationId24
  trace_id?: TraceId24
  type?: Type28
  payload: TaskFailedPayload
}

export interface TaskFailedPayload {
  task_id: TaskId33
  attempt: Attempt2
  error: Error5
  can_retry?: CanRetry
}

export interface TaskRetryingEvent {
  schema_version?: SchemaVersion25
  event_id: EventId25
  run_id: RunId28
  seq: Seq25
  ts?: Ts26
  task_id?: TaskId34
  execution_id?: ExecutionId29
  parent_execution_id?: ParentExecutionId28
  tool_invocation_id?: ToolInvocationId25
  correlation_id?: CorrelationId25
  trace_id?: TraceId25
  type?: Type29
  payload: TaskRetryingPayload
}

export interface TaskRetryingPayload {
  task_id: TaskId35
  next_attempt: NextAttempt1
  delay_ms: DelayMs1
}

export interface TaskBlockedEvent {
  schema_version?: SchemaVersion26
  event_id: EventId26
  run_id: RunId29
  seq: Seq26
  ts?: Ts27
  task_id?: TaskId36
  execution_id?: ExecutionId30
  parent_execution_id?: ParentExecutionId29
  tool_invocation_id?: ToolInvocationId26
  correlation_id?: CorrelationId26
  trace_id?: TraceId26
  type?: Type30
  payload: TaskBlockedPayload
}

export interface TaskBlockedPayload {
  task_id: TaskId37
  blocked_by_task_id: BlockedByTaskId
}

export interface ToolStartedEvent {
  schema_version?: SchemaVersion27
  event_id: EventId27
  run_id: RunId30
  seq: Seq27
  ts?: Ts28
  task_id?: TaskId38
  execution_id?: ExecutionId31
  parent_execution_id?: ParentExecutionId30
  tool_invocation_id?: ToolInvocationId27
  correlation_id?: CorrelationId27
  trace_id?: TraceId27
  type?: Type31
  payload: ToolStartedPayload
}

export interface ToolStartedPayload {
  tool_invocation_id: ToolInvocationId28
  tool_name: ToolName
  sanitized_input?: SanitizedInput
}
export interface SanitizedInput {
  [k: string]: unknown
}

export interface ToolCompletedEvent {
  schema_version?: SchemaVersion28
  event_id: EventId28
  run_id: RunId31
  seq: Seq28
  ts?: Ts29
  task_id?: TaskId39
  execution_id?: ExecutionId32
  parent_execution_id?: ParentExecutionId31
  tool_invocation_id?: ToolInvocationId29
  correlation_id?: CorrelationId28
  trace_id?: TraceId28
  type?: Type32
  payload: ToolCompletedPayload
}

export interface ToolCompletedPayload {
  tool_invocation_id: ToolInvocationId30
  tool_name: ToolName1
  duration_ms: DurationMs2
  summary: Summary2
}

export interface ToolFailedEvent {
  schema_version?: SchemaVersion29
  event_id: EventId29
  run_id: RunId32
  seq: Seq29
  ts?: Ts30
  task_id?: TaskId40
  execution_id?: ExecutionId33
  parent_execution_id?: ParentExecutionId32
  tool_invocation_id?: ToolInvocationId31
  correlation_id?: CorrelationId29
  trace_id?: TraceId29
  type?: Type33
  payload: ToolFailedPayload
}

export interface ToolFailedPayload {
  tool_invocation_id: ToolInvocationId32
  tool_name: ToolName2
  error: Error6
}

export interface FindingSavedEvent {
  schema_version?: SchemaVersion30
  event_id: EventId30
  run_id: RunId33
  seq: Seq30
  ts?: Ts31
  task_id?: TaskId41
  execution_id?: ExecutionId34
  parent_execution_id?: ParentExecutionId33
  tool_invocation_id?: ToolInvocationId33
  correlation_id?: CorrelationId30
  trace_id?: TraceId30
  type?: Type34
  payload: FindingSavedPayload
}

export interface FindingSavedPayload {
  finding_id: FindingId
  category: Category
  claim: Claim
  confidence: Confidence1
  evidence_ids?: EvidenceIds1
}

export interface EvidenceSavedEvent {
  schema_version?: SchemaVersion31
  event_id: EventId31
  run_id: RunId34
  seq: Seq31
  ts?: Ts32
  task_id?: TaskId42
  execution_id?: ExecutionId35
  parent_execution_id?: ParentExecutionId34
  tool_invocation_id?: ToolInvocationId34
  correlation_id?: CorrelationId31
  trace_id?: TraceId31
  type?: Type35
  payload: EvidenceSavedPayload
}

export interface EvidenceSavedPayload {
  source_id: SourceId
  url: Url1
  title: Title4
  chunk_count: ChunkCount
}

export interface ExecutionResumedEvent {
  schema_version?: SchemaVersion32
  event_id: EventId32
  run_id: RunId35
  seq: Seq32
  ts?: Ts33
  task_id?: TaskId43
  execution_id?: ExecutionId36
  parent_execution_id?: ParentExecutionId35
  tool_invocation_id?: ToolInvocationId35
  correlation_id?: CorrelationId32
  trace_id?: TraceId32
  type?: Type36
  payload: ExecutionResumedPayload
}

export interface ExecutionResumedPayload {
  thread_id: ThreadId1
  from_step: FromStep
}

export interface WorkerRecoveredEvent {
  schema_version?: SchemaVersion33
  event_id: EventId33
  run_id: RunId36
  seq: Seq33
  ts?: Ts34
  task_id?: TaskId44
  execution_id?: ExecutionId37
  parent_execution_id?: ParentExecutionId36
  tool_invocation_id?: ToolInvocationId36
  correlation_id?: CorrelationId33
  trace_id?: TraceId33
  type?: Type37
  payload: WorkerRecoveredPayload
}

export interface WorkerRecoveredPayload {
  worker_id: WorkerId
  run_id: RunId37
  resumed_tasks_count: ResumedTasksCount
}

export interface LogEntry {
  id: Id6
  run_id: RunId38
  seq?: Seq34
  task_id?: TaskId45
  execution_id?: ExecutionId38
  tool_invocation_id?: ToolInvocationId37
  trace_id?: TraceId34
  component?: Component
  severity?: Severity
  message: Message
  duration_ms?: DurationMs3
  error_category?: ErrorCategory1
  attributes?: Attributes
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
  id: Id7
  workflow_id: WorkflowId1
  workflow_name: WorkflowName
  workspace_id: WorkspaceId1
  user_id: UserId1
  type: Type38
  status: Status4
  current_phase?: CurrentPhase
  plan_revision?: PlanRevision1
  progress?: WorkflowProgress
  last_seq?: LastSeq1
  started_at?: StartedAt1
  completed_at?: CompletedAt2
  duration_ms?: DurationMs4
  created_at?: CreatedAt3
  updated_at?: UpdatedAt
  error_message?: ErrorMessage
  degraded?: Degraded1
  degraded_reasons?: DegradedReasons1
}

export interface WorkflowProgress {
  total_tasks?: TotalTasks1
  pending?: Pending
  queued?: Queued
  running?: Running
  completed?: Completed
  failed?: Failed
  retrying?: Retrying
  blocked?: Blocked
  cancelled?: Cancelled
  skipped?: Skipped
  percentage?: Percentage
  estimate?: Estimate
  phase_index?: PhaseIndex
  phase_count?: PhaseCount
}

export interface TaskGraphEdge {
  id: Id8
  source: Source1
  target: Target1
  kind?: Kind1
}

export interface TaskGraphNode {
  id: Id9
  label: Label1
  type?: Type39
  data: TaskGraphNodeData
}

export interface TaskGraphNodeData {
  status: Status5
  assigned_agent: AssignedAgent3
  attempt: Attempt3
  task_type: TaskType
  required: Required1
  added_in_revision: AddedInRevision1
  execution_ids?: ExecutionIds
  group?: Group
}

export interface TaskGraphResponse {
  nodes: Nodes1
  edges: Edges1
}

export interface ToolInvocation {
  id: Id10
  execution_id: ExecutionId39
  tool_name: ToolName3
  status?: Status6
  duration_ms?: DurationMs5
  sanitized_input?: SanitizedInput1
  output_summary?: OutputSummary1
  output_ref?: OutputRef
  error?: Error7
  started_at?: StartedAt2
  completed_at?: CompletedAt3
}
export interface SanitizedInput1 {
  [k: string]: unknown
}

export interface ToolInvocationsPageResponse {
  items: Items2
  next_cursor?: NextCursor2
}

export interface ToolOutputResponse {
  tool_invocation_id: ToolInvocationId38
  offset: Offset
  limit: Limit
  total_bytes: TotalBytes
  has_more: HasMore
  content: Content1
}

export interface WorkflowDefinition {
  id: Id11
  name: Name2
  type: Type40
  version?: Version1
  description: Description1
  default_max_retries?: DefaultMaxRetries
  supported_agents?: SupportedAgents
}

export interface WorkflowTask {
  id: Id12
  run_id: RunId39
  task_type: TaskType1
  name: Name3
  assigned_agent: AssignedAgent4
  dependencies?: Dependencies2
  status?: Status7
  attempt?: Attempt4
  required?: Required2
  added_in_revision?: AddedInRevision2
  execution_ids?: ExecutionIds1
  input_data?: InputData
  output_data?: OutputData
  error?: Error8
  created_at?: CreatedAt4
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
