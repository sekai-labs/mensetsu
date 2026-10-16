pub mod proto {
    pub mod control {
        tonic::include_proto!("mensetsu.control.v1");
    }
    pub mod streaming {
        tonic::include_proto!("mensetsu.streaming.v1");
    }
    pub mod health {
        tonic::include_proto!("grpc.health.v1");
    }
}

use std::sync::Arc;
use std::time::Duration;

use axum::response::sse::Event;
use dashmap::DashMap;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::metadata::{Ascii, MetadataValue};
use tonic::transport::Endpoint;
use uuid::Uuid;

use crate::interfaces::http::workflow_dtos::*;
use crate::shared::config::AppConfig;
use crate::shared::error::{AppError, Result};

pub use proto::control::*;
pub use proto::streaming::*;

#[derive(Debug, Clone, Serialize)]
pub struct CreateRunPayload {
    pub workspace_id: String,
    pub user_id: String,
    pub workflow_id: String,
    pub company_name: String,
    pub role_title: String,
    pub company_urls: Vec<String>,
    pub resume_text: String,
    pub resume_pdf_base64: Option<String>,
    pub resume_filename: Option<String>,
    pub job_description: String,
    pub jd_url: Option<String>,
    pub seniority: Option<String>,
    pub user_prompt: Option<String>,
    pub workflow_mode: Option<String>,
    pub llm_config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResumePlanPayload {
    pub prompt: String,
}

#[derive(Clone)]
pub struct GrpcClient {
    command_tx: mpsc::Sender<ControlCommand>,
    pending_requests: Arc<DashMap<String, mpsc::Sender<ControlDataEnvelope>>>,
    streaming_endpoint: Endpoint,
    auth_header_value: MetadataValue<Ascii>,
}

impl GrpcClient {
    pub async fn connect(config: &AppConfig) -> Result<Self> {
        config.validate().map_err(AppError::Internal)?;

        let auth_str = format!("Bearer {}", config.internal_service_token);
        let auth_header_value: MetadataValue<Ascii> = auth_str
            .parse()
            .map_err(|e| AppError::Internal(format!("Invalid INTERNAL_SERVICE_TOKEN: {e}")))?;

        let endpoint_str = if config.intelligence_grpc_url.starts_with("http://")
            || config.intelligence_grpc_url.starts_with("https://")
        {
            config.intelligence_grpc_url.clone()
        } else {
            format!("http://{}", config.intelligence_grpc_url)
        };

        let endpoint = Endpoint::from_shared(endpoint_str.clone())
            .map_err(|e| AppError::Internal(format!("Invalid INTELLIGENCE_GRPC_ENDPOINT: {e}")))?
            .connect_timeout(Duration::from_secs(10));

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to connect to Intelligence gRPC service: {e}")))?;

        let (command_tx, command_rx) = mpsc::channel::<ControlCommand>(256);
        let pending_requests: Arc<DashMap<String, mpsc::Sender<ControlDataEnvelope>>> =
            Arc::new(DashMap::new());

        let mut control_client =
            proto::control::data_control_service_client::DataControlServiceClient::with_interceptor(
                channel,
                {
                    let auth_val = auth_header_value.clone();
                    move |mut req: tonic::Request<()>| {
                        req.metadata_mut().insert("authorization", auth_val.clone());
                        Ok(req)
                    }
                },
            );

        let out_stream = ReceiverStream::new(command_rx);
        let response_stream = control_client
            .stream_control(tonic::Request::new(out_stream))
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to establish DataControl gRPC stream: {e}")))?
            .into_inner();

        // Spawn background listener task to demultiplex control envelopes
        let pending_map = pending_requests.clone();
        tokio::spawn(async move {
            let mut stream = response_stream;
            while let Some(res) = stream.next().await {
                match res {
                    Ok(envelope) => {
                        let corr_id = envelope.correlation_id.clone();
                        let is_end = envelope.is_end_of_stream;
                        if let Some(entry) = pending_map.get(&corr_id) {
                            let _ = entry.value().send(envelope).await;
                        }
                        if is_end {
                            pending_map.remove(&corr_id);
                        }
                    }
                    Err(status) => {
                        tracing::error!(status = %status, "Control response stream terminated with error");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            command_tx,
            pending_requests,
            streaming_endpoint: endpoint,
            auth_header_value,
        })
    }

    async fn send_command(
        &self,
        command_oneof: control_command::Command,
    ) -> Result<ControlDataEnvelope> {
        let correlation_id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel::<ControlDataEnvelope>(32);
        self.pending_requests.insert(correlation_id.clone(), tx);

        let cmd = ControlCommand {
            correlation_id: correlation_id.clone(),
            command: Some(command_oneof),
        };

        if let Err(e) = self.command_tx.send(cmd).await {
            self.pending_requests.remove(&correlation_id);
            return Err(AppError::ExternalService(format!(
                "Failed to send command to DataControl stream: {e}"
            )));
        }

        let mut accumulated_payload = String::new();
        let mut last_status = 200;
        let mut last_binary = Vec::new();

        let result = tokio::time::timeout(Duration::from_secs(30), async {
            while let Some(envelope) = rx.recv().await {
                last_status = envelope.status_code;
                if !envelope.payload_json.is_empty() {
                    accumulated_payload.push_str(&envelope.payload_json);
                }
                if !envelope.binary_chunk.is_empty() {
                    last_binary.extend_from_slice(&envelope.binary_chunk);
                }
                if envelope.is_end_of_stream {
                    return Ok(ControlDataEnvelope {
                        correlation_id: correlation_id.clone(),
                        status_code: last_status,
                        error_message: envelope.error_message,
                        is_end_of_stream: true,
                        payload_json: accumulated_payload,
                        binary_chunk: last_binary,
                    });
                }
            }
            Err(AppError::ExternalService("Control stream closed without terminal envelope".into()))
        }).await;

        self.pending_requests.remove(&correlation_id);

        match result {
            Ok(Ok(envelope)) => {
                if envelope.status_code == 200 || envelope.status_code == 201 {
                    Ok(envelope)
                } else if envelope.status_code == 404 {
                    Err(AppError::NotFound(envelope.error_message))
                } else if envelope.status_code == 400 {
                    Err(AppError::Validation(envelope.error_message))
                } else if envelope.status_code == 503 {
                    Err(AppError::ServiceUnavailable(envelope.error_message))
                } else {
                    Err(AppError::ExternalService(envelope.error_message))
                }
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(AppError::ExternalService("Timed out waiting for DataControl response".into())),
        }
    }

    async fn execute_json<T: DeserializeOwned>(
        &self,
        command_oneof: control_command::Command,
    ) -> Result<T> {
        let envelope = self.send_command(command_oneof).await?;
        serde_json::from_str(&envelope.payload_json).map_err(|e| {
            tracing::error!(error = %e, payload = %envelope.payload_json, "Payload did not match expected contract schema");
            AppError::ExternalService(format!("Intelligence payload did not match contract: {e}"))
        })
    }

    pub async fn list_workflows(&self) -> Result<Vec<WorkflowDefinition>> {
        self.execute_json(control_command::Command::ListWorkflows(ListWorkflowsCommand {})).await
    }

    pub async fn list_runs(
        &self,
        run_ids: Vec<String>,
        workspace_id: Option<String>,
        status: Option<String>,
        run_type: Option<String>,
        limit: u32,
        cursor: Option<String>,
    ) -> Result<RunsPageResponse> {
        self.execute_json(control_command::Command::ListRuns(ListRunsCommand {
            run_ids,
            workspace_id,
            status,
            run_type,
            limit,
            cursor,
        }))
        .await
    }

    pub async fn get_run(&self, run_id: &str) -> Result<WorkflowRun> {
        self.execute_json(control_command::Command::GetRun(GetRunCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn create_run(
        &self,
        payload: &CreateRunPayload,
        _idempotency_key: Option<&str>,
    ) -> Result<WorkflowRun> {
        let llm_config_json = payload
            .llm_config
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default());

        self.execute_json(control_command::Command::CreateRun(CreateRunCommand {
            workspace_id: payload.workspace_id.clone(),
            user_id: payload.user_id.clone(),
            workflow_id: payload.workflow_id.clone(),
            company_name: payload.company_name.clone(),
            role_title: payload.role_title.clone(),
            company_urls: payload.company_urls.clone(),
            resume_text: payload.resume_text.clone(),
            resume_pdf_base64: payload.resume_pdf_base64.clone(),
            resume_filename: payload.resume_filename.clone(),
            job_description: payload.job_description.clone(),
            jd_url: payload.jd_url.clone(),
            seniority: payload.seniority.clone(),
            user_prompt: payload.user_prompt.clone(),
            workflow_mode: payload.workflow_mode.clone(),
            llm_config_json,
        }))
        .await
    }

    pub async fn resume_plan(
        &self,
        run_id: &str,
        payload: &ResumePlanPayload,
    ) -> Result<serde_json::Value> {
        self.execute_json(control_command::Command::ResumePlan(ResumePlanCommand {
            run_id: run_id.to_string(),
            prompt: payload.prompt.clone(),
        }))
        .await
    }

    pub async fn cancel_run(&self, run_id: &str) -> Result<WorkflowRun> {
        self.execute_json(control_command::Command::CancelRun(CancelRunCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn get_task(&self, run_id: &str, task_id: &str) -> Result<WorkflowTask> {
        self.execute_json(control_command::Command::GetTask(GetTaskCommand {
            run_id: run_id.to_string(),
            task_id: task_id.to_string(),
        }))
        .await
    }

    pub async fn list_agents(&self, run_id: &str) -> Result<Vec<AgentExecution>> {
        self.execute_json(control_command::Command::ListAgents(ListAgentsCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn get_agent(&self, run_id: &str, execution_id: &str) -> Result<AgentExecution> {
        self.execute_json(control_command::Command::GetAgent(GetAgentCommand {
            run_id: run_id.to_string(),
            execution_id: execution_id.to_string(),
        }))
        .await
    }

    pub async fn list_agent_tools(
        &self,
        run_id: &str,
        execution_id: &str,
        cursor: Option<String>,
        limit: u32,
    ) -> Result<ToolInvocationsPageResponse> {
        self.execute_json(control_command::Command::ListAgentTools(ListAgentToolsCommand {
            run_id: run_id.to_string(),
            execution_id: execution_id.to_string(),
            cursor,
            limit,
        }))
        .await
    }

    pub async fn get_tool_output(
        &self,
        run_id: &str,
        tool_invocation_id: &str,
        offset: u32,
        limit: u32,
    ) -> Result<ToolOutputResponse> {
        self.execute_json(control_command::Command::GetToolOutput(GetToolOutputCommand {
            run_id: run_id.to_string(),
            tool_invocation_id: tool_invocation_id.to_string(),
            offset,
            limit,
        }))
        .await
    }

    pub async fn list_delegations(&self, run_id: &str) -> Result<Vec<Delegation>> {
        self.execute_json(control_command::Command::ListDelegations(ListDelegationsCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn list_artifacts(&self, run_id: &str) -> Result<Vec<Artifact>> {
        self.execute_json(control_command::Command::ListArtifacts(ListArtifactsCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn get_logs(
        &self,
        run_id: &str,
        cursor: Option<String>,
        limit: u32,
        severity: Option<String>,
        task_id: Option<String>,
        execution_id: Option<String>,
    ) -> Result<LogsPageResponse> {
        self.execute_json(control_command::Command::GetLogs(GetLogsCommand {
            run_id: run_id.to_string(),
            cursor,
            limit,
            severity,
            task_id,
            execution_id,
        }))
        .await
    }

    pub async fn get_task_graph(&self, run_id: &str) -> Result<TaskGraphResponse> {
        self.execute_json(control_command::Command::GetTaskGraph(GetTaskGraphCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn get_delegation_graph(&self, run_id: &str) -> Result<DelegationGraphResponse> {
        self.execute_json(control_command::Command::GetDelegationGraph(GetDelegationGraphCommand {
            run_id: run_id.to_string(),
        }))
        .await
    }

    pub async fn get_events_history(
        &self,
        run_id: &str,
        after_seq: i64,
        limit: u32,
    ) -> Result<EventsHistoryPageResponse> {
        self.execute_json(control_command::Command::GetEventsHistory(GetEventsHistoryCommand {
            run_id: run_id.to_string(),
            after_seq,
            limit,
        }))
        .await
    }

    pub async fn stream_run_events(
        &self,
        run_id: String,
        after_seq: u64,
    ) -> Result<impl Stream<Item = std::result::Result<Event, std::convert::Infallible>>> {
        let channel = self
            .streaming_endpoint
            .connect()
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to connect for streaming: {e}")))?;

        let mut stream_client =
            proto::streaming::workflow_event_stream_service_client::WorkflowEventStreamServiceClient::with_interceptor(
                channel,
                {
                    let auth_val = self.auth_header_value.clone();
                    move |mut req: tonic::Request<()>| {
                        req.metadata_mut().insert("authorization", auth_val.clone());
                        Ok(req)
                    }
                },
            );

        let (sub_tx, sub_rx) = mpsc::channel::<EventStreamClientMessage>(16);

        // Send initial subscribe
        let sub_msg = EventStreamClientMessage {
            message: Some(event_stream_client_message::Message::Subscribe(
                SubscribeRunMessage { run_id, after_seq },
            )),
        };
        let _ = sub_tx.send(sub_msg).await;

        let request_stream = ReceiverStream::new(sub_rx);
        let incoming_stream = stream_client
            .stream_workflow_events(tonic::Request::new(request_stream))
            .await
            .map_err(|e| AppError::ExternalService(format!("Failed to open WorkflowEventStream: {e}")))?
            .into_inner();

        let sse_stream = async_stream::stream! {
            let mut stream = incoming_stream;
            let _keepalive_tx = sub_tx; // hold channel open
            while let Some(item) = stream.next().await {
                match item {
                    Ok(frame) => {
                        let is_term = frame.is_terminal;
                        let event = Event::default()
                            .id(frame.seq.to_string())
                            .event(frame.event_type)
                            .data(frame.payload_json);
                        yield Ok(event);
                        if is_term {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::debug!(error = %e, "gRPC EventStream ended with error");
                        break;
                    }
                }
            }
        };

        Ok(sse_stream)
    }
}
