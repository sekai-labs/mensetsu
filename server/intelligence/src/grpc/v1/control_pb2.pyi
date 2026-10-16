from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class CreateRunCommand(_message.Message):
    __slots__ = ("workspace_id", "user_id", "workflow_id", "company_name", "role_title", "company_urls", "resume_text", "resume_pdf_base64", "resume_filename", "job_description", "jd_url", "seniority", "user_prompt", "workflow_mode", "llm_config_json")
    WORKSPACE_ID_FIELD_NUMBER: _ClassVar[int]
    USER_ID_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_ID_FIELD_NUMBER: _ClassVar[int]
    COMPANY_NAME_FIELD_NUMBER: _ClassVar[int]
    ROLE_TITLE_FIELD_NUMBER: _ClassVar[int]
    COMPANY_URLS_FIELD_NUMBER: _ClassVar[int]
    RESUME_TEXT_FIELD_NUMBER: _ClassVar[int]
    RESUME_PDF_BASE64_FIELD_NUMBER: _ClassVar[int]
    RESUME_FILENAME_FIELD_NUMBER: _ClassVar[int]
    JOB_DESCRIPTION_FIELD_NUMBER: _ClassVar[int]
    JD_URL_FIELD_NUMBER: _ClassVar[int]
    SENIORITY_FIELD_NUMBER: _ClassVar[int]
    USER_PROMPT_FIELD_NUMBER: _ClassVar[int]
    WORKFLOW_MODE_FIELD_NUMBER: _ClassVar[int]
    LLM_CONFIG_JSON_FIELD_NUMBER: _ClassVar[int]
    workspace_id: str
    user_id: str
    workflow_id: str
    company_name: str
    role_title: str
    company_urls: _containers.RepeatedScalarFieldContainer[str]
    resume_text: str
    resume_pdf_base64: str
    resume_filename: str
    job_description: str
    jd_url: str
    seniority: str
    user_prompt: str
    workflow_mode: str
    llm_config_json: str
    def __init__(self, workspace_id: _Optional[str] = ..., user_id: _Optional[str] = ..., workflow_id: _Optional[str] = ..., company_name: _Optional[str] = ..., role_title: _Optional[str] = ..., company_urls: _Optional[_Iterable[str]] = ..., resume_text: _Optional[str] = ..., resume_pdf_base64: _Optional[str] = ..., resume_filename: _Optional[str] = ..., job_description: _Optional[str] = ..., jd_url: _Optional[str] = ..., seniority: _Optional[str] = ..., user_prompt: _Optional[str] = ..., workflow_mode: _Optional[str] = ..., llm_config_json: _Optional[str] = ...) -> None: ...

class ResumePlanCommand(_message.Message):
    __slots__ = ("run_id", "prompt")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    PROMPT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    prompt: str
    def __init__(self, run_id: _Optional[str] = ..., prompt: _Optional[str] = ...) -> None: ...

class ListWorkflowsCommand(_message.Message):
    __slots__ = ()
    def __init__(self) -> None: ...

class ListRunsCommand(_message.Message):
    __slots__ = ("run_ids", "workspace_id", "status", "run_type", "limit", "cursor")
    RUN_IDS_FIELD_NUMBER: _ClassVar[int]
    WORKSPACE_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_FIELD_NUMBER: _ClassVar[int]
    RUN_TYPE_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    run_ids: _containers.RepeatedScalarFieldContainer[str]
    workspace_id: str
    status: str
    run_type: str
    limit: int
    cursor: str
    def __init__(self, run_ids: _Optional[_Iterable[str]] = ..., workspace_id: _Optional[str] = ..., status: _Optional[str] = ..., run_type: _Optional[str] = ..., limit: _Optional[int] = ..., cursor: _Optional[str] = ...) -> None: ...

class GetRunCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class CancelRunCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class GetTaskCommand(_message.Message):
    __slots__ = ("run_id", "task_id")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    task_id: str
    def __init__(self, run_id: _Optional[str] = ..., task_id: _Optional[str] = ...) -> None: ...

class ListAgentsCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class GetAgentCommand(_message.Message):
    __slots__ = ("run_id", "execution_id")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    execution_id: str
    def __init__(self, run_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class ListAgentToolsCommand(_message.Message):
    __slots__ = ("run_id", "execution_id", "cursor", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    execution_id: str
    cursor: str
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., execution_id: _Optional[str] = ..., cursor: _Optional[str] = ..., limit: _Optional[int] = ...) -> None: ...

class GetToolOutputCommand(_message.Message):
    __slots__ = ("run_id", "tool_invocation_id", "offset", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    TOOL_INVOCATION_ID_FIELD_NUMBER: _ClassVar[int]
    OFFSET_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    tool_invocation_id: str
    offset: int
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., tool_invocation_id: _Optional[str] = ..., offset: _Optional[int] = ..., limit: _Optional[int] = ...) -> None: ...

class ListDelegationsCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class ListArtifactsCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class GetLogsCommand(_message.Message):
    __slots__ = ("run_id", "cursor", "limit", "severity", "task_id", "execution_id")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    CURSOR_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    SEVERITY_FIELD_NUMBER: _ClassVar[int]
    TASK_ID_FIELD_NUMBER: _ClassVar[int]
    EXECUTION_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    cursor: str
    limit: int
    severity: str
    task_id: str
    execution_id: str
    def __init__(self, run_id: _Optional[str] = ..., cursor: _Optional[str] = ..., limit: _Optional[int] = ..., severity: _Optional[str] = ..., task_id: _Optional[str] = ..., execution_id: _Optional[str] = ...) -> None: ...

class GetTaskGraphCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class GetDelegationGraphCommand(_message.Message):
    __slots__ = ("run_id",)
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    def __init__(self, run_id: _Optional[str] = ...) -> None: ...

class GetEventsHistoryCommand(_message.Message):
    __slots__ = ("run_id", "after_seq", "limit")
    RUN_ID_FIELD_NUMBER: _ClassVar[int]
    AFTER_SEQ_FIELD_NUMBER: _ClassVar[int]
    LIMIT_FIELD_NUMBER: _ClassVar[int]
    run_id: str
    after_seq: int
    limit: int
    def __init__(self, run_id: _Optional[str] = ..., after_seq: _Optional[int] = ..., limit: _Optional[int] = ...) -> None: ...

class PingControlCommand(_message.Message):
    __slots__ = ("timestamp",)
    TIMESTAMP_FIELD_NUMBER: _ClassVar[int]
    timestamp: int
    def __init__(self, timestamp: _Optional[int] = ...) -> None: ...

class ControlCommand(_message.Message):
    __slots__ = ("correlation_id", "create_run", "resume_plan", "list_workflows", "list_runs", "get_run", "cancel_run", "get_task", "list_agents", "get_agent", "list_agent_tools", "get_tool_output", "list_delegations", "list_artifacts", "get_logs", "get_task_graph", "get_delegation_graph", "get_events_history", "ping")
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    CREATE_RUN_FIELD_NUMBER: _ClassVar[int]
    RESUME_PLAN_FIELD_NUMBER: _ClassVar[int]
    LIST_WORKFLOWS_FIELD_NUMBER: _ClassVar[int]
    LIST_RUNS_FIELD_NUMBER: _ClassVar[int]
    GET_RUN_FIELD_NUMBER: _ClassVar[int]
    CANCEL_RUN_FIELD_NUMBER: _ClassVar[int]
    GET_TASK_FIELD_NUMBER: _ClassVar[int]
    LIST_AGENTS_FIELD_NUMBER: _ClassVar[int]
    GET_AGENT_FIELD_NUMBER: _ClassVar[int]
    LIST_AGENT_TOOLS_FIELD_NUMBER: _ClassVar[int]
    GET_TOOL_OUTPUT_FIELD_NUMBER: _ClassVar[int]
    LIST_DELEGATIONS_FIELD_NUMBER: _ClassVar[int]
    LIST_ARTIFACTS_FIELD_NUMBER: _ClassVar[int]
    GET_LOGS_FIELD_NUMBER: _ClassVar[int]
    GET_TASK_GRAPH_FIELD_NUMBER: _ClassVar[int]
    GET_DELEGATION_GRAPH_FIELD_NUMBER: _ClassVar[int]
    GET_EVENTS_HISTORY_FIELD_NUMBER: _ClassVar[int]
    PING_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    create_run: CreateRunCommand
    resume_plan: ResumePlanCommand
    list_workflows: ListWorkflowsCommand
    list_runs: ListRunsCommand
    get_run: GetRunCommand
    cancel_run: CancelRunCommand
    get_task: GetTaskCommand
    list_agents: ListAgentsCommand
    get_agent: GetAgentCommand
    list_agent_tools: ListAgentToolsCommand
    get_tool_output: GetToolOutputCommand
    list_delegations: ListDelegationsCommand
    list_artifacts: ListArtifactsCommand
    get_logs: GetLogsCommand
    get_task_graph: GetTaskGraphCommand
    get_delegation_graph: GetDelegationGraphCommand
    get_events_history: GetEventsHistoryCommand
    ping: PingControlCommand
    def __init__(self, correlation_id: _Optional[str] = ..., create_run: _Optional[_Union[CreateRunCommand, _Mapping]] = ..., resume_plan: _Optional[_Union[ResumePlanCommand, _Mapping]] = ..., list_workflows: _Optional[_Union[ListWorkflowsCommand, _Mapping]] = ..., list_runs: _Optional[_Union[ListRunsCommand, _Mapping]] = ..., get_run: _Optional[_Union[GetRunCommand, _Mapping]] = ..., cancel_run: _Optional[_Union[CancelRunCommand, _Mapping]] = ..., get_task: _Optional[_Union[GetTaskCommand, _Mapping]] = ..., list_agents: _Optional[_Union[ListAgentsCommand, _Mapping]] = ..., get_agent: _Optional[_Union[GetAgentCommand, _Mapping]] = ..., list_agent_tools: _Optional[_Union[ListAgentToolsCommand, _Mapping]] = ..., get_tool_output: _Optional[_Union[GetToolOutputCommand, _Mapping]] = ..., list_delegations: _Optional[_Union[ListDelegationsCommand, _Mapping]] = ..., list_artifacts: _Optional[_Union[ListArtifactsCommand, _Mapping]] = ..., get_logs: _Optional[_Union[GetLogsCommand, _Mapping]] = ..., get_task_graph: _Optional[_Union[GetTaskGraphCommand, _Mapping]] = ..., get_delegation_graph: _Optional[_Union[GetDelegationGraphCommand, _Mapping]] = ..., get_events_history: _Optional[_Union[GetEventsHistoryCommand, _Mapping]] = ..., ping: _Optional[_Union[PingControlCommand, _Mapping]] = ...) -> None: ...

class ControlDataEnvelope(_message.Message):
    __slots__ = ("correlation_id", "status_code", "error_message", "is_end_of_stream", "payload_json", "binary_chunk")
    CORRELATION_ID_FIELD_NUMBER: _ClassVar[int]
    STATUS_CODE_FIELD_NUMBER: _ClassVar[int]
    ERROR_MESSAGE_FIELD_NUMBER: _ClassVar[int]
    IS_END_OF_STREAM_FIELD_NUMBER: _ClassVar[int]
    PAYLOAD_JSON_FIELD_NUMBER: _ClassVar[int]
    BINARY_CHUNK_FIELD_NUMBER: _ClassVar[int]
    correlation_id: str
    status_code: int
    error_message: str
    is_end_of_stream: bool
    payload_json: str
    binary_chunk: bytes
    def __init__(self, correlation_id: _Optional[str] = ..., status_code: _Optional[int] = ..., error_message: _Optional[str] = ..., is_end_of_stream: _Optional[bool] = ..., payload_json: _Optional[str] = ..., binary_chunk: _Optional[bytes] = ...) -> None: ...
