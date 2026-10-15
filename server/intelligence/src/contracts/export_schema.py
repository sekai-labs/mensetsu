import json
from pathlib import Path

from pydantic.json_schema import models_json_schema

from src.contracts.v1 import (
    AgentActivityEvent,
    AgentCompletedEvent,
    AgentExecution,
    AgentFailedEvent,
    AgentStartedEvent,
    Artifact,
    ArtifactCreatedEvent,
    CheckpointCreatedEvent,
    Delegation,
    DelegationAcceptedEvent,
    DelegationCompletedEvent,
    DelegationFailedEvent,
    DelegationGraphResponse,
    DelegationRequestedEvent,
    DelegationStartedEvent,
    EventsHistoryPageResponse,
    EvidenceSavedEvent,
    ExecutionResumedEvent,
    FindingSavedEvent,
    LogEntry,
    LogsPageResponse,
    PlanCreatedEvent,
    PlanRevisedEvent,
    RunsPageResponse,
    TaskAddedEvent,
    TaskBlockedEvent,
    TaskCompletedEvent,
    TaskFailedEvent,
    TaskGraphResponse,
    TaskQueuedEvent,
    TaskRemovedEvent,
    TaskRetryingEvent,
    TaskStartedEvent,
    ToolCompletedEvent,
    ToolFailedEvent,
    ToolInvocation,
    ToolInvocationsPageResponse,
    ToolOutputResponse,
    ToolStartedEvent,
    WorkerRecoveredEvent,
    WorkflowCancelledEvent,
    WorkflowCompletedEvent,
    WorkflowCreatedEvent,
    WorkflowDefinition,
    WorkflowFailedEvent,
    WorkflowRetryingEvent,
    WorkflowRun,
    WorkflowStartedEvent,
    WorkflowTask,
)


def generate_workflows_v1_schema() -> dict:
    models = [
        (WorkflowDefinition, "validation"),
        (WorkflowRun, "validation"),
        (WorkflowTask, "validation"),
        (AgentExecution, "validation"),
        (ToolInvocation, "validation"),
        (Delegation, "validation"),
        (Artifact, "validation"),
        (LogEntry, "validation"),
        (TaskGraphResponse, "validation"),
        (DelegationGraphResponse, "validation"),
        (RunsPageResponse, "validation"),
        (LogsPageResponse, "validation"),
        (EventsHistoryPageResponse, "validation"),
        (ToolInvocationsPageResponse, "validation"),
        (ToolOutputResponse, "validation"),
        (WorkflowCreatedEvent, "validation"),
        (WorkflowStartedEvent, "validation"),
        (WorkflowCompletedEvent, "validation"),
        (WorkflowFailedEvent, "validation"),
        (WorkflowCancelledEvent, "validation"),
        (WorkflowRetryingEvent, "validation"),
        (PlanCreatedEvent, "validation"),
        (PlanRevisedEvent, "validation"),
        (TaskAddedEvent, "validation"),
        (TaskRemovedEvent, "validation"),
        (TaskQueuedEvent, "validation"),
        (TaskStartedEvent, "validation"),
        (TaskCompletedEvent, "validation"),
        (TaskFailedEvent, "validation"),
        (TaskRetryingEvent, "validation"),
        (TaskBlockedEvent, "validation"),
        (AgentStartedEvent, "validation"),
        (AgentActivityEvent, "validation"),
        (AgentCompletedEvent, "validation"),
        (AgentFailedEvent, "validation"),
        (DelegationRequestedEvent, "validation"),
        (DelegationAcceptedEvent, "validation"),
        (DelegationStartedEvent, "validation"),
        (DelegationCompletedEvent, "validation"),
        (DelegationFailedEvent, "validation"),
        (ToolStartedEvent, "validation"),
        (ToolCompletedEvent, "validation"),
        (ToolFailedEvent, "validation"),
        (ArtifactCreatedEvent, "validation"),
        (FindingSavedEvent, "validation"),
        (EvidenceSavedEvent, "validation"),
        (CheckpointCreatedEvent, "validation"),
        (ExecutionResumedEvent, "validation"),
        (WorkerRecoveredEvent, "validation"),
    ]
    _, top_level_schema = models_json_schema(
        models,
        title="MensetsuWorkflowsV1Schemas",
        description="Public v1 Workflow and Multi-Agent Orchestration Contracts",
    )
    return top_level_schema


def main():

    repo_root = Path(__file__).resolve().parents[4]
    
    packages_schema_path = repo_root / "packages" / "contracts" / "schema" / "workflows.v1.json"
    packages_schema_path.parent.mkdir(parents=True, exist_ok=True)
    
    api_schema_path = repo_root / "server" / "api" / "contracts" / "schema" / "workflows.v1.json"
    api_schema_path.parent.mkdir(parents=True, exist_ok=True)

    schema = generate_workflows_v1_schema()
    serialized = json.dumps(schema, indent=2) + "\n"
    
    packages_schema_path.write_text(serialized)
    api_schema_path.write_text(serialized)
    print(f"Exported schema to {packages_schema_path} and {api_schema_path}")


if __name__ == "__main__":
    main()
