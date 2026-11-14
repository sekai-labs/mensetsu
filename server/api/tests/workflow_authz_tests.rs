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
            application::{
                oauth_dtos::OAuthExchangeResponseDto, oauth_service::OAuthService, IdentityService,
            },
            domain::{
                hasher::Argon2PasswordHasher,
                oauth::{generate_handoff_code, hash_handoff_code},
                token::TokenService,
            },
            infrastructure::{
                oauth_http::OAuthHttpClient,
                oauth_repository::{
                    SurrealAuthHandoffRepository, SurrealOAuthFlowRepository,
                    SurrealOAuthProviderRepository, SurrealUserOAuthIdentityRepository,
                },
                SurrealAuthSessionRepository, SurrealUserRepository,
            },
        },
        workflow::{application::WorkflowService, infrastructure::SurrealWorkflowRunRepository},
        workspace::{
            application::WorkspaceService,
            infrastructure::{db::DatabasePool, SurrealWorkspaceRepository},
        },
    },
    interfaces::http::{create_router, workspaces::AppState},
    shared::{config::AppConfig, encryption::CryptoService},
};
use serde_json::{json, Value};
use tower::ServiceExt;

const INTERNAL_TOKEN: &str = "test-internal-token-7f3a";

async fn app_with_upstream(intelligence_grpc_url: &str) -> (Router, DatabasePool) {
    let mut config = AppConfig::from_env();
    config.intelligence_grpc_url = intelligence_grpc_url.to_string();
    config.internal_service_token = INTERNAL_TOKEN.to_string();
    config.jwt_secret = "workflow-test-jwt-secret-32-bytes-minimum".to_string();
    config.encryption_key = "workflow-test-encryption-key-32-bytes-minimum".to_string();

    let db = DatabasePool::connect(&config)
        .await
        .expect("SurrealDB must be reachable");
    let ws_repo = Arc::new(SurrealWorkspaceRepository::new((*db.client).clone()));
    let grpc_client = Arc::new(
        mensetsu_api::interfaces::grpc::GrpcClient::connect(&config)
            .await
            .expect("GrpcClient must connect"),
    );
    let storage = Arc::new(
        mensetsu_api::contexts::workspace::infrastructure::storage::ResumeObjectStore::from_config(
            &config,
        )
        .await
        .expect("S3 signer should initialize"),
    );
    let oauth_service = Arc::new(OAuthService::new(
        Arc::new(SurrealOAuthProviderRepository::new(db.client.clone())),
        Arc::new(SurrealUserOAuthIdentityRepository::new(db.client.clone())),
        Arc::new(SurrealOAuthFlowRepository::new(db.client.clone())),
        Arc::new(SurrealAuthHandoffRepository::new(db.client.clone())),
        Arc::new(SurrealUserRepository::new(db.client.clone())),
        Arc::new(SurrealAuthSessionRepository::new(db.client.clone())),
        TokenService::new(
            &config.jwt_secret,
            config.access_token_ttl_seconds,
            config.refresh_token_ttl_seconds,
        ),
        CryptoService::new(&config.encryption_key),
        Arc::new(OAuthHttpClient::new(Vec::new())),
        config.oauth_state_ttl_seconds,
        config.oauth_handoff_ttl_seconds,
        Some("http://localhost:3001/auth/callback".to_string()),
    ));
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        oauth_service: Some(oauth_service),
        identity_service: Arc::new(IdentityService::new(
            Arc::new(SurrealUserRepository::new(db.client.clone())),
            Arc::new(SurrealAuthSessionRepository::new(db.client.clone())),
            Arc::new(Argon2PasswordHasher::new()),
            TokenService::new(
                &config.jwt_secret,
                config.access_token_ttl_seconds,
                config.refresh_token_ttl_seconds,
            ),
        )),
        workspace_service: Arc::new(WorkspaceService::new(ws_repo.clone())),
        workflow_service: Arc::new(WorkflowService::new(
            Arc::new(SurrealWorkflowRunRepository::new((*db.client).clone())),
            ws_repo,
        )),
        grpc_client,
        storage,
        metrics: Arc::new(mensetsu_api::interfaces::http::metrics::MetricsCollector::new()),
    };
    (create_router(state), db)
}

async fn app_with_upstream_and_metrics_token(
    intelligence_grpc_url: &str,
    metrics_token: Option<&str>,
) -> (Router, DatabasePool, AppState) {
    let mut config = AppConfig::from_env();
    config.intelligence_grpc_url = intelligence_grpc_url.to_string();
    config.internal_service_token = INTERNAL_TOKEN.to_string();
    config.jwt_secret = "workflow-test-jwt-secret-32-bytes-minimum".to_string();
    config.encryption_key = "workflow-test-encryption-key-32-bytes-minimum".to_string();
    config.prometheus_metrics_token = metrics_token.map(str::to_string);

    let db = DatabasePool::connect(&config)
        .await
        .expect("SurrealDB must connect for test");
    let ws_repo = Arc::new(SurrealWorkspaceRepository::new((*db.client).clone()));
    let grpc_client = Arc::new(
        mensetsu_api::interfaces::grpc::GrpcClient::connect(&config)
            .await
            .expect("GrpcClient must connect"),
    );
    let storage = Arc::new(
        mensetsu_api::contexts::workspace::infrastructure::storage::ResumeObjectStore::from_config(
            &config,
        )
        .await
        .expect("S3 signer should initialize"),
    );
    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        oauth_service: None,
        identity_service: Arc::new(IdentityService::new(
            Arc::new(SurrealUserRepository::new(db.client.clone())),
            Arc::new(SurrealAuthSessionRepository::new(db.client.clone())),
            Arc::new(Argon2PasswordHasher::new()),
            TokenService::new(
                &config.jwt_secret,
                config.access_token_ttl_seconds,
                config.refresh_token_ttl_seconds,
            ),
        )),
        workspace_service: Arc::new(WorkspaceService::new(ws_repo.clone())),
        workflow_service: Arc::new(WorkflowService::new(
            Arc::new(SurrealWorkflowRunRepository::new((*db.client).clone())),
            ws_repo,
        )),
        grpc_client,
        storage,
        metrics: Arc::new(mensetsu_api::interfaces::http::metrics::MetricsCollector::new()),
    };
    (create_router(state.clone()), db, state)
}

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec();
    (status, body)
}

async fn register(app: &Router) -> String {
    let email = format!("wf_{}@example.com", uuid::Uuid::new_v4());
    let (status, body) = send(
        app,
        Request::post("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(
                json!({"email": email, "password": "Password123!", "name": "WF"}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice::<Value>(&body).unwrap()["access_token"]
        .as_str()
        .unwrap()
        .to_string()
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
    assert_eq!(
        status,
        StatusCode::CREATED,
        "{}",
        String::from_utf8_lossy(&body)
    );
    serde_json::from_slice::<Value>(&body).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn authenticated_user_id(app: &Router, token: &str) -> String {
    let (status, body) = send(
        app,
        authed("GET", "/api/v1/auth/me", token)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    serde_json::from_slice::<Value>(&body).unwrap()["user"]["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn seed_token_usage_full(
    db: &DatabasePool,
    run_id: &str,
    workspace_id: &str,
    user_id: &str,
    event_id: &str,
    agent_id: &str,
    model_provider: &str,
    model_name: &str,
    input_tokens: u64,
    output_tokens: u64,
    ts: Option<&str>,
) {
    let clean_workspace = workspace_id
        .strip_prefix("workspace:")
        .unwrap_or(workspace_id);
    let event_create = if ts.is_some() {
        "CREATE type::record('execution_event', $event_record_id) SET
            event_id = $event_id,
            run_id = type::record('workflow_run', $run_id),
            seq = 1,
            type = 'agent.model_usage',
            execution_id = 'exec_usage',
            ts = <datetime>$ts,
            payload = $payload;"
    } else {
        "CREATE type::record('execution_event', $event_record_id) SET
            event_id = $event_id,
            run_id = type::record('workflow_run', $run_id),
            seq = 1,
            type = 'agent.model_usage',
            execution_id = 'exec_usage',
            payload = $payload;"
    };
    let query_str = format!(
        "CREATE type::record('workflow_run', $run_id) SET
            workspace_id = type::record('workspace', $workspace_id),
            user_id = $user_id,
            workflow_id = 'interview-preparation-static',
            workflow_name = 'Static Interview Preparation Workflow',
            type = 'static',
            status = 'completed',
            current_phase = 'ready',
            last_seq = 1;
         {}",
        event_create
    );
    let mut q = db
        .client
        .query(&query_str)
        .bind(("run_id", run_id.to_owned()))
        .bind(("workspace_id", clean_workspace.to_owned()))
        .bind(("user_id", user_id.to_owned()))
        .bind(("event_record_id", format!("event_{event_id}")))
        .bind(("event_id", event_id.to_owned()))
        .bind((
            "payload",
            json!({
                "agent_id": agent_id,
                "model_provider": model_provider,
                "model_name": model_name,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "total_tokens": input_tokens + output_tokens,
            }),
        ));
    if let Some(timestamp) = ts {
        q = q.bind(("ts", timestamp.to_owned()));
    }
    q.await.unwrap().check().unwrap();
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
    seed_token_usage_full(
        db,
        run_id,
        workspace_id,
        user_id,
        event_id,
        "technical-research",
        "openrouter",
        "example/model",
        input_tokens,
        output_tokens,
        None,
    )
    .await;
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
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/delegations"),
        ),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/artifacts")),
        ("GET", format!("/api/v1/workflows/runs/{run_id}/agents")),
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/events/history"),
        ),
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/tasks/task_1"),
        ),
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/agents/exec_1"),
        ),
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/agents/exec_1/tools"),
        ),
        (
            "GET",
            format!("/api/v1/workflows/runs/{run_id}/tools/tool_1/output"),
        ),
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
        let (status, _body) = send(
            &app,
            authed(method, &uri, &token_b).body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "B: {method} {uri}");
    }
    for (method, uri) in run_scoped_routes(&unknown) {
        let (status, _body) = send(
            &app,
            authed(method, &uri, &token_b).body(Body::empty()).unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "unknown: {method} {uri}");
    }

    assert!(
        state.received_commands.lock().is_empty(),
        "nothing may reach gRPC upstream for unauthorized runs: {:?}",
        state.received_commands.lock()
    );

    let (status, body) = send(
        &app,
        authed("GET", &format!("/api/v1/workflows/runs/{run_a}"), &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));

    let (status, body) = send(
        &app,
        authed(
            "POST",
            &format!("/api/v1/workflows/runs/{run_a}/cancel"),
            &token_a,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(
        serde_json::from_slice::<Value>(&body).unwrap()["status"],
        "cancelled"
    );

    let cmds = state.received_commands.lock().clone();
    assert!(
        cmds.iter().any(|c| c.starts_with("get_run:")),
        "Expected get_run command in upstream"
    );
    assert!(
        cmds.iter().any(|c| c.starts_with("cancel_run:")),
        "Expected cancel_run command in upstream"
    );
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
    let a_runs: Vec<String> = (0..3)
        .map(|_| format!("run_{}", uuid::Uuid::new_v4()))
        .collect();
    for (i, r) in a_runs.iter().enumerate() {
        seed_run(&db, r, &ws_a, &format!("2026-10-12T10:00:0{i}Z")).await;
    }
    let b_run = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &b_run, &ws_b, "2026-10-12T11:00:00Z").await;

    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/workflows/runs", &token_b)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let page: Value = serde_json::from_slice(&body).unwrap();
    let ids: Vec<&str> = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![b_run.as_str()]);
    for a in &a_runs {
        assert!(
            !String::from_utf8_lossy(&body).contains(a.as_str()),
            "B's list leaked A's run {a}"
        );
    }
    assert!(!ids.contains(&"run_decoy_not_requested"));

    let cmds = state.received_commands.lock().clone();
    assert!(
        cmds.contains(&"list_runs".to_string()),
        "Upstream must receive list_runs command"
    );
}

#[tokio::test]
async fn sse_preserves_id_event_and_exact_json_and_translates_last_event_id() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let req = authed(
        "GET",
        &format!("/api/v1/workflows/runs/{run_a}/events"),
        &token_a,
    )
    .header("last-event-id", "2")
    .body(Body::empty())
    .unwrap();
    let (status, body) = tokio::time::timeout(Duration::from_secs(10), send(&app, req))
        .await
        .expect("SSE stream must yield events");
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(body).unwrap();

    assert!(
        text.contains("id: 3"),
        "Expected frame seq 3 in stream: {text}"
    );
    assert!(
        text.contains("event: task.started"),
        "Expected event task.started in stream: {text}"
    );
    assert!(
        text.contains("event: workflow.completed"),
        "Expected terminal workflow.completed: {text}"
    );
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
    assert!(report["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["id"] == event_a));

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

#[tokio::test]
async fn test_prometheus_metrics_bearer_auth_fail_closed_and_safe_comparison() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;

    // 1. Fail closed when unset
    let (app_no_token, _db, _state) = app_with_upstream_and_metrics_token(&url, None).await;
    let (status, _) = send(
        &app_no_token,
        Request::get("/metrics").body(Body::empty()).unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let (status, _) = send(
        &app_no_token,
        Request::get("/metrics")
            .header("authorization", "Bearer test_token")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // 2. Configured token
    let secret = "prometheus-test-token-constant-time-safety-42";
    let (app, _db, _state) = app_with_upstream_and_metrics_token(&url, Some(secret)).await;

    // Unauthenticated
    let (status, _) = send(&app, Request::get("/metrics").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Wrong token
    let (status, _) = send(
        &app,
        Request::get("/metrics")
            .header("authorization", "Bearer wrong-secret")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Valid bearer token
    let (status, body) = send(
        &app,
        Request::get("/metrics")
            .header("authorization", format!("Bearer {secret}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(body).unwrap();
    assert!(text.contains("# HELP http_requests_total"));
    assert!(text.contains("# TYPE http_requests_total counter"));
    assert!(text.contains("# HELP http_request_duration_seconds"));
    assert!(text.contains("# TYPE http_request_duration_seconds histogram"));
    assert!(text.contains("mensetsu_service_success_rate_percent"));
    assert!(text.contains("mensetsu_service_requests_per_second"));

    // Verify bounded cardinality and NEVER label by user ID
    assert!(!text.contains("user_id"));
    assert!(!text.contains("usr_"));
}

#[tokio::test]
async fn test_metrics_summary_roles_and_scoping() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db, _state) = app_with_upstream_and_metrics_token(&url, Some("token-42")).await;

    // Unauthenticated -> 401
    let (status, _) = send(
        &app,
        Request::get("/api/v1/metrics/summary")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // Register User A and User B
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let user_a = authenticated_user_id(&app, &token_a).await;
    let user_b = authenticated_user_id(&app, &token_b).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let ws_b = seed_workspace(&db, &token_b, &app).await;

    let event_a = format!("ms_a_{}", uuid::Uuid::new_v4());
    let event_b = format!("ms_b_{}", uuid::Uuid::new_v4());
    seed_token_usage(
        &db,
        &format!("run_ms_a_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_a,
        100,
        50,
    )
    .await;
    seed_token_usage(
        &db,
        &format!("run_ms_b_{}", uuid::Uuid::new_v4()),
        &ws_b,
        &user_b,
        &event_b,
        200,
        100,
    )
    .await;

    // Regular user A calls summary: only own totals, no service-wide fields
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/metrics/summary", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let summary_a: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary_a["token_totals"]["total_tokens"], 150);
    assert_eq!(summary_a["token_totals"]["request_count"], 1);
    assert!(
        summary_a.get("platform_token_totals").is_none()
            || summary_a["platform_token_totals"].is_null()
    );
    assert!(summary_a.get("service_metrics").is_none() || summary_a["service_metrics"].is_null());

    // Promote User B to admin
    db.client
        .query("UPDATE type::record('user', $user_id) SET role = 'admin';")
        .bind(("user_id", user_b.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    // Admin user B calls summary: receives platform totals and service metrics
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/metrics/summary", &token_b)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let summary_b: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(summary_b["token_totals"]["total_tokens"], 300);
    assert!(
        summary_b["platform_token_totals"]["total_tokens"]
            .as_u64()
            .unwrap()
            >= 450
    );
    assert!(
        summary_b["platform_token_totals"]["request_count"]
            .as_u64()
            .unwrap()
            >= 2
    );
    assert!(summary_b["service_metrics"]["success_rate_percent"].is_number());
    assert!(summary_b["service_metrics"]["requests_per_second"].is_number());

    // Clean up
    db.client
        .query(
            "DELETE execution_event WHERE run_id.user_id IN [$user_a, $user_b];
             DELETE workflow_run WHERE user_id IN [$user_a, $user_b];
             DELETE workspace WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
             DELETE auth_session WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
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

#[tokio::test]
async fn test_daily_token_analytics_user_scoped_and_admin_wide() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let user_a = authenticated_user_id(&app, &token_a).await;
    let user_b = authenticated_user_id(&app, &token_b).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let ws_b = seed_workspace(&db, &token_b, &app).await;

    let event_a1 = format!("d_a1_{}", uuid::Uuid::new_v4());
    let event_a2 = format!("d_a2_{}", uuid::Uuid::new_v4());
    let event_b1 = format!("d_b1_{}", uuid::Uuid::new_v4());

    seed_token_usage(
        &db,
        &format!("run_da1_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_a1,
        100,
        50,
    )
    .await;
    seed_token_usage(
        &db,
        &format!("run_da2_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_a2,
        80,
        20,
    )
    .await;
    seed_token_usage(
        &db,
        &format!("run_db1_{}", uuid::Uuid::new_v4()),
        &ws_b,
        &user_b,
        &event_b1,
        300,
        100,
    )
    .await;

    // User A: GET /api/v1/usage/tokens/daily
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/usage/tokens/daily", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let daily_a: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(daily_a["totals"]["total_tokens"], 250);
    assert_eq!(daily_a["totals"]["request_count"], 2);
    assert!(!daily_a["items"].as_array().unwrap().is_empty());

    // User A calls admin daily endpoint -> 403 Forbidden
    let (status, _) = send(
        &app,
        authed("GET", "/api/v1/admin/usage/tokens/daily", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Promote User B to admin
    db.client
        .query("UPDATE type::record('user', $user_id) SET role = 'admin';")
        .bind(("user_id", user_b.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    // User B calls admin daily endpoint -> 200 OK
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/admin/usage/tokens/daily", &token_b)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let daily_admin: Value = serde_json::from_slice(&body).unwrap();
    assert!(daily_admin["totals"]["total_tokens"].as_u64().unwrap() >= 650);
    assert!(daily_admin["totals"]["request_count"].as_u64().unwrap() >= 3);

    // Clean up
    db.client
        .query(
            "DELETE execution_event WHERE run_id.user_id IN [$user_a, $user_b];
             DELETE workflow_run WHERE user_id IN [$user_a, $user_b];
             DELETE workspace WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
             DELETE auth_session WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
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

#[tokio::test]
async fn test_token_usage_filters_search_and_pagination() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let user_a = authenticated_user_id(&app, &token_a).await;
    let user_b = authenticated_user_id(&app, &token_b).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let _ws_b = seed_workspace(&db, &token_b, &app).await;

    let event_1 = format!("filter_1_{}", uuid::Uuid::new_v4());
    let event_2 = format!("filter_2_{}", uuid::Uuid::new_v4());
    let event_3 = format!("filter_3_{}", uuid::Uuid::new_v4());

    // Record 1: openrouter, meta-llama/llama-3, technical-research
    seed_token_usage_full(
        &db,
        &format!("run_f1_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_1,
        "technical-research",
        "openrouter",
        "meta-llama/llama-3",
        100,
        50,
        Some("2026-11-10T10:00:00Z"),
    )
    .await;

    // Record 2: anthropic, claude-3-opus, culture-matcher
    seed_token_usage_full(
        &db,
        &format!("run_f2_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_2,
        "culture-matcher",
        "anthropic",
        "claude-3-opus",
        200,
        100,
        Some("2026-11-11T12:00:00Z"),
    )
    .await;

    // Record 3: anthropic, claude-3-sonnet, technical-research
    seed_token_usage_full(
        &db,
        &format!("run_f3_{}", uuid::Uuid::new_v4()),
        &ws_a,
        &user_a,
        &event_3,
        "technical-research",
        "anthropic",
        "claude-3-sonnet",
        300,
        150,
        Some("2026-11-12T14:00:00Z"),
    )
    .await;

    // 1. Search filter: q=culture matches Record 2
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/usage/tokens?q=culture", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let report: Value = serde_json::from_slice(&body).unwrap();
    let items = report["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["id"], event_2);
    // Totals remain scope-wide!
    assert_eq!(report["totals"]["request_count"], 3);
    assert_eq!(report["totals"]["total_tokens"], 900);

    // 2. Provider filter: provider=anthropic matches Records 2 and 3
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/usage/tokens?provider=anthropic", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let report: Value = serde_json::from_slice(&body).unwrap();
    let items = report["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Scope-wide totals preserved
    assert_eq!(report["totals"]["request_count"], 3);

    // 3. Agent filter: agent=culture-matcher matches Record 2
    let (status, body) = send(
        &app,
        authed(
            "GET",
            "/api/v1/usage/tokens?agent=culture-matcher",
            &token_a,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let report: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["items"].as_array().unwrap().len(), 1);

    // 4. Date range filter: from 2026-11-11 to 2026-11-12 matches Records 2 and 3
    let (status, body) = send(
        &app,
        authed(
            "GET",
            "/api/v1/usage/tokens?from=2026-11-11&to=2026-11-12",
            &token_a,
        )
        .body(Body::empty())
        .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let report: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(report["items"].as_array().unwrap().len(), 2);

    // 5. Pagination with limit=1 and cursor
    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/usage/tokens?limit=1", &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let page1: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(page1["items"].as_array().unwrap().len(), 1);
    assert!(page1["next_cursor"].is_object());
    let next_ts = page1["next_cursor"]["before_timestamp"].as_str().unwrap();
    let next_id = page1["next_cursor"]["before_event_id"].as_str().unwrap();

    let page2_url = format!(
        "/api/v1/usage/tokens?limit=1&before_timestamp={}&before_event_id={}",
        urlencoding_simple(next_ts),
        next_id
    );
    let (status, body) = send(
        &app,
        authed("GET", &page2_url, &token_a)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let page2: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(page2["items"].as_array().unwrap().len(), 1);
    assert_ne!(page1["items"][0]["id"], page2["items"][0]["id"]);

    // 6. Admin user_id filter
    db.client
        .query("UPDATE type::record('user', $user_id) SET role = 'admin';")
        .bind(("user_id", user_b.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let admin_url = format!("/api/v1/admin/usage/tokens?user_id={user_a}");
    let (status, body) = send(
        &app,
        authed("GET", &admin_url, &token_b)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let admin_report: Value = serde_json::from_slice(&body).unwrap();
    let admin_items = admin_report["items"].as_array().unwrap();
    assert_eq!(admin_items.len(), 3);
    for item in admin_items {
        assert_eq!(item["user_id"], user_a);
    }

    // Clean up
    db.client
        .query(
            "DELETE execution_event WHERE run_id.user_id IN [$user_a, $user_b];
             DELETE workflow_run WHERE user_id IN [$user_a, $user_b];
             DELETE workspace WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
             DELETE auth_session WHERE user_id IN [type::record('user', $user_a), type::record('user', $user_b)];
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

#[tokio::test]
async fn test_admin_oauth_provider_crud_and_ssrf() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_user = register(&app).await;
    let user_id = authenticated_user_id(&app, &token_user).await;

    let pkey = format!("test_idp_{}", uuid::Uuid::new_v4().simple());

    // 1. Regular user gets 403 on admin OAuth endpoint
    let (status, _) = send(
        &app,
        authed("GET", "/api/v1/admin/oauth/providers", &token_user)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Promote user to admin
    db.client
        .query("UPDATE type::record('user', $user_id) SET role = 'admin';")
        .bind(("user_id", user_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let provider_url = format!("/api/v1/admin/oauth/providers/{pkey}");
    let ssrf_payload = json!({
        "name": "Malicious Provider",
        "provider_type": "oauth2",
        "client_id": "test-client-id",
        "client_secret": "test-secret-123",
        "redirect_uri": "https://api.example.com/api/v1/auth/oauth/callback",
        "authorization_endpoint": "https://93.184.216.34/oauth/authorize",
        "token_endpoint": "https://127.0.0.1/token",
        "userinfo_endpoint": "https://93.184.216.34/userinfo"
    });
    let (status, _) = send(
        &app,
        authed("PUT", &provider_url, &token_user)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&ssrf_payload).unwrap()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let create_payload = json!({
        "name": "Custom Secure IdP",
        "provider_type": "oauth2",
        "client_id": "my-client-id-abc",
        "client_secret": "super-confidential-secret-xyz",
        "redirect_uri": "https://api.example.com/api/v1/auth/oauth/callback",
        "authorization_endpoint": "https://93.184.216.34/oauth/authorize",
        "token_endpoint": "https://93.184.216.34/oauth/token",
        "userinfo_endpoint": "https://93.184.216.34/userinfo",
        "is_enabled": true
    });
    let (status, body) = send(
        &app,
        authed("PUT", &provider_url, &token_user)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&create_payload).unwrap()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));

    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/admin/oauth/providers", &token_user)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list: Value = serde_json::from_slice(&body).unwrap();
    let providers = list.as_array().unwrap();
    let found = providers
        .iter()
        .find(|provider| provider["provider_key"] == pkey)
        .expect("Created provider must be in admin list");
    assert_eq!(found["name"], "Custom Secure IdP");
    assert_eq!(found["has_client_secret"], true);
    assert!(found.get("client_secret").is_none());
    assert!(found.get("encrypted_client_secret").is_none());

    let update_payload = json!({
        "name": "Updated IdP Name"
    });
    let (status, _) = send(
        &app,
        authed("PUT", &provider_url, &token_user)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&update_payload).unwrap()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = send(
        &app,
        authed("GET", "/api/v1/admin/oauth/providers", &token_user)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let list: Value = serde_json::from_slice(&body).unwrap();
    let updated = list
        .as_array()
        .unwrap()
        .iter()
        .find(|provider| provider["provider_key"] == pkey)
        .unwrap();
    assert_eq!(updated["name"], "Updated IdP Name");
    assert_eq!(updated["has_client_secret"], true);

    let (status, body) = send(
        &app,
        Request::builder()
            .method("GET")
            .uri("/api/v1/auth/oauth/providers")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let public_list: Value = serde_json::from_slice(&body).unwrap();
    let public_provider = public_list
        .as_array()
        .unwrap()
        .iter()
        .find(|provider| provider["provider_key"] == pkey)
        .expect("Public provider must be present");
    assert_eq!(public_provider["name"], "Updated IdP Name");
    assert_eq!(public_provider["provider_type"], "oauth2");
    assert!(public_provider.get("client_id").is_none());
    assert!(public_provider.get("client_secret").is_none());

    let (status, _) = send(
        &app,
        authed("DELETE", &provider_url, &token_user)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Clean up
    db.client
        .query(
            "DELETE auth_session WHERE user_id = type::record('user', $uid);
             DELETE type::record('user', $uid);",
        )
        .bind(("uid", user_id))
        .await
        .unwrap()
        .check()
        .unwrap();
}

#[tokio::test]
async fn test_oauth_handoff_exchange_and_single_use() {
    let (url, _state, _server) = mock_grpc_server::start_dynamic_mock_grpc_server("".into()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_user = register(&app).await;
    let user_id = authenticated_user_id(&app, &token_user).await;

    let handoff_code = generate_handoff_code();
    let handoff_hash = hash_handoff_code(&handoff_code);
    let record_id = uuid::Uuid::new_v4().to_string();
    let expires_at = (chrono::Utc::now() + chrono::Duration::seconds(60)).to_rfc3339();
    let exchange_payload = OAuthExchangeResponseDto {
        user: None,
        access_token: Some("test-access-token".to_string()),
        refresh_token: Some("test-refresh-token".to_string()),
        requires_2fa: false,
        temp_token: None,
        linked_identity: None,
    };
    let encrypted_payload = CryptoService::new("workflow-test-encryption-key-32-bytes-minimum")
        .encrypt(&serde_json::to_string(&exchange_payload).unwrap())
        .unwrap();

    db.client
        .query(
            "CREATE type::record('auth_handoff', $id) SET
                code = $code,
                user_id = type::record('user', $uid),
                encrypted_payload = $encrypted_payload,
                expires_at = type::datetime($expires),
                used = false,
                created_at = time::now();",
        )
        .bind(("id", record_id.clone()))
        .bind(("code", handoff_hash))
        .bind(("uid", user_id.clone()))
        .bind(("encrypted_payload", encrypted_payload))
        .bind(("expires", expires_at))
        .await
        .unwrap()
        .check()
        .unwrap();

    let request_payload = json!({ "code": handoff_code });
    let (status, body) = send(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/oauth/exchange")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let exchanged: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(exchanged["access_token"], "test-access-token");

    let (status, _) = send(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/auth/oauth/exchange")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&request_payload).unwrap()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    db.client
        .query(
            "DELETE type::record('auth_handoff', $id);
             DELETE auth_session WHERE user_id = type::record('user', $uid);
             DELETE type::record('user', $uid);",
        )
        .bind(("id", record_id))
        .bind(("uid", user_id))
        .await
        .unwrap()
        .check()
        .unwrap();
}

fn urlencoding_simple(s: &str) -> String {
    s.replace(':', "%3A").replace('+', "%2B")
}
