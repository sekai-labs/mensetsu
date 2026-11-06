mod mock_grpc_server;

use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use mensetsu_api::{
    contexts::{
        identity::{
            application::IdentityService,
            domain::{hasher::Argon2PasswordHasher, token::TokenService},
            infrastructure::{SurrealAuthSessionRepository, SurrealUserRepository},
        },
        workflow::{application::WorkflowService, infrastructure::SurrealWorkflowRunRepository},
        workspace::{
            application::WorkspaceService,
            infrastructure::{db::DatabasePool, SurrealWorkspaceRepository},
        },
    },
    interfaces::http::{create_router, workspaces::AppState},
    shared::config::AppConfig,
};
use serde_json::{json, Value};
use tower::ServiceExt;

const INTERNAL_TOKEN: &str = "test-internal-token-7f3a";

async fn app_with_upstream(intelligence_grpc_url: &str) -> (Router, DatabasePool) {
    let mut config = AppConfig::from_env();
    config.intelligence_grpc_url = intelligence_grpc_url.to_string();
    config.internal_service_token = INTERNAL_TOKEN.to_string();

    let db = DatabasePool::connect(&config).await.expect("SurrealDB must be reachable");
    let ws_repo = Arc::new(SurrealWorkspaceRepository::new((*db.client).clone()));
    let grpc_client = Arc::new(
        mensetsu_api::interfaces::grpc::GrpcClient::connect(&config)
            .await
            .expect("GrpcClient must connect"),
    );
    let storage = Arc::new(
        mensetsu_api::contexts::workspace::infrastructure::storage::ResumeObjectStore::from_config(&config)
            .await
            .expect("S3 signer should initialize"),
    );
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        identity_service: Arc::new(IdentityService::new(
            Arc::new(SurrealUserRepository::new(db.client.clone())),
            Arc::new(SurrealAuthSessionRepository::new(db.client.clone())),
            Arc::new(Argon2PasswordHasher::new()),
            TokenService::new(&config.jwt_secret, config.access_token_ttl_seconds, config.refresh_token_ttl_seconds),
        )),
        workspace_service: Arc::new(WorkspaceService::new(ws_repo.clone())),
        workflow_service: Arc::new(WorkflowService::new(
            Arc::new(SurrealWorkflowRunRepository::new((*db.client).clone())),
            ws_repo,
        )),
        grpc_client,
        storage,
    };
    (create_router(state), db)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap().to_vec();
    (status, body)
}

async fn register(app: &Router) -> String {
    let email = format!("wf_{}@example.com", uuid::Uuid::new_v4());
    let (status, body) = send(
        app,
        Request::post("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(json!({"email": email, "password": "Password123!", "name": "WF"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice::<Value>(&body).unwrap()["access_token"].as_str().unwrap().to_string()
}

fn authed(method: &'static str, uri: &str, token: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
}

async fn seed_workspace(_db: &DatabasePool, token: &str, app: &Router) -> String {
    let (status, body) = send(
        app,
        authed("POST", "/api/v1/workspaces", token)
            .header("content-type", "application/json")
            .body(Body::from(
                json!({
                    "role_title": "Backend Architect",
                    "company_name": "Acme",
                    "job_description": "Rust and distributed systems"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice::<Value>(&body).unwrap()["id"].as_str().unwrap().to_string()
}

async fn authenticated_user_id(app: &Router, token: &str) -> String {
    let (status, body) = send(
        app,
        authed("GET", "/api/v1/auth/me", token).body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice::<Value>(&body).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn seed_token_usage(
    db: &DatabasePool,
    run_id: &str,
    workspace_id: &str,
    user_id: &str,
    event_id: &str,
    input_tokens: u64,
    output_tokens: u64,
) {
    let clean_workspace = workspace_id.strip_prefix("workspace:").unwrap_or(workspace_id);
    db.client
        .query(
            "CREATE type::record('workflow_run', $run_id) SET
                workspace_id = type::record('workspace', $workspace_id),
                user_id = $user_id,
                workflow_id = 'interview-preparation-static',
                workflow_name = 'Static Interview Preparation Workflow',
                type = 'static',
                status = 'completed',
                current_phase = 'ready',
                last_seq = 1;
             CREATE type::record('execution_event', $event_record_id) SET
                event_id = $event_id,
                run_id = type::record('workflow_run', $run_id),
                seq = 1,
                type = 'agent.model_usage',
                execution_id = 'exec_usage',
                payload = $payload;",
        )
        .bind(("run_id", run_id.to_owned()))
        .bind(("workspace_id", clean_workspace.to_owned()))
        .bind(("user_id", user_id.to_owned()))
        .bind(("event_record_id", format!("event_{event_id}")))
        .bind(("event_id", event_id.to_owned()))
        .bind(("payload", json!({
            "agent_id": "technical-research",
            "model_provider": "openrouter",
            "model_name": "example/model",
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "total_tokens": input_tokens + output_tokens,
        })))
        .await
        .unwrap()
        .check()
        .unwrap();
}

async fn seed_run(db: &DatabasePool, run_id: &str, ws_id: &str, created_at: &str) {
    let clean_ws = ws_id.strip_prefix("workspace:").unwrap_or(ws_id);
    let clean_run = run_id.strip_prefix("workflow_run:").unwrap_or(run_id);
    db.client
        .query(
            "CREATE type::record('workflow_run', $id) SET workspace_id = type::record('workspace', $ws),
             user_id = 'ledger-user', workflow_id = 'interview-preparation-static',
             workflow_name = 'Static Interview Preparation Workflow', type = 'static', status = 'running',
             current_phase = 'research', created_at = <datetime> $created_at;",
        )
        .bind(("id", clean_run.to_string()))
        .bind(("ws", clean_ws.to_string()))
        .bind(("created_at", created_at.to_string()))
        .await
        .unwrap()
        .check()
        .unwrap();
}

fn run_scoped_routes(run_id: &str) -> Vec<(&'static str, String)> {
    vec![
        ("GET", format!("/api/v1/workflows/runs/{run_id}")),
        ("POST", format!("/api/v1/workflows/runs/{run_id}/cancel")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/graph")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/logs")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/delegations")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/artifacts")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/agents")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/events/history")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/tasks/task_1")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/agents/exec_1")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/agents/exec_1/tools")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/tools/tool_1/output")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/events")),
    ]
}

#[tokio::test]
async fn foreign_and_unknown_runs_are_404_on_every_route_and_never_forwarded() {
    let (url, state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;
    let unknown = format!("run_{}", uuid::Uuid::new_v4());


    state.received_commands.lock().clear();

    for (method, uri) in run_scoped_routes(&run_a) {
        let (status, _body) = send(&app, authed(method, &uri, &token_b).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "B: {method} {uri}");
    }
    for (method, uri) in run_scoped_routes(&unknown) {
        let (status, _body) = send(&app, authed(method, &uri, &token_b).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "unknown: {method} {uri}");
    }


    assert!(
        state.received_commands.lock().is_empty(),
        "nothing may reach gRPC upstream for unauthorized runs: {:?}",
        state.received_commands.lock()
    );


    let (status, body) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));

    let (status, body) = send(&app, authed("POST", &format!("/api/v1/workflows/runs/{run_a}/cancel"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(serde_json::from_slice::<Value>(&body).unwrap()["status"], "cancelled");

    let cmds = state.received_commands.lock().clone();
    assert!(cmds.iter().any(|c| c.starts_with("get_run:")), "Expected get_run command in upstream");
    assert!(cmds.iter().any(|c| c.starts_with("cancel_run:")), "Expected cancel_run command in upstream");
}

#[tokio::test]
async fn list_runs_is_scoped_paged_by_rust_and_never_leaks() {
    let (url, state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let ws_b = seed_workspace(&db, &token_b, &app).await;
    state.received_commands.lock().clear();
    let a_runs: Vec<String> = (0..3).map(|_| format!("run_{}", uuid::Uuid::new_v4())).collect();
    for (i, r) in a_runs.iter().enumerate() {
        seed_run(&db, r, &ws_a, &format!("2026-10-12T10:00:0{i}Z")).await;
    }
    let b_run = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &b_run, &ws_b, "2026-10-12T11:00:00Z").await;

    let (status, body) = send(&app, authed("GET", "/api/v1/workflows/runs", &token_b).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let page: Value = serde_json::from_slice(&body).unwrap();
    let ids: Vec<&str> = page["items"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(ids, vec![b_run.as_str()]);
    for a in &a_runs {
        assert!(!String::from_utf8_lossy(&body).contains(a.as_str()), "B's list leaked A's run {a}");
    }
    assert!(!ids.contains(&"run_decoy_not_requested"));

    let cmds = state.received_commands.lock().clone();
    assert!(cmds.contains(&"list_runs".to_string()), "Upstream must receive list_runs command");
}

#[tokio::test]
async fn sse_preserves_id_event_and_exact_json_and_translates_last_event_id() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let req = authed("GET", &format!("/api/v1/workflows/runs/{run_a}/events"), &token_a)
        .header("last-event-id", "2")
        .body(Body::empty())
        .unwrap();
    let (status, body) = tokio::time::timeout(Duration::from_secs(10), send(&app, req))
        .await
        .expect("SSE stream must yield events");
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(body).unwrap();

    assert!(text.contains("id: 3"), "Expected frame seq 3 in stream: {text}");
    assert!(text.contains("event: task.started"), "Expected event task.started in stream: {text}");
    assert!(text.contains("event: workflow.completed"), "Expected terminal workflow.completed: {text}");
}

#[tokio::test]
async fn token_usage_is_user_scoped_and_admin_only_for_global_reports() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let user_a = authenticated_user_id(&app, &token_a).await;
    let user_b = authenticated_user_id(&app, &token_b).await;
    let workspace_a = seed_workspace(&db, &token_a, &app).await;
    let workspace_b = seed_workspace(&db, &token_b, &app).await;
    let event_a = format!("usage_a_{}", uuid::Uuid::new_v4());
    let event_b = format!("usage_b_{}", uuid::Uuid::new_v4());
    seed_token_usage(
        &db,
        &format!("run_usage_a_{}", uuid::Uuid::new_v4()),
        &workspace_a,
        &user_a,
        &event_a,
        120,
        30,
    )
    .await;
    seed_token_usage(
        &db,
        &format!("run_usage_b_{}", uuid::Uuid::new_v4()),
        &workspace_b,
        &user_b,
        &event_b,
        80,
        20,
    )
    .await;

    for (token, own_event, foreign_event) in [
        (&token_a, &event_a, &event_b),
        (&token_b, &event_b, &event_a),
    ] {
        let (status, body) = send(
            &app,
            authed("GET", "/api/v1/usage/tokens", token)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let report: Value = serde_json::from_slice(&body).unwrap();
        let items = report["items"].as_array().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["id"].as_str(), Some(own_event.as_str()));
        assert!(!String::from_utf8_lossy(&body).contains(foreign_event));
        assert_eq!(report["totals"]["request_count"], 1);
    }
    let (status, body) = send(
        &app,
        authed(
            "GET",
            "/api/v1/usage/tokens?before_timestamp=2999-01-01T00%3A00%3A00Z&before_event_id=zzzz",
            &token_a,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let report: Value = serde_json::from_slice(&body).unwrap();
    assert!(report["items"].as_array().unwrap().iter().any(|item| item["id"] == event_a));

    let (status, _) = send(
        &app,
        authed(
            "GET",
            "/api/v1/usage/tokens?before_timestamp=2999-01-01T00%3A00%3A00Z",
            &token_a,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let (status, _) = send(
        &app,
        authed("GET", "/api/v1/admin/usage/tokens", &token_b)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    db.client
        .query("UPDATE type::record('user', $user_id) SET role = 'admin';")
        .bind(("user_id", user_a.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/admin/usage/tokens", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let report: Value = serde_json::from_slice(&body).unwrap();
    assert!(report["totals"]["request_count"].as_u64().unwrap() >= 2);
    let items = report["items"].as_array().unwrap();
    assert!(items.iter().any(|item| item["id"] == event_a));
    assert!(items.iter().any(|item| item["id"] == event_b));
    db.client
        .query(
            "DELETE execution_event WHERE run_id.user_id IN [$user_a, $user_b];
             DELETE workflow_run WHERE user_id IN [$user_a, $user_b];
             DELETE workspace WHERE user_id IN [
                type::record('user', $user_a),
                type::record('user', $user_b)
             ];
             DELETE auth_session WHERE user_id IN [
                type::record('user', $user_a),
                type::record('user', $user_b)
             ];
             DELETE type::record('user', $user_a);
             DELETE type::record('user', $user_b);",
        )
        .bind(("user_a", user_a))
        .bind(("user_b", user_b))
        .await
        .unwrap()
        .check()
        .unwrap();
}
