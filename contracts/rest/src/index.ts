export type { paths, components, operations } from './generated/api.js';
import type { components } from './generated/api.js';

export type Schemas = components['schemas'];
export * from './generated/workflows.v1.js'

export type UserId = Schemas['UserId'];
export type SessionId = Schemas['SessionId'];
export type User = Schemas['User'];
export type UserRole = Schemas['UserRole'];
export type UserStatus = Schemas['UserStatus'];
export type UserResponse = Schemas['UserResponseDto'];
export type AuthResponse = Schemas['AuthResponseDto'];
export type TokenResponse = Schemas['TokenResponseDto'];
export type RegisterRequest = Schemas['RegisterRequestDto'];
export type LoginRequest = Schemas['LoginRequestDto'];
export type LoginResponse = Schemas['LoginResponseDto'];

export type WorkspaceStatus = Schemas['WorkspaceStatus'];
export type PipelinePhase = Schemas['PipelinePhase'];
export type SeniorityLevel = Schemas['SeniorityLevel'];
export type WorkspaceId = Schemas['WorkspaceId'];
export type Workspace = Schemas['Workspace'];
export type CreateWorkspaceRequest = Schemas['CreateWorkspaceRequestDto'];
export type WorkspaceResponse = Schemas['WorkspaceResponseDto'];
export type WorkspaceProgressEvent = Schemas['WorkspaceProgressEventDto'];

export type MatchAnalysisResponse = Schemas['MatchAnalysis'];
export type StrengthEvidence = Schemas['StrengthEvidence'];
export type GapEvidence = Schemas['GapEvidence'];
export type WordingImprovement = Schemas['WordingImprovement'];
export type CompanyResearchResponse = Schemas['CompanyResearch'];
export type SourceCitation = Schemas['SourceCitation'];
export type CompanyClaim = Schemas['CompanyClaim'];
export type KnowledgeGraphResponse = Schemas['KnowledgeGraphResponse'];
export type KnowledgeSkillNode = Schemas['KnowledgeSkillNode'];
export type KnowledgeEdge = Schemas['KnowledgeEdge'];

export type LearningUnitResponse = Schemas['LearningUnit'];
export type VisualDiagram = Schemas['VisualDiagram'];
export type InterviewQuestionResponse = Schemas['InterviewQuestion'];
export type QuestionCategory = Schemas['QuestionCategory'];
export type QuestionDifficulty = Schemas['QuestionDifficulty'];

export type UserProfileResponse = Schemas['UserProfileResponseDto'];
export type LlmConfigItem = Schemas['LlmConfigItemDto'];
export type UpsertLlmConfigRequest = Schemas['UpsertLlmConfigRequestDto'];

export type WorkflowType = Schemas['WorkflowType'];
export type RunStatus = Schemas['RunStatus'];
export type TaskStatus = Schemas['TaskStatus'];
export type ExecutionStatus = Schemas['ExecutionStatus'];
export type ToolStatus = Schemas['ToolStatus'];
export type DelegationStatus = Schemas['DelegationStatus'];
export type LogSeverity = Schemas['LogSeverity'];
export type ErrorCategory = Schemas['ErrorCategory'];
export type ArtifactType = Schemas['ArtifactType'];
export type WorkflowProgress = Schemas['WorkflowProgress'];
export type WorkflowDefinition = Schemas['WorkflowDefinition'];
export type WorkflowRun = Schemas['WorkflowRun'];
export type WorkflowTask = Schemas['WorkflowTask'];
export type ExecutionNote = Schemas['ExecutionNote'];
export type ExecutionBudgets = Schemas['ExecutionBudgets'];
export type ExecutionBudgetsUsed = Schemas['ExecutionBudgetsUsed'];
export type AgentExecution = Schemas['AgentExecution'];
export type ToolInvocation = Schemas['ToolInvocation'];
export type Delegation = Schemas['Delegation'];
export type SourceLink = Schemas['SourceLink'];
export type EvidenceProvenance = Schemas['EvidenceProvenance'];
export type Artifact = Schemas['Artifact'];
export type LogEntry = Schemas['LogEntry'];
export type ExecutionEvent = Schemas['ExecutionEvent'];
export type RunsPageResponse = Schemas['RunsPageResponse'];
export type LogsPageResponse = Schemas['LogsPageResponse'];
export type EventsHistoryPageResponse = Schemas['EventsHistoryPageResponse'];
export type ToolInvocationsPageResponse = Schemas['ToolInvocationsPageResponse'];
export type ToolOutputResponse = Schemas['ToolOutputResponse'];
export type TaskGraphResponse = Schemas['TaskGraphResponse'];
export type ExecutionEventType = Schemas['ExecutionEventType'];
export type WorkflowRetryingPayload = Schemas['WorkflowRetryingPayload'];
export type DelegationGraphResponse = Schemas['DelegationGraphResponse'];
export type TriggerWorkflowRunRequest = Schemas['TriggerWorkflowRunRequestDto'];
