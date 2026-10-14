use std::sync::Arc;
use std::time::Duration;

use axum::{
    body::Body,
    extract::{Path, Request as AxumRequest},
    http::{HeaderMap, Request, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
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
    interfaces::http::{create_router, intelligence_client::IntelligenceClient, workspaces::AppState},
    shared::config::AppConfig,
};
use serde_json::{json, Value};
use tokio::io::AsyncWriteExt;
use tower::ServiceExt;

const INTERNAL_TOKEN: &str = "test-internal-token-7f3a";
const BROWSER_COOKIE: &str = "mensetsu_session_id=browser-session-secret; theme=dark";

#[derive(Debug, Clone)]
struct Seen {
    method: String,
    path: String,
    query: String,
    headers: HeaderMap,
}

#[derive(Default)]
struct Upstream {
    seen: parking_lot::Mutex<Vec<Seen>>,
}

impl Upstream {
    fn seen(&self) -> Vec<Seen> {
        self.seen.lock().clone()
    }
    fn hits(&self, method: &str, suffix: &str) -> usize {
        self.seen().iter().filter(|s| s.method == method && s.path.ends_with(suffix)).count()
    }
}

fn run_json(id: &str, workspace_id: &str, status: &str) -> Value {
    json!({
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

const SSE_CHUNKS: &[&str] = &[
    ": keepalive\n\nid: 3\nevent: task.sta",
    "rted\ndata: {\"sequence\":3,\"type\":\"task.started\",\"payload\":{\"task_id\":\"t1\"}}\r",
    "\n\r\nid: 4\r\nevent: agent.started\r\ndata: {\"sequence\":4,",
    "\"payload\":{\"line\":\"a\\nb\"}}\r\n",
    "\r\n",
    "id: 5\nevent: workflow.completed\ndata: {\"sequence\":5}\n\n",
];

async fn record(axum::extract::State(state): axum::extract::State<Arc<Upstream>>, req: AxumRequest) -> AxumRequest {
    state.seen.lock().push(Seen {
        method: req.method().to_string(),
        path: req.uri().path().to_string(),
        query: req.uri().query().unwrap_or("").to_string(),
        headers: req.headers().clone(),
    });
    req
}

async fn start_upstream(workspace_id: String) -> (String, Arc<Upstream>) {
    let up = Arc::new(Upstream::default());
    let base = "/internal/v1/workflows";

    let ws = workspace_id.clone();
    let ws2 = workspace_id.clone();
    let ws3 = workspace_id.clone();
    let app = Router::new()
        .route(base, get(|| async { Json(json!([])) }))
        .route(
            &format!("{base}/runs"),
            get(move |axum::extract::RawQuery(q): axum::extract::RawQuery| {
                let ws = ws.clone();
                async move {
                    let q = q.unwrap_or_default();
                    let ids = url_param(&q, "run_ids").unwrap_or_default();
                    let mut items: Vec<Value> = ids
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|id| run_json(id, &ws, "running"))
                        .collect();
                    items.push(run_json("run_decoy_not_requested", "someone_else", "running"));
                    Json(json!({"items": items, "next_cursor": null}))
                }
            })
            .post(move |Json(body): Json<Value>| {
                let ws = ws2.clone();
                async move {
                    assert_eq!(body["workspace_id"], json!(ws));
                    (StatusCode::CREATED, Json(run_json("run_created_by_trigger", &ws, "pending")))
                }
            }),
        )
        .route(
            &format!("{base}/runs/{{run_id}}"),
            get(move |Path(run_id): Path<String>| {
                let ws = ws3.clone();
                async move { Json(run_json(&run_id, &ws, "running")) }
            }),
        )
        .route(
            &format!("{base}/runs/{{run_id}}/cancel"),
            post(|Path(run_id): Path<String>| async move { Json(run_json(&run_id, "x", "cancelled")) }),
        )
        .route(&format!("{base}/runs/{{run_id}}/graph"), get(|| async { Json(json!({"nodes": [], "edges": []})) }))
        .route(&format!("{base}/runs/{{run_id}}/logs"), get(|| async { Json(json!({"items": [], "next_cursor": null})) }))
        .route(&format!("{base}/runs/{{run_id}}/delegations"), get(|| async { Json(json!([])) }))
        .route(&format!("{base}/runs/{{run_id}}/artifacts"), get(|| async { Json(json!([])) }))
        .route(&format!("{base}/runs/{{run_id}}/agents"), get(|| async { Json(json!([])) }))
        .route(&format!("{base}/runs/{{run_id}}/events/history"), get(|| async { Json(json!({"events": [], "next_after_seq": null, "last_seq": 0})) }))
        .route(
            &format!("{base}/runs/{{run_id}}/tasks/{{task_id}}"),
            get(|| async { StatusCode::NOT_FOUND }),
        )
        .route(
            &format!("{base}/runs/{{run_id}}/agents/{{execution_id}}"),
            get(|| async { StatusCode::NOT_FOUND }),
        )
        .route(
            &format!("{base}/runs/{{run_id}}/agents/{{execution_id}}/tools"),
            get(|| async { Json(json!({"items": [], "next_cursor": null})) }),
        )
        .route(
            &format!("{base}/runs/{{run_id}}/tools/{{tool_id}}/output"),
            get(|Path((_run, tool_id)): Path<(String, String)>| async move {
                Json(json!({
                    "tool_invocation_id": tool_id,
                    "offset": 0,
                    "limit": 100,
                    "total_bytes": 0,
                    "has_more": false,
                    "content": ""
                }))
            }),
        )
        .route(&format!("{base}/runs/{{run_id}}/events"), get(sse_handler))
        .layer(axum::middleware::map_request_with_state(up.clone(), record));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), up)
}

async fn sse_handler() -> Response {
    let (mut tx, rx) = tokio::io::duplex(64);
    tokio::spawn(async move {
        for chunk in SSE_CHUNKS {
            tx.write_all(chunk.as_bytes()).await.unwrap();
            tx.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
    });
    let body = Body::from_stream(tokio_util::io::ReaderStream::new(rx));
    ([("content-type", "text/event-stream")], body).into_response()
}

fn url_param(query: &str, key: &str) -> Option<String> {
    url::form_urlencoded::parse(query.as_bytes())
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

async fn app_with_upstream(intelligence_url: &str) -> (Router, DatabasePool) {
    let mut config = AppConfig::from_env();
    config.intelligence_url = intelligence_url.to_string();
    config.internal_service_token = INTERNAL_TOKEN.to_string();

    let db = DatabasePool::connect(&config).await.expect("SurrealDB must be reachable");
    let ws_repo = Arc::new(SurrealWorkspaceRepository::new((*db.client).clone()));
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
        intelligence_client: Arc::new(IntelligenceClient::new(&config).unwrap()),
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
    assert!(status.is_success(), "register failed: {status} {}", String::from_utf8_lossy(&body));
    serde_json::from_slice::<Value>(&body).unwrap()["access_token"].as_str().unwrap().to_string()
}

async fn user_id_of(app: &Router, token: &str) -> String {
    let (_, me) = send(
        app,
        Request::get("/api/v1/auth/me").header("authorization", format!("Bearer {token}")).body(Body::empty()).unwrap(),
    )
    .await;
    let me: Value = serde_json::from_slice(&me).unwrap();
    let user_id = me["id"].as_str().or_else(|| me["user"]["id"].as_str()).expect("me.id");
    user_id.replace("user:", "").replace('`', "")
}

async fn seed_workspace(db: &DatabasePool, owner_token: &str, app: &Router) -> String {
    let user_id = user_id_of(app, owner_token).await;
    let ws_id = format!("ws_{}", uuid::Uuid::new_v4().simple());
    db.client
        .query(
            "CREATE type::record('workspace', $id) SET user_id = type::record('user', $uid), company_name = 'Acme',
             role_title = 'Engineer', status = 'processing', current_phase = 'preparing_input', progress_percent = 0;",
        )
        .bind(("id", ws_id.clone()))
        .bind(("uid", user_id))
        .await
        .unwrap()
        .check()
        .unwrap();
    ws_id
}

async fn seed_run(db: &DatabasePool, run_id: &str, ws_id: &str, created_at: &str) {
    db.client
        .query(
            "CREATE type::record('workflow_run', $id) SET workspace_id = type::record('workspace', $ws),
             user_id = 'ledger-user', workflow_id = 'interview-preparation-static',
             workflow_name = 'Static Interview Preparation Workflow', type = 'static', status = 'running',
             current_phase = 'research', created_at = <datetime> $created_at;",
        )
        .bind(("id", run_id.to_string()))
        .bind(("ws", ws_id.to_string()))
        .bind(("created_at", created_at.to_string()))
        .await
        .unwrap()
        .check()
        .unwrap();
}

fn authed(method: &str, uri: &str, token: &str) -> axum::http::request::Builder {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .header("cookie", BROWSER_COOKIE)
}

fn run_scoped_routes(run_id: &str) -> Vec<(&'static str, String)> {
    let base = format!("/api/v1/workflows/runs/{run_id}");
    vec![
        ("GET", base.clone()),
        ("GET", format!("{base}/graph")),
        ("GET", format!("{base}/delegations/graph")),
        ("GET", format!("{base}/events")),
        ("GET", format!("{base}/events/history")),
        ("GET", format!("{base}/logs")),
        ("GET", format!("{base}/tasks/task_1")),
        ("GET", format!("{base}/agents")),
        ("GET", format!("{base}/agents/exec_1")),
        ("GET", format!("{base}/agents/exec_1/tools")),
        ("GET", format!("{base}/tools/tool_1/output")),
        ("GET", format!("{base}/delegations")),
        ("GET", format!("{base}/artifacts")),
        ("POST", format!("{base}/cancel")),
    ]
}

fn assert_upstream_headers_clean(up: &Upstream, user_tokens: &[&str]) {
    let seen = up.seen();
    assert!(!seen.is_empty(), "upstream was never called; header assertions would be vacuous");
    for s in &seen {
        let auth = s.headers.get("authorization").map(|v| v.to_str().unwrap().to_string());
        assert_eq!(
            auth.as_deref(),
            Some(format!("Bearer {INTERNAL_TOKEN}").as_str()),
            "{} {} must carry exactly the internal service token",
            s.method,
            s.path
        );
        assert_eq!(s.headers.get_all("authorization").iter().count(), 1, "{} duplicated Authorization", s.path);
        assert!(s.headers.get("cookie").is_none(), "{} {} forwarded the browser Cookie", s.method, s.path);
        for raw in s.headers.values() {
            let v = raw.to_str().unwrap_or("");
            assert!(!v.contains("browser-session-secret"), "{} leaked browser cookie value", s.path);
            for tok in user_tokens {
                assert!(!v.contains(tok), "{} leaked a user access token", s.path);
            }
        }
    }
}

#[tokio::test]
async fn foreign_and_unknown_runs_are_404_on_every_route_and_never_forwarded() {
    let (url, up) = start_upstream(String::new()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;
    let unknown = format!("run_{}", uuid::Uuid::new_v4());

    let mut foreign_bodies = Vec::new();
    for (method, uri) in run_scoped_routes(&run_a) {
        let (status, body) = send(&app, authed(method, &uri, &token_b).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "B: {method} {uri}");
        foreign_bodies.push(body);
    }
    for (i, (method, uri)) in run_scoped_routes(&unknown).into_iter().enumerate() {
        let (status, body) = send(&app, authed(method, &uri, &token_b).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "unknown: {method} {uri}");
        assert_eq!(
            String::from_utf8_lossy(&body).replace(&unknown, "<id>"),
            String::from_utf8_lossy(&foreign_bodies[i]).replace(&run_a, "<id>"),
            "foreign vs unknown 404 differ for {uri}"
        );
    }
    assert!(up.seen().is_empty(), "nothing may reach upstream for unauthorized runs: {:?}", up.seen());
    assert_eq!(up.hits("POST", "/cancel"), 0, "B's cancel must not be forwarded");

    let (status, body) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let (status, body) = send(&app, authed("POST", &format!("/api/v1/workflows/runs/{run_a}/cancel"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    assert_eq!(serde_json::from_slice::<Value>(&body).unwrap()["status"], "cancelled");
    assert_eq!(up.hits("POST", "/cancel"), 1, "only A's cancel is forwarded");

    assert_upstream_headers_clean(&up, &[&token_a, &token_b]);
}

#[tokio::test]
async fn list_runs_is_scoped_paged_by_rust_and_never_leaks() {
    let (url, up) = start_upstream(String::new()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let ws_b = seed_workspace(&db, &token_b, &app).await;

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

    let list_calls: Vec<Seen> = up.seen().into_iter().filter(|s| s.method == "GET" && s.path.ends_with("/runs")).collect();
    assert_eq!(url_param(&list_calls.last().unwrap().query, "run_ids").as_deref(), Some(b_run.as_str()));

    let (status, body) = send(&app, authed("GET", "/api/v1/workflows/runs?limit=2", &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    let p1: Value = serde_json::from_slice(&body).unwrap();
    let p1_ids: Vec<&str> = p1["items"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(p1_ids, vec![a_runs[2].as_str(), a_runs[1].as_str()]);
    let cursor = p1["next_cursor"].as_str().expect("next_cursor on a full page").to_string();

    let (status, body) = send(&app, authed("GET", &format!("/api/v1/workflows/runs?limit=2&cursor={cursor}"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    let p2: Value = serde_json::from_slice(&body).unwrap();
    let p2_ids: Vec<&str> = p2["items"].as_array().unwrap().iter().map(|r| r["id"].as_str().unwrap()).collect();
    assert_eq!(p2_ids, vec![a_runs[0].as_str()]);
    assert!(p2["next_cursor"].is_null());

    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs?cursor={cursor}"), &token_b).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs?workspace_id={ws_a}"), &token_b).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    assert_upstream_headers_clean(&up, &[&token_a, &token_b]);
}

#[tokio::test]
async fn sse_preserves_id_event_and_exact_json_and_translates_last_event_id() {
    let (url, up) = start_upstream(String::new()).await;
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
        .expect("SSE stream must end when upstream closes");
    assert_eq!(status, StatusCode::OK);
    let text = String::from_utf8(body).unwrap();

    let mut parser = mensetsu_api::interfaces::http::sse_parser::SseParser::new();
    let frames = parser.feed(text.as_bytes());
    assert_eq!(frames.len(), 3, "client stream:\n{text}");

    assert_eq!(frames[0].id.as_deref(), Some("3"));
    assert_eq!(frames[0].event.as_deref(), Some("task.started"));
    let data0: Value = serde_json::from_str(&frames[0].data).expect("data must be exactly JSON");
    assert_eq!(data0, json!({"sequence": 3, "type": "task.started", "payload": {"task_id": "t1"}}));
    for f in &frames {
        assert!(!f.data.contains("id:") && !f.data.contains("event:"), "framing leaked into data: {}", f.data);
    }

    assert_eq!(frames[1].id.as_deref(), Some("4"));
    assert_eq!(frames[1].event.as_deref(), Some("agent.started"));
    assert_eq!(serde_json::from_str::<Value>(&frames[1].data).unwrap()["payload"]["line"], "a\nb");

    assert_eq!(frames[2].id.as_deref(), Some("5"));
    assert_eq!(frames[2].event.as_deref(), Some("workflow.completed"));

    let sse_calls: Vec<Seen> = up.seen().into_iter().filter(|s| s.path.ends_with("/events")).collect();
    assert_eq!(sse_calls.len(), 1);
    assert_eq!(url_param(&sse_calls[0].query, "after_seq").as_deref(), Some("2"));
    assert!(sse_calls[0].headers.get("last-event-id").is_none(), "cursor is passed as after_seq, not forwarded raw");

    let req = authed("GET", &format!("/api/v1/workflows/runs/{run_a}/events?after_seq=7"), &token_a)
        .header("last-event-id", "2")
        .body(Body::empty())
        .unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(10), send(&app, req)).await.unwrap();
    let last = up.seen().into_iter().filter(|s| s.path.ends_with("/events")).last().unwrap();
    assert_eq!(url_param(&last.query, "after_seq").as_deref(), Some("7"));

    assert_upstream_headers_clean(&up, &[&token_a]);
}

#[tokio::test]
async fn trigger_uses_internal_token_and_server_side_identity() {
    let (app0, db) = app_with_upstream("http://127.0.0.1:9").await;
    let token_a = register(&app0).await;
    let ws_a = seed_workspace(&db, &token_a, &app0).await;
    let (url, up) = start_upstream(ws_a.clone()).await;
    let (app, _) = app_with_upstream(&url).await;

    let body = json!({"workspace_id": ws_a, "workflow_id": "interview-preparation-static"});
    let (status, resp) = send(
        &app,
        authed("POST", "/api/v1/workflows/runs", &token_a)
            .header("content-type", "application/json")
            .header("idempotency-key", "client-key-123")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "{}", String::from_utf8_lossy(&resp));
    assert_eq!(serde_json::from_slice::<Value>(&resp).unwrap()["id"], "run_created_by_trigger");

    let posts: Vec<Seen> = up.seen().into_iter().filter(|s| s.method == "POST" && s.path.ends_with("/runs")).collect();
    assert_eq!(posts.len(), 1);
    assert_eq!(posts[0].headers.get("idempotency-key").unwrap(), "client-key-123");

    let smuggled = json!({"workspace_id": ws_a, "workflow_id": "interview-preparation-static", "user_id": "victim"});
    let (status, _) = send(
        &app,
        authed("POST", "/api/v1/workflows/runs", &token_a)
            .header("content-type", "application/json")
            .body(Body::from(smuggled.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(up.hits("POST", "/runs"), 1, "rejected body must not be forwarded");

    assert_upstream_headers_clean(&up, &[&token_a]);
}

#[tokio::test]
async fn upstream_contract_mismatch_is_502() {
    let app_up = Router::new()
        .route(
            "/internal/v1/workflows/runs/{run_id}",
            get(|Path(id): Path<String>| async move { Json(json!({"id": id, "unexpected": true})) }),
        )
        .route(
            "/internal/v1/workflows/runs/{run_id}/agents",
            get(|| async { Json(json!([{"agent_id": "test", "unexpected": true}])) }),
        )
        .route(
            "/internal/v1/workflows/runs/{run_id}/events/history",
            get(|| async { Json(json!({"events": [{"unexpected": true}]})) }),
        );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app_up).await.unwrap() });

    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}/agents"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}/events/history"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn misnamed_request_fields_are_422_and_never_persisted() {
    let (url, up) = start_upstream(String::new()).await;
    let (app, db) = app_with_upstream(&url).await;
    let token = register(&app).await;
    let post = |uri: &'static str, body: Value| {
        authed("POST", uri, &token)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    };
    let owner = user_id_of(&app, &token).await;
    let count_workspaces = || async {
        let mut res = db
            .client
            .query("SELECT count() AS n FROM workspace WHERE user_id = type::record('user', $uid) GROUP ALL;")
            .bind(("uid", owner.clone()))
            .await
            .unwrap();
        let rows: Vec<Value> = res.take(0).unwrap();
        rows.first().and_then(|r| r["n"].as_i64()).unwrap_or(0)
    };

    let before = count_workspaces().await;
    for misnamed in ["raw_resume_text", "raw_job_description_text", "preparation_time_weeks", "target_interview_date"] {
        let mut body = json!({"company_name": "Cloudflare", "role_title": "Senior Systems Engineer", "company_urls": []});
        body[misnamed] = json!("x");
        let (status, resp) = send(&app, post("/api/v1/workspaces", body)).await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{misnamed}: {}", String::from_utf8_lossy(&resp));
        assert!(String::from_utf8_lossy(&resp).contains(misnamed), "422 should name the field {misnamed}");
    }
    assert_eq!(count_workspaces().await, before, "a rejected body must not create a workspace");
    assert_eq!(up.hits("POST", "/runs"), 0, "a rejected body must not trigger a run");

    let (status, resp) = send(
        &app,
        Request::post("/api/v1/auth/register")
            .header("content-type", "application/json")
            .body(Body::from(json!({"email": "x@example.com", "password": "Password123!", "name": "X", "role": "admin"}).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "register must reject unknown fields: {}", String::from_utf8_lossy(&resp));
}

fn agent_json(id: &str, run_id: &str) -> Value {
    json!({
        "id": id, "run_id": run_id, "task_id": "task_company", "parent_execution_id": null,
        "child_execution_ids": [], "agent_id": "company-research", "objective": "research",
        "status": "completed", "delegation_depth": 0, "retries": 0, "current_activity": null,
        "model_provider": "ChatOpenAI", "model_name": "m", "budgets": {}, "budgets_used": {},
        "tool_call_count": 2, "notes": [], "artifact_ids": [], "public_summary": "", "error": null,
        "started_at": "2026-10-12T10:00:00Z", "completed_at": "2026-10-12T10:01:00Z"
    })
}

fn tool_json(id: &str, execution_id: &str) -> Value {
    json!({
        "id": id, "execution_id": execution_id, "tool_name": "web_search", "status": "completed",
        "duration_ms": 12.5, "sanitized_input": {"query": "q"}, "output_summary": "ok",
        "output_ref": null, "error": null, "started_at": "2026-10-12T10:00:00Z", "completed_at": null
    })
}

fn event_json(run_id: &str, seq: i64) -> Value {
    json!({
        "schema_version": "1.0", "event_id": format!("evt_{seq}"), "run_id": run_id, "seq": seq,
        "ts": "2026-10-12T10:00:00Z", "type": "task.started", "task_id": "task_company",
        "execution_id": null, "parent_execution_id": null, "tool_invocation_id": null,
        "correlation_id": null, "trace_id": null, "payload": {"task_id": "task_company"}
    })
}

async fn start_detail_upstream() -> (String, Arc<Upstream>) {
    let up = Arc::new(Upstream::default());
    let base = "/internal/v1/workflows/runs/{run_id}";
    let app = Router::new()
        .route(
            base,
            get(|Path(run_id): Path<String>| async move {
                let mut run = run_json(&run_id, "`ws_backticked`", "running");
                run["id"] = json!(format!("workflow_run:`{run_id}`"));
                Json(run)
            }),
        )
        .route(
            &format!("{base}/agents"),
            get(|Path(run_id): Path<String>| async move { Json(json!([agent_json("exec_1", &run_id)])) }),
        )
        .route(
            &format!("{base}/agents/{{execution_id}}"),
            get(|Path((run_id, ex)): Path<(String, String)>| async move { Json(agent_json(&ex, &run_id)) }),
        )
        .route(
            &format!("{base}/agents/{{execution_id}}/tools"),
            get(|Path((_run, ex)): Path<(String, String)>| async move {
                Json(json!({"items": [tool_json("tool_1", &ex)], "next_cursor": null}))
            }),
        )
        .route(
            &format!("{base}/tools/{{tool_id}}/output"),
            get(|Path((_run, tool)): Path<(String, String)>, axum::extract::RawQuery(q): axum::extract::RawQuery| async move {
                let q = q.unwrap_or_default();
                let offset: i64 = url_param(&q, "offset").and_then(|v| v.parse().ok()).unwrap_or(0);
                let limit: i64 = url_param(&q, "limit").and_then(|v| v.parse().ok()).unwrap_or(1000);
                Json(json!({"tool_invocation_id": tool, "offset": offset, "limit": limit,
                            "total_bytes": 5, "has_more": false, "content": "hello"}))
            }),
        )
        .route(
            &format!("{base}/events/history"),
            get(|Path(run_id): Path<String>| async move {
                Json(json!({"events": [event_json(&run_id, 1), event_json(&run_id, 2)], "next_after_seq": null, "last_seq": 2}))
            }),
        )
        .layer(axum::middleware::map_request_with_state(up.clone(), record));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}"), up)
}

#[tokio::test]
async fn new_run_detail_routes_serve_owner_and_hide_from_foreign_users() {
    let (url, up) = start_detail_upstream().await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let token_b = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let routes = [
        format!("/api/v1/workflows/runs/{run_a}/agents"),
        format!("/api/v1/workflows/runs/{run_a}/events/history?after_seq=0&limit=50"),
        format!("/api/v1/workflows/runs/{run_a}/agents/exec_1/tools?limit=10"),
        format!("/api/v1/workflows/runs/{run_a}/tools/tool_1/output?offset=2&limit=3"),
    ];

    for uri in &routes {
        let (status, body) = send(&app, authed("GET", uri, &token_b).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "foreign B: {uri} {}", String::from_utf8_lossy(&body));
    }
    assert!(up.seen().is_empty(), "foreign requests reached upstream: {:?}", up.seen());

    let (status, body) = send(&app, authed("GET", &routes[0], &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let agents: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(agents.as_array().unwrap().len(), 1);
    assert_eq!(agents[0]["tool_call_count"], 2);

    let (status, body) = send(&app, authed("GET", &routes[1], &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let history: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(history["events"].as_array().unwrap().len(), 2);
    assert_eq!(history["last_seq"], 2);
    let hist_call = up.seen().into_iter().find(|s| s.path.ends_with("/events/history")).unwrap();
    assert_eq!(url_param(&hist_call.query, "after_seq").as_deref(), Some("0"));
    assert_eq!(url_param(&hist_call.query, "limit").as_deref(), Some("50"));

    let (status, body) = send(&app, authed("GET", &routes[2], &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let tools: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(tools["items"][0]["id"], "tool_1");

    let (status, body) = send(&app, authed("GET", &routes[3], &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let output: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!((output["offset"].as_i64(), output["limit"].as_i64()), (Some(2), Some(3)));
    assert_eq!(output["content"], "hello");

    assert_upstream_headers_clean(&up, &[&token_a, &token_b]);
}

#[tokio::test]
async fn run_detail_returns_plain_record_ids() {
    let (url, _up) = start_detail_upstream().await;
    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let (status, body) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let run: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(run["id"], json!(run_a));
    assert_eq!(run["workspace_id"], "ws_backticked");
}

#[tokio::test]
async fn tool_output_for_a_different_tool_is_502() {
    let app_up = Router::new().route(
        "/internal/v1/workflows/runs/{run_id}/tools/{tool_id}/output",
        get(|| async {
            Json(json!({"tool_invocation_id": "tool_other", "offset": 0, "limit": 10,
                        "total_bytes": 0, "has_more": false, "content": ""}))
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    tokio::spawn(async move { axum::serve(listener, app_up).await.unwrap() });

    let (app, db) = app_with_upstream(&url).await;
    let token_a = register(&app).await;
    let ws_a = seed_workspace(&db, &token_a, &app).await;
    let run_a = format!("run_{}", uuid::Uuid::new_v4());
    seed_run(&db, &run_a, &ws_a, "2026-10-12T10:00:00Z").await;

    let (status, _) = send(&app, authed("GET", &format!("/api/v1/workflows/runs/{run_a}/tools/tool_1/output"), &token_a).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::BAD_GATEWAY);
}
