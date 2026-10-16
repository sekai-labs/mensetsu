use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status};

use mensetsu_api::interfaces::grpc::proto::control::data_control_service_server::{
    DataControlService, DataControlServiceServer,
};
use mensetsu_api::interfaces::grpc::proto::control::{
    control_command, ControlCommand, ControlDataEnvelope,
};
use mensetsu_api::interfaces::grpc::proto::streaming::workflow_event_stream_service_server::{
    WorkflowEventStreamService, WorkflowEventStreamServiceServer,
};
use mensetsu_api::interfaces::grpc::proto::streaming::{
    EventStreamClientMessage, ExecutionEventFrame,
};

#[derive(Default, Clone)]
pub struct RecordedGrpcState {
    pub received_commands: Arc<parking_lot::Mutex<Vec<String>>>,
}

pub struct DynamicMockControlService {
    pub workspace_id: String,
    pub state: RecordedGrpcState,
}

#[tonic::async_trait]
impl DataControlService for DynamicMockControlService {
    type StreamControlStream = Pin<Box<dyn Stream<Item = Result<ControlDataEnvelope, Status>> + Send + 'static>>;

    async fn stream_control(
        &self,
        request: Request<tonic::Streaming<ControlCommand>>,
    ) -> Result<Response<Self::StreamControlStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(128);
        let ws_id = self.workspace_id.clone();
        let rec = self.state.received_commands.clone();

        tokio::spawn(async move {
            while let Some(Ok(cmd)) = in_stream.next().await {
                let corr_id = cmd.correlation_id;
                let cmd_inner = match cmd.command {
                    Some(c) => c,
                    None => continue,
                };

                let (status_code, payload_json, error_message) = match cmd_inner {
                    control_command::Command::ListWorkflows(_) => {
                        rec.lock().push("list_workflows".into());
                        (200, "[]".into(), String::new())
                    }
                    control_command::Command::ListRuns(req) => {
                        rec.lock().push("list_runs".into());
                        let mut items: Vec<serde_json::Value> = req
                            .run_ids
                            .into_iter()
                            .map(|id| run_json_helper(&id, &ws_id, "running"))
                            .collect();
                        items.push(run_json_helper("run_decoy_not_requested", "someone_else", "running"));
                        (
                            200,
                            serde_json::to_string(&serde_json::json!({
                                "items": items,
                                "next_cursor": null
                            }))
                            .unwrap(),
                            String::new(),
                        )
                    }
                    control_command::Command::GetRun(req) => {
                        rec.lock().push(format!("get_run:{}", req.run_id));
                        (200, serde_json::to_string(&run_json_helper(&req.run_id, &ws_id, "running")).unwrap(), String::new())
                    }
                    control_command::Command::CreateRun(req) => {
                        rec.lock().push("create_run".into());
                        (201, serde_json::to_string(&run_json_helper("run_created_by_trigger", &req.workspace_id, "pending")).unwrap(), String::new())
                    }
                    control_command::Command::CancelRun(req) => {
                        rec.lock().push(format!("cancel_run:{}", req.run_id));
                        (200, serde_json::to_string(&run_json_helper(&req.run_id, "x", "cancelled")).unwrap(), String::new())
                    }
                    control_command::Command::GetTaskGraph(_) => {
                        rec.lock().push("get_task_graph".into());
                        (200, "{\"nodes\":[],\"edges\":[]}".into(), String::new())
                    }
                    control_command::Command::GetDelegationGraph(_) => {
                        rec.lock().push("get_delegation_graph".into());
                        (200, "{\"nodes\":[],\"edges\":[]}".into(), String::new())
                    }
                    control_command::Command::GetLogs(_) => {
                        rec.lock().push("get_logs".into());
                        (200, "{\"items\":[],\"next_cursor\":null}".into(), String::new())
                    }
                    control_command::Command::ListDelegations(_) => {
                        rec.lock().push("list_delegations".into());
                        (200, "[]".into(), String::new())
                    }
                    control_command::Command::ListArtifacts(_) => {
                        rec.lock().push("list_artifacts".into());
                        (200, "[]".into(), String::new())
                    }
                    control_command::Command::ListAgents(_) => {
                        rec.lock().push("list_agents".into());
                        (200, "[]".into(), String::new())
                    }
                    control_command::Command::GetEventsHistory(_) => {
                        rec.lock().push("get_events_history".into());
                        (200, "{\"events\":[],\"next_after_seq\":null,\"last_seq\":0}".into(), String::new())
                    }
                    control_command::Command::GetTask(_) => {
                        rec.lock().push("get_task".into());
                        (404, "{}".into(), "Task not found".into())
                    }
                    control_command::Command::GetAgent(_) => {
                        rec.lock().push("get_agent".into());
                        (404, "{}".into(), "Agent not found".into())
                    }
                    control_command::Command::ListAgentTools(_) => {
                        rec.lock().push("list_agent_tools".into());
                        (404, "{}".into(), "Agent tools not found".into())
                    }
                    control_command::Command::GetToolOutput(_) => {
                        rec.lock().push("get_tool_output".into());
                        (404, "{}".into(), "Tool output not found".into())
                    }
                    control_command::Command::ResumePlan(req) => {
                        rec.lock().push("resume_plan".into());
                        (200, serde_json::to_string(&serde_json::json!({"status": "plan_resumed", "run_id": req.run_id})).unwrap(), String::new())
                    }
                    control_command::Command::Ping(_) => {
                        rec.lock().push("ping".into());
                        (200, "{}".into(), String::new())
                    }
                };

                let env = ControlDataEnvelope {
                    correlation_id: corr_id,
                    status_code,
                    error_message,
                    is_end_of_stream: true,
                    payload_json,
                    binary_chunk: vec![],
                };
                let _ = tx.send(Ok(env)).await;
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

pub struct DynamicMockStreamingService;

#[tonic::async_trait]
impl WorkflowEventStreamService for DynamicMockStreamingService {
    type StreamWorkflowEventsStream =
        Pin<Box<dyn Stream<Item = Result<ExecutionEventFrame, Status>> + Send + 'static>>;

    async fn stream_workflow_events(
        &self,
        request: Request<tonic::Streaming<EventStreamClientMessage>>,
    ) -> Result<Response<Self::StreamWorkflowEventsStream>, Status> {
        let mut in_stream = request.into_inner();
        let (tx, rx) = mpsc::channel(16);

        tokio::spawn(async move {
            let mut run_id = "mock_run".to_string();
            if let Some(Ok(msg)) = in_stream.next().await {
                if let Some(mensetsu_api::interfaces::grpc::proto::streaming::event_stream_client_message::Message::Subscribe(sub)) = msg.message {
                    run_id = sub.run_id;
                }
            }

            let frames = vec![
                ExecutionEventFrame {
                    seq: 3,
                    run_id: run_id.clone(),
                    event_type: "task.started".into(),
                    task_id: "t1".into(),
                    execution_id: "".into(),
                    payload_json: "{\"sequence\":3,\"type\":\"task.started\",\"payload\":{\"task_id\":\"t1\"}}".into(),
                    timestamp: "2026-10-12T10:00:00Z".into(),
                    is_terminal: false,
                },
                ExecutionEventFrame {
                    seq: 4,
                    run_id: run_id.clone(),
                    event_type: "agent.started".into(),
                    task_id: "".into(),
                    execution_id: "".into(),
                    payload_json: "{\"sequence\":4,\"payload\":{\"line\":\"a\\nb\"}}".into(),
                    timestamp: "2026-10-12T10:00:01Z".into(),
                    is_terminal: false,
                },
                ExecutionEventFrame {
                    seq: 5,
                    run_id: run_id.clone(),
                    event_type: "workflow.completed".into(),
                    task_id: "".into(),
                    execution_id: "".into(),
                    payload_json: "{\"sequence\":5}".into(),
                    timestamp: "2026-10-12T10:00:02Z".into(),
                    is_terminal: true,
                },
            ];

            for frame in frames {
                let _ = tx.send(Ok(frame)).await;
            }
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}

pub fn run_json_helper(id: &str, workspace_id: &str, status: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "workflow_id": "interview-preparation-static",
        "workflow_name": "Static Interview Preparation Workflow",
        "workspace_id": workspace_id,
        "user_id": "ignored-by-rust",
        "type": "static",
        "status": status,
        "current_phase": "research",
        "plan_revision": 1,
        "progress": {"total_tasks": 0, "pending": 0, "queued": 0, "running": 0, "completed": 0,
                     "failed": 0, "retrying": 0, "blocked": 0, "cancelled": 0, "skipped": 0,
                     "percentage": null, "estimate": false, "phase_index": null, "phase_count": null},
        "last_seq": 3,
        "started_at": null, "completed_at": null, "duration_ms": null,
        "created_at": "2026-10-12T10:00:00Z", "updated_at": "2026-10-12T10:00:00Z",
        "error_message": null
    })
}
#[allow(dead_code)]
pub async fn start_mock_grpc_server(
    _run_json: String,
) -> (String, tokio::task::JoinHandle<()>) {
    let (url, _state, handle) = start_dynamic_mock_grpc_server("ws_test".into()).await;
    (url, handle)
}

pub async fn start_dynamic_mock_grpc_server(
    workspace_id: String,
) -> (String, RecordedGrpcState, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let state = RecordedGrpcState::default();
    let control_svc = DynamicMockControlService {
        workspace_id,
        state: state.clone(),
    };
    let streaming_svc = DynamicMockStreamingService;

    let handle = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(DataControlServiceServer::new(control_svc))
            .add_service(WorkflowEventStreamServiceServer::new(streaming_svc))
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    (format!("http://{addr}"), state, handle)
}
