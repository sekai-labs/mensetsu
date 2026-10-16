mod mock_grpc_server;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use mensetsu_api::{
    contexts::{
        identity::{
            application::IdentityService,
            domain::{hasher::Argon2PasswordHasher, token::TokenService},
            infrastructure::{SurrealAuthSessionRepository, SurrealUserRepository},
        },
        workspace::infrastructure::db::DatabasePool,
    },
    interfaces::http::{create_router, workspaces::AppState},
    shared::config::AppConfig,
};
use std::sync::Arc;
use tower::ServiceExt;

const TEST_INTERNAL_TOKEN: &str = "test-internal-service-token";

async fn setup_test_app() -> (axum::Router, AppConfig, DatabasePool) {
    let run_json = serde_json::json!({
        "id": "run_test_123",
        "workflow_id": "interview-preparation-hybrid",
        "workflow_name": "Hybrid",
        "workspace_id": "ws_test_123",
        "user_id": "usr_test_123",
        "type": "hybrid",
        "status": "pending",
        "current_phase": "queued",
        "plan_revision": 1,
        "last_seq": 0,
        "created_at": "2026-10-12T10:00:00Z",
        "updated_at": "2026-10-12T10:00:00Z"
    }).to_string();
    let (url, _handle) = mock_grpc_server::start_mock_grpc_server(run_json).await;
    setup_test_app_with_upstream(&url).await
}

async fn setup_test_app_with_upstream(intelligence_url: &str) -> (axum::Router, AppConfig, DatabasePool) {
    let mut config = AppConfig::from_env();
    config.intelligence_grpc_url = intelligence_url.to_string();
    config.internal_service_token = TEST_INTERNAL_TOKEN.to_string();
    let db = DatabasePool::connect(&config)
        .await
        .expect("SurrealDB should connect during test");

    
    let schema_sql = include_str!("../../../infra/migrations/001_initial_schema.surql");
    let _ = db.client.query(schema_sql).await;
    let schema_sql2 = include_str!("../../../infra/migrations/002_user_roles_and_llm_settings.surql");
    let _ = db.client.query(schema_sql2).await;
    let schema_sql3 = include_str!("../../../infra/migrations/003_research_and_jd_intelligence.surql");
    let _ = db.client.query(schema_sql3).await;
    let schema_sql4 = include_str!("../../../infra/migrations/004_workflow_ledger_and_runs.surql");
    let _ = db.client.query(schema_sql4).await;
    let schema_sql5 = include_str!("../../../infra/migrations/005_converge_with_schema_surql.surql");
    let _ = db.client.query(schema_sql5).await;

    let user_repo = Arc::new(SurrealUserRepository::new(db.client.clone()));
    let session_repo = Arc::new(SurrealAuthSessionRepository::new(db.client.clone()));
    let hasher = Arc::new(Argon2PasswordHasher::new());
    let token_service = TokenService::new(
        &config.jwt_secret,
        config.access_token_ttl_seconds,
        config.refresh_token_ttl_seconds,
    );
    let identity_service = Arc::new(IdentityService::new(
        user_repo,
        session_repo,
        hasher,
        token_service,
    ));

    let ws_repo = Arc::new(mensetsu_api::contexts::workspace::infrastructure::SurrealWorkspaceRepository::new((*db.client).clone()));
    let workspace_service = Arc::new(mensetsu_api::contexts::workspace::application::WorkspaceService::new(ws_repo.clone()));

    let wf_repo = Arc::new(mensetsu_api::contexts::workflow::infrastructure::SurrealWorkflowRunRepository::new((*db.client).clone()));
    let workflow_service = Arc::new(mensetsu_api::contexts::workflow::application::WorkflowService::new(wf_repo, ws_repo));

    let grpc_client = Arc::new(
        mensetsu_api::interfaces::grpc::GrpcClient::connect(&config)
            .await
            .expect("GrpcClient should initialize"),
    );

    let state = AppState {
        db: db.clone(),
        config: config.clone(),
        identity_service,
        workspace_service,
        workflow_service,
        grpc_client,
    };

    let app = create_router(state);
    (app, config, db)
}

#[tokio::test]
async fn test_health_and_ready_endpoints() {
    let (app, _, _) = setup_test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_auth_registration_login_refresh_and_me() {
    let (app, _, _) = setup_test_app().await;
    let unique_email = format!("test_{}@example.com", uuid::Uuid::new_v4());

    
    let register_payload = serde_json::json!({
        "email": unique_email,
        "password": "Password123!",
        "name": "Alice Wonderland"
    });

    let reg_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(register_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let reg_status = reg_resp.status();
    let headers = reg_resp.headers().clone();
    let reg_bytes = axum::body::to_bytes(reg_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let reg_body_str = String::from_utf8_lossy(&reg_bytes);
    assert_eq!(reg_status, StatusCode::CREATED, "Register failed body: {}", reg_body_str);
    let set_cookie_header = headers
        .get("set-cookie")
        .expect("Should set cookies")
        .to_str()
        .unwrap();
    assert!(set_cookie_header.contains("mensetsu_access="));

    let reg_json: serde_json::Value = serde_json::from_str(&reg_body_str).unwrap();
    let access_token = reg_json["access_token"].as_str().unwrap();
    let _refresh_token = reg_json["refresh_token"].as_str().unwrap();
    let user_id = reg_json["user"]["id"].as_str().unwrap();
    assert!(!user_id.is_empty());
    
    let me_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/auth/me")
                .header("authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let me_status = me_resp.status();
    let me_bytes = axum::body::to_bytes(me_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let me_body_str = String::from_utf8_lossy(&me_bytes);
    assert_eq!(me_status, StatusCode::OK, "me failed: status={}, body={}", me_status, me_body_str);
    let me_json: serde_json::Value = serde_json::from_slice(&me_bytes).unwrap();
    assert_eq!(me_json["user"]["email"], unique_email);

    
    let login_payload = serde_json::json!({
        "email": unique_email,
        "password": "Password123!"
    });
    let login_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(login_resp.status(), StatusCode::OK);
    let login_bytes = axum::body::to_bytes(login_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_json: serde_json::Value = serde_json::from_slice(&login_bytes).unwrap();
    let login_refresh = login_json["refresh_token"].as_str().unwrap();

    
    let refresh_payload = serde_json::json!({
        "refresh_token": login_refresh
    });
    let refresh_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(refresh_resp.status(), StatusCode::OK);
    let ref_bytes = axum::body::to_bytes(refresh_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let ref_json: serde_json::Value = serde_json::from_slice(&ref_bytes).unwrap();
    let new_refresh_token = ref_json["refresh_token"].as_str().unwrap();
    assert_ne!(new_refresh_token, login_refresh);

    
    let reuse_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reuse_resp.status(), StatusCode::UNAUTHORIZED);

    
    let logout_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .header("authorization", format!("Bearer {access_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_workspace_cross_user_isolation() {
    let (app, _, _) = setup_test_app().await;

    
    let email_a = format!("user_a_{}@example.com", uuid::Uuid::new_v4());
    let reg_a = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": email_a,
                        "password": "Password123!",
                        "name": "User Alpha"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let json_a: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(reg_a.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let token_a = json_a["access_token"].as_str().unwrap();

    
    let email_b = format!("user_b_{}@example.com", uuid::Uuid::new_v4());
    let reg_b = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": email_b,
                        "password": "Password123!",
                        "name": "User Beta"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let json_b: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(reg_b.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    let token_b = json_b["access_token"].as_str().unwrap();

    
    let payload = serde_json::json!({
        "company_name": "Anthropic AI",
        "role_title": "Security & Identity Engineer",
        "company_urls": ["https://anthropic.com"],
        "seniority": "senior",
        "prep_time_weeks": 2,
    });

    let ws_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/workspaces")
                .header("authorization", format!("Bearer {token_a}"))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let ws_status = ws_resp.status();
    let ws_bytes = axum::body::to_bytes(ws_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let ws_body_str = String::from_utf8_lossy(&ws_bytes);
    assert_eq!(ws_status, StatusCode::CREATED, "Workspace create failed: {}", ws_body_str);
    let ws_json: serde_json::Value = serde_json::from_str(&ws_body_str).unwrap();
    let ws_id = ws_json["id"].as_str().unwrap();

    
    let get_a_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/workspaces/{ws_id}"))
                .header("authorization", format!("Bearer {token_a}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_a_resp.status(), StatusCode::OK);

    
    let get_b_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/workspaces/{ws_id}"))
                .header("authorization", format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_b_resp.status(), StatusCode::FORBIDDEN);

    
    let get_unauth_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/workspaces/{ws_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get_unauth_resp.status(), StatusCode::UNAUTHORIZED);

    
    let list_b_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/workspaces")
                .header("authorization", format!("Bearer {token_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_b_resp.status(), StatusCode::OK);
    let list_b_json: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(list_b_resp.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(!list_b_json.iter().any(|w| w["id"] == ws_id));
}

#[tokio::test]
async fn test_llm_settings_encryption_and_sharing() {
    let (app, _, db) = setup_test_app().await;
    
    let admin_email = format!("admin_{}@company.com", uuid::Uuid::new_v4());
    let admin_payload = serde_json::json!({
        "email": admin_email,
        "password": "Password123!",
        "name": "System Administrator"
    });
    let admin_reg = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(admin_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(admin_reg.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(admin_reg.into_body(), usize::MAX).await.unwrap();
    let admin_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_token = admin_auth["access_token"].as_str().unwrap();
    assert_eq!(admin_auth["user"]["role"], "admin");

    
    let llm_upsert = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/settings/llm")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({
                    "provider": "openai",
                    "api_key": "sk-proj-supersecretadminkey1234567890",
                    "model_chat": "gpt-4o",
                    "is_shared": true
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(llm_upsert.status(), StatusCode::OK);
    let body = axum::body::to_bytes(llm_upsert.into_body(), usize::MAX).await.unwrap();
    let res_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(res_json["provider"], "openai");
    assert_eq!(res_json["has_key"], true);
    assert!(res_json["masked_key"].as_str().unwrap().contains("7890"));

    
    let member_email = format!("candidate_{}@gmail.com", uuid::Uuid::new_v4());
    let member_payload = serde_json::json!({
        "email": member_email,
        "password": "Password123!",
        "name": "Jane Candidate"
    });
    let member_reg = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(member_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(member_reg.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(member_reg.into_body(), usize::MAX).await.unwrap();
    let member_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let member_token = member_auth["access_token"].as_str().unwrap();
    assert_eq!(member_auth["user"]["role"], "member");

    
    let member_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/settings/llm")
                .header("authorization", format!("Bearer {member_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(member_get.status(), StatusCode::OK);
    let body = axum::body::to_bytes(member_get.into_body(), usize::MAX).await.unwrap();
    let list: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    let openai_cfg = list.iter().find(|c| c["provider"] == "openai").unwrap();
    assert_eq!(openai_cfg["has_key"], true);
    assert_eq!(openai_cfg["is_inherited"], true);

    let member_key_suffix: String = uuid::Uuid::new_v4().simple().to_string()[..4].to_string();
    for method in ["PUT", "POST"] {
        let member_share = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri("/api/v1/settings/llm")
                    .header("authorization", format!("Bearer {member_token}"))
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::json!({
                        "provider": "anthropic",
                        "api_key": format!("sk-ant-membersecret{member_key_suffix}"),
                        "is_shared": true
                    }).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(member_share.status(), StatusCode::OK, "{method}");
        let body = axum::body::to_bytes(member_share.into_body(), usize::MAX).await.unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(saved["is_shared"], false, "{method}: member share request must be downgraded");
        assert_eq!(saved["is_inherited"], false);
    }

    let member_id = member_auth["user"]["id"].as_str().unwrap().replace("user:", "").replace('`', "");
    let mut stored = db
        .client
        .query("SELECT VALUE is_shared FROM user_llm_config WHERE user_id = type::record('user', $uid) AND provider = 'anthropic';")
        .bind(("uid", member_id))
        .await
        .unwrap();
    let flags: Vec<bool> = stored.take(0).unwrap();
    assert_eq!(flags, vec![false], "exactly one personal, unshared row");

    let other_reg = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({
                    "email": format!("other_{}@gmail.com", uuid::Uuid::new_v4()),
                    "password": "Password123!",
                    "name": "Other Member"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(other_reg.into_body(), usize::MAX).await.unwrap();
    let other_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(other_auth["user"]["role"], "member");
    let other_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/settings/llm")
                .header("authorization", format!("Bearer {}", other_auth["access_token"].as_str().unwrap()))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(other_get.status(), StatusCode::OK);
    let body = axum::body::to_bytes(other_get.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(!text.contains(&member_key_suffix), "other member saw the first member's key: {text}");
    assert!(!text.contains("membersecret"));
}

#[tokio::test]
async fn test_workflow_endpoints_auth_and_routing() {
    let (app, _, _) = setup_test_app().await;

    let unauth_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_res.status(), StatusCode::UNAUTHORIZED);

    let unauth_runs = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/workflows/runs")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauth_runs.status(), StatusCode::UNAUTHORIZED);
}
