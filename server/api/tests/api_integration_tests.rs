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
use tokio::sync::OnceCell;
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
    })
    .to_string();
    let (url, _handle) = mock_grpc_server::start_mock_grpc_server(run_json).await;
    setup_test_app_with_upstream(&url).await
}

async fn setup_test_app_with_grpc_recorder() -> (
    axum::Router,
    AppConfig,
    DatabasePool,
    AppState,
    mock_grpc_server::RecordedGrpcState,
) {
    let (url, grpc_state, _handle) =
        mock_grpc_server::start_dynamic_mock_grpc_server("ws_test".into()).await;
    let (app, config, db, app_state) = setup_test_app_with_upstream_state(&url).await;
    (app, config, db, app_state, grpc_state)
}

async fn setup_test_app_with_upstream(
    intelligence_url: &str,
) -> (axum::Router, AppConfig, DatabasePool) {
    let (app, config, db, _) = setup_test_app_with_upstream_state(intelligence_url).await;
    (app, config, db)
}

async fn setup_test_app_with_upstream_state(
    intelligence_url: &str,
) -> (axum::Router, AppConfig, DatabasePool, AppState) {
    let mut config = AppConfig::from_env();
    config.jwt_secret = "integration_test_jwt_secret_distinct_from_default".to_string();
    config.encryption_key =
        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string();
    config.intelligence_grpc_url = intelligence_url.to_string();
    config.internal_service_token = TEST_INTERNAL_TOKEN.to_string();
    let db = DatabasePool::connect(&config)
        .await
        .expect("SurrealDB should connect during test");

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

    let ws_repo = Arc::new(
        mensetsu_api::contexts::workspace::infrastructure::SurrealWorkspaceRepository::new(
            (*db.client).clone(),
        ),
    );
    let workspace_service = Arc::new(
        mensetsu_api::contexts::workspace::application::WorkspaceService::new(ws_repo.clone()),
    );

    let wf_repo = Arc::new(
        mensetsu_api::contexts::workflow::infrastructure::SurrealWorkflowRunRepository::new(
            (*db.client).clone(),
        ),
    );
    let workflow_service = Arc::new(
        mensetsu_api::contexts::workflow::application::WorkflowService::new(wf_repo, ws_repo),
    );

    let grpc_client = Arc::new(
        mensetsu_api::interfaces::grpc::GrpcClient::connect(&config)
            .await
            .expect("GrpcClient should initialize"),
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
        identity_service,
        oauth_service: None,
        workspace_service,
        workflow_service,
        grpc_client,
        storage,
        metrics: Arc::new(mensetsu_api::interfaces::http::metrics::MetricsCollector::new()),
    };

    let app = create_router(state.clone());
    (app, config, db, state)
}

#[tokio::test]
async fn test_public_registration_never_grants_admin_role() {
    let first_email = format!("admin@{}.attacker.example", uuid::Uuid::new_v4());
    let prefixed_email = format!("admin_{}@attacker.example", uuid::Uuid::new_v4());

    let (app, _, _) = setup_test_app().await;
    for email in [&first_email, &prefixed_email] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/auth/register")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "email": email,
                            "password": "Password123!",
                            "name": "Untrusted Administrator"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(status, StatusCode::CREATED, "registration failed: {body:?}");
        let registration: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(registration["user"]["role"], "member", "{email}");
    }
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
        .oneshot(
            Request::builder()
                .uri("/ready")
                .body(Body::empty())
                .unwrap(),
        )
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
    assert_eq!(
        reg_status,
        StatusCode::CREATED,
        "Register failed body: {}",
        reg_body_str
    );
    let set_cookie_header = headers
        .get("set-cookie")
        .expect("Should set cookies")
        .to_str()
        .unwrap();
    assert!(set_cookie_header.contains("mensetsu_access="));
    assert!(set_cookie_header.contains("Secure"));

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
    assert_eq!(
        me_status,
        StatusCode::OK,
        "me failed: status={}, body={}",
        me_status,
        me_body_str
    );
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
    let (app, _, db) = setup_test_app().await;

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
        "target_date": "2030-01-15T00:00:00Z",
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
    assert_eq!(
        ws_status,
        StatusCode::CREATED,
        "Workspace create failed: {}",
        ws_body_str
    );
    let ws_json: serde_json::Value = serde_json::from_str(&ws_body_str).unwrap();
    let ws_id = ws_json["id"].as_str().unwrap();
    let mut target_date_response = db
        .client
        .query(
            "SELECT VALUE <string>target_date FROM ONLY type::record('workspace', $workspace_id);",
        )
        .bind(("workspace_id", ws_id.to_string()))
        .await
        .unwrap();
    let persisted_target_date: Option<String> = target_date_response.take(0).unwrap();
    assert_eq!(
        chrono::DateTime::parse_from_rfc3339(
            persisted_target_date
                .as_deref()
                .expect("target date persisted")
        )
        .unwrap(),
        chrono::DateTime::parse_from_rfc3339("2030-01-15T00:00:00Z").unwrap()
    );
    let owned_lesson_id = uuid::Uuid::new_v4().to_string();
    db.client
        .query(
            "CREATE type::record('learning_unit', $lesson_id) SET
                workspace_id = type::record('workspace', $workspace_id),
                skill = 'Rust',
                title = 'Owned lesson',
                prerequisites = [],
                estimated_minutes = 5,
                summary = '',
                content_markdown = '',
                completed = false;",
        )
        .bind(("lesson_id", owned_lesson_id))
        .bind(("workspace_id", ws_id.to_string()))
        .await
        .unwrap();
    let qualified_learning = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/workspaces/workspace:{ws_id}/learning"))
                .header("authorization", format!("Bearer {token_a}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(qualified_learning.status(), StatusCode::OK);
    let qualified_lessons: Vec<serde_json::Value> = serde_json::from_slice(
        &axum::body::to_bytes(qualified_learning.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert!(qualified_lessons
        .iter()
        .any(|lesson| lesson["title"] == "Owned lesson"));

    let foreign_workspace_id = uuid::Uuid::new_v4().to_string();
    let foreign_lesson_id = uuid::Uuid::new_v4().to_string();
    let foreign_user_id = json_b["user"]["id"]
        .as_str()
        .unwrap()
        .replace("user:", "")
        .replace('`', "");
    db.client
        .query(
            "CREATE type::record('workspace', $workspace_id) SET
                user_id = type::record('user', $user_id),
                company_name = 'Foreign workspace',
                role_title = 'Engineer',
                status = 'ready',
                current_phase = 'ready',
                progress_percent = 100;",
        )
        .bind(("workspace_id", foreign_workspace_id.clone()))
        .bind(("user_id", foreign_user_id))
        .await
        .unwrap();
    db.client
        .query(
            "CREATE type::record('learning_unit', $lesson_id) SET
                workspace_id = type::record('workspace', $workspace_id),
                skill = 'Rust',
                title = 'Foreign lesson',
                prerequisites = [],
                estimated_minutes = 5,
                summary = '',
                content_markdown = '',
                completed = false;",
        )
        .bind(("lesson_id", foreign_lesson_id.clone()))
        .bind(("workspace_id", foreign_workspace_id))
        .await
        .unwrap();

    let foreign_toggle = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!(
                    "/api/v1/workspaces/{ws_id}/learning/{foreign_lesson_id}/toggle"
                ))
                .header("authorization", format!("Bearer {token_a}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(foreign_toggle.status(), StatusCode::NOT_FOUND);
    let mut completion = db
        .client
        .query("SELECT VALUE completed FROM ONLY type::record('learning_unit', $lesson_id);")
        .bind(("lesson_id", foreign_lesson_id))
        .await
        .unwrap();
    let completed: Option<bool> = completion.take(0).unwrap();
    assert_eq!(completed, Some(false));

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
    let body = axum::body::to_bytes(admin_reg.into_body(), usize::MAX)
        .await
        .unwrap();
    let admin_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(admin_auth["user"]["role"], "member");

    let admin_id = admin_auth["user"]["id"]
        .as_str()
        .unwrap()
        .replace("user:", "")
        .replace('`', "");
    db.client
        .query("UPDATE type::record('user', $uid) SET role = 'admin';")
        .bind(("uid", admin_id))
        .await
        .unwrap();

    let admin_login = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": admin_email,
                        "password": "Password123!"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(admin_login.status(), StatusCode::OK);
    let body = axum::body::to_bytes(admin_login.into_body(), usize::MAX)
        .await
        .unwrap();
    let admin_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let admin_token = admin_auth["access_token"].as_str().unwrap();
    assert_eq!(admin_auth["user"]["role"], "admin");

    let setting_update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/admin/settings")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "audit_round_trip": "saved",
                        "feature_flags": {"workspace_learning": true}
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(setting_update.status(), StatusCode::OK);

    let settings = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/admin/settings")
                .header("authorization", format!("Bearer {admin_token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(settings.status(), StatusCode::OK);
    let settings: serde_json::Value = serde_json::from_slice(
        &axum::body::to_bytes(settings.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(settings["audit_round_trip"], "saved");
    assert_eq!(settings["feature_flags"]["workspace_learning"], true);

    let llm_upsert = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/settings/llm")
                .header("authorization", format!("Bearer {admin_token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "provider": "openai",
                        "api_key": "sk-proj-supersecretadminkey1234567890",
                        "model_chat": "gpt-4o",
                        "is_shared": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(llm_upsert.status(), StatusCode::OK);
    let body = axum::body::to_bytes(llm_upsert.into_body(), usize::MAX)
        .await
        .unwrap();
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
    let body = axum::body::to_bytes(member_reg.into_body(), usize::MAX)
        .await
        .unwrap();
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
    let body = axum::body::to_bytes(member_get.into_body(), usize::MAX)
        .await
        .unwrap();
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
                    .body(Body::from(
                        serde_json::json!({
                            "provider": "anthropic",
                            "api_key": format!("sk-ant-membersecret{member_key_suffix}"),
                            "is_shared": true
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(member_share.status(), StatusCode::OK, "{method}");
        let body = axum::body::to_bytes(member_share.into_body(), usize::MAX)
            .await
            .unwrap();
        let saved: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(
            saved["is_shared"], false,
            "{method}: member share request must be downgraded"
        );
        assert_eq!(saved["is_inherited"], false);
    }

    let member_id = member_auth["user"]["id"]
        .as_str()
        .unwrap()
        .replace("user:", "")
        .replace('`', "");
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
                .body(Body::from(
                    serde_json::json!({
                        "email": format!("other_{}@gmail.com", uuid::Uuid::new_v4()),
                        "password": "Password123!",
                        "name": "Other Member"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let body = axum::body::to_bytes(other_reg.into_body(), usize::MAX)
        .await
        .unwrap();
    let other_auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(other_auth["user"]["role"], "member");
    let other_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/v1/settings/llm")
                .header(
                    "authorization",
                    format!("Bearer {}", other_auth["access_token"].as_str().unwrap()),
                )
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(other_get.status(), StatusCode::OK);
    let body = axum::body::to_bytes(other_get.into_body(), usize::MAX)
        .await
        .unwrap();
    let text = String::from_utf8_lossy(&body);
    assert!(
        !text.contains(&member_key_suffix),
        "other member saw the first member's key: {text}"
    );
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

static APPLICATION_MIGRATIONS: OnceCell<()> = OnceCell::const_new();

async fn ensure_application_migrations(db: &DatabasePool) {
    APPLICATION_MIGRATIONS
        .get_or_init(|| async {
            let migrations = [
                (
                    "006_store_workspace_run_inputs",
                    include_str!("../../../infra/migrations/006_store_workspace_run_inputs.surql"),
                ),
                (
                    "007_workflow_run_degraded",
                    include_str!("../../../infra/migrations/007_workflow_run_degraded.surql"),
                ),
                (
                    "008_llm_provider_routing_cutover",
                    include_str!(
                        "../../../infra/migrations/008_llm_provider_routing_cutover.surql"
                    ),
                ),
                (
                    "009_workflow_run_hybrid_type",
                    include_str!("../../../infra/migrations/009_workflow_run_hybrid_type.surql"),
                ),
                (
                    "010_resume_objects",
                    include_str!("../../../infra/migrations/010_resume_objects.surql"),
                ),
                (
                    "011_drop_resume_base64",
                    include_str!("../../../infra/migrations/011_drop_resume_base64.surql"),
                ),
                (
                    "012_token_usage_event_index",
                    include_str!("../../../infra/migrations/012_token_usage_event_index.surql"),
                ),
                (
                    "013_run_idempotency",
                    include_str!("../../../infra/migrations/013_run_idempotency.surql"),
                ),
                (
                    "014_cv_library",
                    include_str!("../../../infra/migrations/014_cv_library.surql"),
                ),
                (
                    "015_saved_jobs_application_flow",
                    include_str!("../../../infra/migrations/015_saved_jobs_application_flow.surql"),
                ),
                (
                    "016_application_dispatch_and_saved_job_index",
                    include_str!(
                        "../../../infra/migrations/016_application_dispatch_and_saved_job_index.surql"
                    ),
                ),
            ];
            for (version, sql) in migrations {
                db.client
                    .query(sql)
                    .await
                    .unwrap_or_else(|error| panic!("migration {version} query failed: {error}"))
                    .check()
                    .unwrap_or_else(|error| panic!("migration {version} failed: {error}"));
            }
        })
        .await;
}

struct ApplicationTestUser {
    user_id: String,
    token: String,
    upload_id: String,
    upload_filename: String,
}

async fn setup_application_user_and_resume(
    app: &axum::Router,
    db: &DatabasePool,
    name: &str,
    email_prefix: &str,
) -> ApplicationTestUser {
    ensure_application_migrations(db).await;

    let email = format!("{}_{}@example.com", email_prefix, uuid::Uuid::new_v4());
    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": email,
                        "password": "Password123!",
                        "name": name
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(register.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = auth["access_token"].as_str().unwrap().to_string();
    let user_id = auth["user"]["id"]
        .as_str()
        .unwrap()
        .replace("user:", "")
        .replace('`', "");

    let upload_id = uuid::Uuid::new_v4().to_string();
    let upload_filename = "resume.pdf".to_string();
    db.client
        .query("CREATE type::record('resume_upload',$id) SET user_id=type::record('user',$uid),object_key=$key,staging_key=NONE,filename=$filename,size_bytes=1024,content_type='application/pdf',status='completed',created_at=time::now(),updated_at=time::now();")
        .bind(("id", upload_id.clone()))
        .bind(("uid", user_id.clone()))
        .bind(("key", format!("resumes/{upload_id}.pdf")))
        .bind(("filename", upload_filename.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    ApplicationTestUser {
        user_id,
        token,
        upload_id,
        upload_filename,
    }
}

async fn send_app_request(
    app: &axum::Router,
    req: Request<Body>,
) -> (StatusCode, serde_json::Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
        serde_json::json!({
            "raw_text": String::from_utf8_lossy(&bytes).to_string()
        })
    });
    (status, json)
}

fn assert_application_dto_shape(v: &serde_json::Value) {
    assert!(
        v.is_object(),
        "application DTO must be a JSON object: {v:?}"
    );
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .expect("id must be string");
    assert!(!id.is_empty(), "id must not be empty");

    let site = v
        .get("site")
        .and_then(|x| x.as_str())
        .expect("site must be string");
    assert!(
        matches!(site, "greenhouse" | "topcv" | "linkedin"),
        "site must be greenhouse, topcv, or linkedin: {site}"
    );

    let canonical_url = v
        .get("canonical_url")
        .and_then(|x| x.as_str())
        .expect("canonical_url must be string");
    assert!(
        canonical_url.starts_with("https://"),
        "canonical_url must be https: {canonical_url}"
    );

    let source_url = v
        .get("source_url")
        .and_then(|x| x.as_str())
        .expect("source_url must be string");
    assert!(!source_url.is_empty(), "source_url must not be empty");

    assert!(!v
        .get("company_name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());
    assert!(!v
        .get("role_title")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());
    assert!(!v
        .get("job_description")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());

    let resume_upload_id = v
        .get("resume_upload_id")
        .and_then(|x| x.as_str())
        .expect("resume_upload_id must be string");
    assert!(!resume_upload_id.is_empty());

    let resume_filename = v
        .get("resume_filename")
        .and_then(|x| x.as_str())
        .expect("resume_filename must be string");
    assert!(!resume_filename.is_empty());

    let revision = v
        .get("revision")
        .and_then(|x| x.as_str())
        .expect("revision must be string");
    assert_eq!(
        revision.len(),
        64,
        "revision must be 64-character sha256 hex"
    );

    let status = v
        .get("status")
        .and_then(|x| x.as_str())
        .expect("status must be string");
    assert!(
        matches!(
            status,
            "prepared" | "approved" | "submitting" | "submitted" | "needs_review" | "unknown"
        ),
        "status invalid: {status}"
    );

    let fields = v
        .get("prepared_fields")
        .expect("prepared_fields must be present");
    assert!(fields.is_object(), "prepared_fields must be an object");
    assert!(!fields
        .get("first_name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());
    assert!(!fields
        .get("last_name")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());
    assert!(!fields
        .get("email")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .is_empty());

    assert!(
        v.get("created_at").and_then(|x| x.as_str()).is_some(),
        "created_at must be string"
    );
    assert!(
        v.get("updated_at").and_then(|x| x.as_str()).is_some(),
        "updated_at must be string"
    );
}

fn assert_error_dto_shape(v: &serde_json::Value, expected_status: StatusCode) {
    let error = v.get("error").expect("error object must be present");
    assert_eq!(
        error.get("status_code").and_then(|x| x.as_u64()),
        Some(expected_status.as_u16() as u64),
        "status_code in error object mismatch: {v:?}"
    );
    let msg = error
        .get("message")
        .and_then(|x| x.as_str())
        .expect("error message must be string");
    assert!(!msg.is_empty(), "error message must not be empty");
}

#[tokio::test]
async fn application_prepare_approval_and_confirmed_result_are_owner_bound() {
    let (app, _, db) = setup_test_app().await;
    ensure_application_migrations(&db).await;

    let email = format!("application_{}@example.com", uuid::Uuid::new_v4());
    let register = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "email": email, "password": "Password123!", "name": "Alice Wonderland"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(register.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(register.into_body(), usize::MAX)
        .await
        .unwrap();
    let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let token = auth["access_token"].as_str().unwrap();
    let user_id = auth["user"]["id"].as_str().unwrap().replace("user:", "");
    let upload_id = uuid::Uuid::new_v4().to_string();
    db.client.query("CREATE type::record('resume_upload',$id) SET user_id=type::record('user',$uid),object_key=$key,staging_key=NONE,filename='resume.pdf',size_bytes=1024,content_type='application/pdf',status='completed',created_at=time::now(),updated_at=time::now();")
        .bind(("id", upload_id.clone())).bind(("uid", user_id))
        .bind(("key", format!("resumes/{upload_id}.pdf"))).await.unwrap().check().unwrap();

    let payload = serde_json::json!({
        "site":"greenhouse","source_url":"https://boards.greenhouse.io/acme/jobs/123?gh_src=tracking",
        "company_name":"Acme","role_title":"Engineer","job_description":"Build reliable software.",
        "resume_upload_id":upload_id
    });
    let prepared_resp = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/applications")
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = prepared_resp.status();
    let body = axum::body::to_bytes(prepared_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(
        status,
        StatusCode::CREATED,
        "application preparation failed: {}",
        String::from_utf8_lossy(&body)
    );
    let prepared: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_application_dto_shape(&prepared);
    assert_eq!(prepared["status"], "prepared");
    assert_eq!(
        prepared["canonical_url"],
        "https://boards.greenhouse.io/acme/123"
    );
    let id = prepared["id"].as_str().unwrap();
    let revision = prepared["revision"].as_str().unwrap();

    let duplicate = app.clone().oneshot(
        Request::builder().method("POST").uri("/api/v1/applications")
            .header("authorization", format!("Bearer {token}")).header("content-type","application/json")
            .body(Body::from(serde_json::json!({
                "site":"greenhouse","source_url":"https://boards.greenhouse.io/acme/jobs/123?gh_src=tracking",
                "company_name":"Acme","role_title":"Engineer","job_description":"Build reliable software.",
                "resume_upload_id": upload_id
            }).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(duplicate.status(), StatusCode::OK);
    let duplicate_body = axum::body::to_bytes(duplicate.into_body(), usize::MAX)
        .await
        .unwrap();
    let duplicate_json: serde_json::Value = serde_json::from_slice(&duplicate_body).unwrap();
    assert_application_dto_shape(&duplicate_json);
    assert_eq!(duplicate_json["id"], id);
    assert_eq!(duplicate_json["revision"], revision);

    let approve = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/applications/{id}/approve"))
                .header("authorization", format!("Bearer {token}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({"revision":revision}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(approve.status(), StatusCode::OK);
    let approve_body = axum::body::to_bytes(approve.into_body(), usize::MAX)
        .await
        .unwrap();
    let approved_json: serde_json::Value = serde_json::from_slice(&approve_body).unwrap();
    assert_application_dto_shape(&approved_json);
    assert_eq!(approved_json["status"], "approved");

    let start = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/v1/applications/{id}/start"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(start.status(), StatusCode::OK);
    let start_body = axum::body::to_bytes(start.into_body(), usize::MAX)
        .await
        .unwrap();
    let started_json: serde_json::Value = serde_json::from_slice(&start_body).unwrap();
    assert_application_dto_shape(&started_json);
    assert_eq!(started_json["status"], "submitting");

    let result = app.clone().oneshot(
        Request::builder().method("POST").uri(format!("/api/v1/applications/{id}/result"))
            .header("authorization", format!("Bearer {token}")).header("content-type","application/json")
            .body(Body::from(serde_json::json!({
                "outcome":"confirmed","confirmation_url":"https://boards.greenhouse.io/acme/123/confirmation"
            }).to_string())).unwrap()
    ).await.unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let result_body = axum::body::to_bytes(result.into_body(), usize::MAX)
        .await
        .unwrap();
    let result_json: serde_json::Value = serde_json::from_slice(&result_body).unwrap();
    assert_application_dto_shape(&result_json);
    assert_eq!(result_json["status"], "submitted");
    assert!(result_json["workspace_id"].as_str().is_some());

    let get = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/v1/applications/{id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);
    let body = axum::body::to_bytes(get.into_body(), usize::MAX)
        .await
        .unwrap();
    let submitted: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_application_dto_shape(&submitted);
    assert_eq!(submitted["status"], "submitted");
    assert!(submitted["workspace_id"].as_str().is_some());
}

#[tokio::test]
async fn test_application_unauthenticated_and_protected_access() {
    let (app, _, _) = setup_test_app().await;

    let dummy_app_id = uuid::Uuid::new_v4().to_string();
    let dummy_upload_id = uuid::Uuid::new_v4().to_string();

    let unauth_cases = [
        ("GET", "/api/v1/applications".to_string(), None),
        (
            "POST",
            "/api/v1/applications".to_string(),
            Some(serde_json::json!({
                "site": "greenhouse",
                "source_url": "https://boards.greenhouse.io/acme/jobs/101",
                "company_name": "Acme",
                "role_title": "Engineer",
                "job_description": "Build systems.",
                "resume_upload_id": dummy_upload_id
            })),
        ),
        ("GET", format!("/api/v1/applications/{dummy_app_id}"), None),
        (
            "PUT",
            format!("/api/v1/applications/{dummy_app_id}/prepare"),
            Some(serde_json::json!({
                "site": "greenhouse",
                "source_url": "https://boards.greenhouse.io/acme/jobs/101",
                "company_name": "Acme",
                "role_title": "Engineer",
                "job_description": "Build systems.",
                "resume_upload_id": dummy_upload_id
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{dummy_app_id}/approve"),
            Some(serde_json::json!({
                "revision": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{dummy_app_id}/start"),
            None,
        ),
        (
            "POST",
            format!("/api/v1/applications/{dummy_app_id}/block"),
            Some(serde_json::json!({
                "reason_code": "captcha"
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{dummy_app_id}/result"),
            Some(serde_json::json!({
                "outcome": "confirmed",
                "confirmation_url": "https://boards.greenhouse.io/acme/101/confirmation"
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{dummy_app_id}/reconcile"),
            Some(serde_json::json!({
                "submitted": true
            })),
        ),
    ];

    for (method, uri, body) in unauth_cases {
        let mut builder = Request::builder().method(method).uri(&uri);
        let req_body = if let Some(b) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        } else {
            Body::empty()
        };
        let (status, res) = send_app_request(&app, builder.body(req_body).unwrap()).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{method} {uri} should be 401"
        );
        assert_error_dto_shape(&res, StatusCode::UNAUTHORIZED);
    }

    let (status, res) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/applications")
            .header("authorization", "Bearer invalid-jwt-token-or-session")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_error_dto_shape(&res, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_application_prepare_no_workspace_and_duplicate_replay_with_missing_cv() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Bob Developer", "app_prep").await;

    let payload = serde_json::json!({
        "site": "greenhouse",
        "source_url": "https://boards.greenhouse.io/acme/jobs/5001?gh_src=tracking_campaign",
        "company_name": "Acme Global",
        "role_title": "Principal Architect",
        "job_description": "Lead distributed database and cloud platform architecture.",
        "resume_upload_id": user.upload_id
    });

    let (status, prepared) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    assert_application_dto_shape(&prepared);
    assert_eq!(prepared["status"], "prepared");
    assert_eq!(
        prepared["canonical_url"],
        "https://boards.greenhouse.io/acme/5001"
    );
    assert_eq!(prepared["company_name"], "Acme Global");
    assert_eq!(prepared["role_title"], "Principal Architect");
    assert_eq!(prepared["resume_upload_id"], user.upload_id);
    assert_eq!(prepared["resume_filename"], user.upload_filename);
    assert!(
        prepared["workspace_id"].is_null(),
        "prepare must NOT create a workspace"
    );
    assert!(prepared["confirmation_url"].is_null());

    let app_id = prepared["id"].as_str().unwrap().to_string();
    let revision = prepared["revision"].as_str().unwrap().to_string();

    let (get_status, get_body) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/applications/{app_id}"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(get_status, StatusCode::OK);
    assert_application_dto_shape(&get_body);
    assert_eq!(get_body["id"], app_id);
    assert_eq!(get_body["status"], "prepared");
    assert!(
        get_body["workspace_id"].is_null(),
        "GET application must show workspace_id is null"
    );

    let (list_status, list_body) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let list_arr = list_body.as_array().expect("list response must be array");
    let listed_app = list_arr
        .iter()
        .find(|item| item["id"].as_str() == Some(&app_id))
        .expect("application must appear in list");
    assert_application_dto_shape(listed_app);
    assert_eq!(listed_app["revision"], revision);

    let duplicate_payload = serde_json::json!({
        "site": "greenhouse",
        "source_url": "https://boards.greenhouse.io/acme/5001#overview-details",
        "company_name": "Totally Different Company",
        "role_title": "Totally Different Role",
        "job_description": "Totally Different Description",
        "resume_upload_id": user.upload_id
    });

    let (dup_status, dup_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(duplicate_payload.to_string()))
            .unwrap(),
    )
    .await;

    assert_eq!(dup_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&dup_body, StatusCode::CONFLICT);

    let (after_status, after_body) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/applications/{app_id}"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(after_status, StatusCode::OK);
    assert_eq!(after_body["revision"], revision);
    assert_eq!(after_body["role_title"], "Principal Architect");
}

#[tokio::test]
async fn test_application_approval_exact_and_stale_revision_and_single_use_start() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Carol Danvers", "app_rev").await;

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "linkedin",
                    "source_url": "https://www.linkedin.com/jobs/view/901234?trk=ref",
                    "company_name": "Cosmic Innovations",
                    "role_title": "Site Reliability Engineer",
                    "job_description": "Ensure high availability of cloud clusters.",
                    "resume_upload_id": user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    assert_application_dto_shape(&prep_body);
    let app_id = prep_body["id"].as_str().unwrap().to_string();
    let exact_revision = prep_body["revision"].as_str().unwrap().to_string();

    let (stale_status, stale_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "revision": "0000000000000000000000000000000000000000000000000000000000000000"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(stale_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&stale_body, StatusCode::CONFLICT);

    let (_, check_prepared) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/applications/{app_id}"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(check_prepared["status"], "prepared");

    let (appr_status, appr_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "revision": exact_revision
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(appr_status, StatusCode::OK);
    assert_application_dto_shape(&appr_body);
    assert_eq!(appr_body["status"], "approved");
    assert_eq!(appr_body["revision"], exact_revision);
    assert!(appr_body["workspace_id"].is_null());

    let (re_appr_status, re_appr_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "revision": exact_revision
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(re_appr_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&re_appr_body, StatusCode::CONFLICT);

    let (start_status, start_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(start_status, StatusCode::OK);
    assert_application_dto_shape(&start_body);
    assert_eq!(start_body["status"], "submitting");
    assert!(start_body["workspace_id"].is_null());

    let (second_start_status, second_start_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(second_start_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&second_start_body, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_application_confirmed_result_provenance_and_replay_same_workspace() {
    let (app, _, db, api_state, grpc_state) = setup_test_app_with_grpc_recorder().await;
    let user = setup_application_user_and_resume(&app, &db, "David Copperfield", "app_conf").await;

    let _ = db.client.query("UPSERT type::record('candidate_profile', $uid) SET user_id = type::record('user', $uid), phone = '+15550199', location = 'Seattle, WA', linkedin_url = 'https://linkedin.com/in/david', github_url = 'https://github.com/david', portfolio_url = 'https://david.me';")
        .bind(("uid", user.user_id.clone())).await;

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({
                "site": "greenhouse",
                "source_url": "https://boards.greenhouse.io/spacex/jobs/7701?gh_src=outreach",
                "company_name": "SpaceX Starlink",
                "role_title": "Avionics Software Lead",
                "job_description": "Develop low-latency telemetry software for orbital constellations.",
                "resume_upload_id": user.upload_id
            }).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    assert_application_dto_shape(&prep_body);
    let app_id = prep_body["id"].as_str().unwrap().to_string();
    let revision = prep_body["revision"].as_str().unwrap().to_string();

    let (appr_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"revision": revision}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(appr_status, StatusCode::OK);

    let (start_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(start_status, StatusCode::OK);

    let result_request = || {
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/result"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "outcome": "confirmed",
                    "confirmation_url": "https://boards.greenhouse.io/spacex/7701/confirmation?ref=done"
                })
                .to_string(),
            ))
            .unwrap()
    };
    let ((res_status, res_body), (duplicate_status, duplicate_body)) = tokio::join!(
        send_app_request(&app, result_request()),
        send_app_request(&app, result_request())
    );
    assert_eq!(res_status, StatusCode::OK);
    assert_eq!(duplicate_status, StatusCode::OK);
    assert_application_dto_shape(&res_body);
    assert_eq!(res_body["status"], "submitted");
    assert_eq!(duplicate_body["status"], "submitted");
    assert_application_dto_shape(&duplicate_body);
    let ws_id = res_body["workspace_id"]
        .as_str()
        .expect("workspace_id must be Some")
        .to_string();
    assert_eq!(
        duplicate_body["workspace_id"].as_str(),
        Some(ws_id.as_str())
    );
    assert_eq!(
        res_body["confirmation_url"].as_str().unwrap(),
        "https://boards.greenhouse.io/spacex/7701/confirmation"
    );

    let calls_before_delivery = grpc_state
        .received_commands
        .lock()
        .iter()
        .filter(|command| *command == "create_run")
        .count();
    assert_eq!(
        calls_before_delivery, 0,
        "confirmation only enqueues dispatch"
    );

    *grpc_state.create_run_failures.lock() = 1;
    assert_eq!(
        mensetsu_api::interfaces::http::application_dispatch::drain_once_for_workspace(
            &api_state, &ws_id
        )
        .await
        .unwrap(),
        0,
        "a failed CreateRun must remain retryable"
    );
    let mut failed_dispatch = db
        .client
        .query(
            "SELECT state, attempts, last_error FROM application_dispatch WHERE workspace_id = type::record('workspace', $ws_id);",
        )
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let failed_dispatch_rows: Vec<serde_json::Value> = failed_dispatch.take(0).unwrap();
    assert_eq!(failed_dispatch_rows.len(), 1);
    assert_eq!(failed_dispatch_rows[0]["state"], "pending");
    assert_eq!(failed_dispatch_rows[0]["attempts"], 1);
    assert!(!failed_dispatch_rows[0]["last_error"].is_null());

    db.client
        .query(
            "UPDATE application_dispatch SET due_at = time::now() WHERE workspace_id = type::record('workspace', $ws_id);",
        )
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();
    assert_eq!(
        mensetsu_api::interfaces::http::application_dispatch::drain_once_for_workspace(
            &api_state, &ws_id
        )
        .await
        .unwrap(),
        1
    );
    let mut delivered_dispatch = db
        .client
        .query(
            "SELECT state, attempts, run_id FROM application_dispatch WHERE workspace_id = type::record('workspace', $ws_id);",
        )
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let delivered_dispatch_rows: Vec<serde_json::Value> = delivered_dispatch.take(0).unwrap();
    assert_eq!(delivered_dispatch_rows.len(), 1);
    assert_eq!(delivered_dispatch_rows[0]["state"], "delivered");
    assert_eq!(delivered_dispatch_rows[0]["attempts"], 1);
    assert!(!delivered_dispatch_rows[0]["run_id"].is_null());

    let initial_runs = grpc_state
        .received_commands
        .lock()
        .iter()
        .filter(|command| *command == "create_run")
        .count();
    assert_eq!(initial_runs, 2, "one failure and one successful retry");

    let mut ws_q = db
        .client
        .query("SELECT * FROM type::record('workspace', $ws_id);")
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let ws_rows: Vec<serde_json::Value> = ws_q.take(0).unwrap();
    assert_eq!(ws_rows.len(), 1, "exactly one workspace must exist");
    assert_eq!(ws_rows[0]["company_name"], "SpaceX Starlink");
    assert_eq!(ws_rows[0]["role_title"], "Avionics Software Lead");
    let ws_urls = ws_rows[0]["company_urls"].as_array().unwrap();
    assert!(ws_urls.iter().any(
        |u| u.as_str() == Some("https://boards.greenhouse.io/spacex/jobs/7701?gh_src=outreach")
    ));

    let mut jd_q = db
        .client
        .query(
            "SELECT * FROM job_description WHERE workspace_id = type::record('workspace', $ws_id);",
        )
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let jd_rows: Vec<serde_json::Value> = jd_q.take(0).unwrap();
    assert_eq!(jd_rows.len(), 1, "exactly one job description must exist");
    assert_eq!(
        jd_rows[0]["title"], "Avionics Software Lead",
        "source_title must equal role_title"
    );
    assert_eq!(
        jd_rows[0]["source_url"],
        "https://boards.greenhouse.io/spacex/jobs/7701?gh_src=outreach"
    );
    assert_eq!(
        jd_rows[0]["canonical_url"],
        "https://boards.greenhouse.io/spacex/7701"
    );
    assert!(
        !jd_rows[0]["retrieved_at"].is_null(),
        "retrieved_at provenance must be recorded"
    );
    assert_eq!(
        jd_rows[0]["extraction_method"], "extension_capture",
        "extraction_method must be extension_capture"
    );

    let mut resume_q = db
        .client
        .query("SELECT * FROM resume WHERE workspace_id = type::record('workspace', $ws_id);")
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let resume_rows: Vec<serde_json::Value> = resume_q.take(0).unwrap();
    assert_eq!(resume_rows.len(), 1, "exactly one resume must exist");
    let linked_upload = resume_rows[0]["source_upload_id"].as_str().unwrap();
    assert!(
        linked_upload.contains(&user.upload_id),
        "source_upload_id must link to user's upload_id"
    );
    assert_eq!(resume_rows[0]["source_filename"], user.upload_filename);

    let mut app_q = db
        .client
        .query("SELECT * FROM type::record('job_application', $app_id);")
        .bind(("app_id", app_id.clone()))
        .await
        .unwrap();
    let app_rows: Vec<serde_json::Value> = app_q.take(0).unwrap();
    assert_eq!(app_rows.len(), 1);
    assert_eq!(app_rows[0]["source_title"], "Avionics Software Lead");
    assert_eq!(
        app_rows[0]["canonical_url"],
        "https://boards.greenhouse.io/spacex/7701"
    );
    assert_eq!(
        app_rows[0]["source_url"],
        "https://boards.greenhouse.io/spacex/jobs/7701?gh_src=outreach"
    );
    assert!(!app_rows[0]["retrieved_at"].is_null());

    let (replay_status, replay_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/result"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "outcome": "confirmed",
                    "confirmation_url": "https://boards.greenhouse.io/spacex/7701/confirmation"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(replay_status, StatusCode::OK);
    assert_application_dto_shape(&replay_body);
    assert_eq!(replay_body["status"], "submitted");
    assert_eq!(
        replay_body["workspace_id"].as_str().unwrap(),
        ws_id,
        "replay must return identical workspace_id"
    );

    assert_eq!(
        mensetsu_api::interfaces::http::application_dispatch::drain_once_for_workspace(
            &api_state, &ws_id
        )
        .await
        .unwrap(),
        0,
        "a delivered dispatch must not be replayed"
    );
    let replay_runs = grpc_state
        .received_commands
        .lock()
        .iter()
        .filter(|c| *c == "create_run")
        .count();
    assert_eq!(
        replay_runs, 2,
        "confirmed replay after recovery must NOT trigger additional CreateRun commands"
    );

    let mut total_ws = db.client.query("SELECT count() AS cnt FROM workspace WHERE user_id = type::record('user', $uid) GROUP ALL;")
        .bind(("uid", user.user_id.clone())).await.unwrap();
    let total_rows: Vec<serde_json::Value> = total_ws.take(0).unwrap();
    let count = total_rows
        .first()
        .and_then(|r| r.get("cnt"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0);
    assert_eq!(count, 1, "exactly one workspace must exist after replay");
}

#[tokio::test]
async fn test_application_uncertain_result_no_workspace_then_false_reconcile_needs_review() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Elena Rostova", "app_uncert").await;

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(serde_json::json!({
                "site": "topcv",
                "source_url": "https://www.topcv.vn/viec-lam/88888-software-engineer.html?utm_source=fb",
                "company_name": "Fintech Vietnam",
                "role_title": "Backend Go Developer",
                "job_description": "Build resilient payment gateway infrastructure.",
                "resume_upload_id": user.upload_id
            }).to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    assert_application_dto_shape(&prep_body);
    let app_id = prep_body["id"].as_str().unwrap().to_string();
    let revision = prep_body["revision"].as_str().unwrap().to_string();

    let (appr_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"revision": revision}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(appr_status, StatusCode::OK);

    let (start_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(start_status, StatusCode::OK);

    let (res_status, res_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/result"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "outcome": "uncertain",
                    "confirmation_url": "https://www.topcv.vn/viec-lam/88888-software-engineer.html"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(res_status, StatusCode::OK);
    assert_application_dto_shape(&res_body);
    assert_eq!(res_body["status"], "unknown");
    assert!(
        res_body["workspace_id"].is_null(),
        "uncertain outcome must NOT create a workspace"
    );

    let mut ws_q = db
        .client
        .query("SELECT * FROM workspace WHERE user_id = type::record('user', $uid);")
        .bind(("uid", user.user_id.clone()))
        .await
        .unwrap();
    let ws_rows: Vec<serde_json::Value> = ws_q.take(0).unwrap();
    assert!(
        ws_rows.is_empty(),
        "no workspace should exist after uncertain outcome"
    );

    let (rec_status, rec_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/reconcile"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "submitted": false
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(rec_status, StatusCode::OK);
    assert_application_dto_shape(&rec_body);
    assert_eq!(rec_body["status"], "needs_review");
    assert!(rec_body["workspace_id"].is_null());

    let (reprep_status, reprep_body) = send_app_request(
        &app,
        Request::builder()
            .method("PUT")
            .uri(format!("/api/v1/applications/{app_id}/prepare"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "topcv",
                    "source_url": "https://www.topcv.vn/viec-lam/88888-software-engineer.html",
                    "company_name": "Fintech Vietnam Updated",
                    "role_title": "Senior Backend Go Developer",
                    "job_description": "Updated job description for resilience.",
                    "resume_upload_id": user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(reprep_status, StatusCode::OK);
    assert_application_dto_shape(&reprep_body);
    assert_eq!(reprep_body["status"], "prepared");
    assert_eq!(reprep_body["company_name"], "Fintech Vietnam Updated");
    assert!(reprep_body["workspace_id"].is_null());
}

#[tokio::test]
async fn test_application_unknown_positive_reconcile_creates_one_workspace() {
    let (app, _, db, api_state, grpc_state) = setup_test_app_with_grpc_recorder().await;
    let user = setup_application_user_and_resume(&app, &db, "Felix Baum", "app_pos_rec").await;

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "linkedin",
                    "source_url": "https://www.linkedin.com/jobs/view/456789?context=search",
                    "company_name": "Nebula Cloud",
                    "role_title": "Infrastructure Engineer",
                    "job_description": "Build high-throughput networking for AI superclusters.",
                    "resume_upload_id": user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    let app_id = prep_body["id"].as_str().unwrap().to_string();
    let revision = prep_body["revision"].as_str().unwrap().to_string();

    send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"revision": revision}).to_string(),
            ))
            .unwrap(),
    )
    .await;

    send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    let (res_status, res_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/result"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "outcome": "uncertain",
                    "confirmation_url": "https://www.linkedin.com/jobs/view/456789"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(res_status, StatusCode::OK);
    assert_eq!(res_body["status"], "unknown");
    assert!(res_body["workspace_id"].is_null());

    let (rec_status, rec_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/reconcile"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "submitted": true
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(rec_status, StatusCode::OK);
    assert_application_dto_shape(&rec_body);
    assert_eq!(rec_body["status"], "submitted");
    let ws_id = rec_body["workspace_id"]
        .as_str()
        .expect("workspace_id must be present")
        .to_string();

    assert_eq!(
        mensetsu_api::interfaces::http::application_dispatch::drain_once_for_workspace(
            &api_state, &ws_id
        )
        .await
        .unwrap(),
        1
    );
    let initial_runs = grpc_state
        .received_commands
        .lock()
        .iter()
        .filter(|command| *command == "create_run")
        .count();
    assert_eq!(initial_runs, 1);

    let mut ws_q = db
        .client
        .query("SELECT * FROM type::record('workspace', $ws_id);")
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let ws_rows: Vec<serde_json::Value> = ws_q.take(0).unwrap();
    assert_eq!(ws_rows.len(), 1);
    assert_eq!(ws_rows[0]["company_name"], "Nebula Cloud");
    assert_eq!(ws_rows[0]["role_title"], "Infrastructure Engineer");

    let mut jd_q = db
        .client
        .query(
            "SELECT * FROM job_description WHERE workspace_id = type::record('workspace', $ws_id);",
        )
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let jd_rows: Vec<serde_json::Value> = jd_q.take(0).unwrap();
    assert_eq!(jd_rows.len(), 1);
    assert_eq!(jd_rows[0]["title"], "Infrastructure Engineer");

    let mut res_q = db
        .client
        .query("SELECT * FROM resume WHERE workspace_id = type::record('workspace', $ws_id);")
        .bind(("ws_id", ws_id.clone()))
        .await
        .unwrap();
    let resume_rows: Vec<serde_json::Value> = res_q.take(0).unwrap();
    assert_eq!(resume_rows.len(), 1);
    assert!(resume_rows[0]["source_upload_id"]
        .as_str()
        .unwrap()
        .contains(&user.upload_id));
    assert_eq!(
        resume_rows[0]["source_object_key"],
        format!("resumes/{}.pdf", user.upload_id)
    );

    let (re_rec_status, re_rec_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/reconcile"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "submitted": true
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(re_rec_status, StatusCode::OK);
    assert_application_dto_shape(&re_rec_body);
    assert_eq!(re_rec_body["status"], "submitted");
    assert_eq!(re_rec_body["workspace_id"].as_str().unwrap(), ws_id);

    assert_eq!(
        mensetsu_api::interfaces::http::application_dispatch::drain_once_for_workspace(
            &api_state, &ws_id
        )
        .await
        .unwrap(),
        0
    );
    let replay_runs = grpc_state
        .received_commands
        .lock()
        .iter()
        .filter(|command| *command == "create_run")
        .count();
    assert_eq!(
        replay_runs, 1,
        "positive reconcile replay must not send another CreateRun"
    );

    let mut total_ws = db.client.query("SELECT count() AS cnt FROM workspace WHERE user_id = type::record('user', $uid) GROUP ALL;")
        .bind(("uid", user.user_id.clone())).await.unwrap();
    let total_rows: Vec<serde_json::Value> = total_ws.take(0).unwrap();
    let count = total_rows
        .first()
        .and_then(|r| r.get("cnt"))
        .and_then(|c| c.as_u64())
        .unwrap_or(0);
    assert_eq!(
        count, 1,
        "exactly one workspace must exist after reconcile replay"
    );
}

#[tokio::test]
async fn test_application_deleted_or_foreign_cv() {
    let (app, _, db) = setup_test_app().await;
    let user_a = setup_application_user_and_resume(&app, &db, "Grace Hopper", "app_cv_a").await;
    let user_b = setup_application_user_and_resume(&app, &db, "Alan Turing", "app_cv_b").await;

    let (malformed_status, malformed_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/3001",
                    "company_name": "Acme",
                    "role_title": "Compiler Engineer",
                    "job_description": "Build modern compilers.",
                    "resume_upload_id": "not-a-valid-uuid"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(malformed_status, StatusCode::BAD_REQUEST);
    assert_error_dto_shape(&malformed_body, StatusCode::BAD_REQUEST);

    let (missing_status, missing_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/3001",
                    "company_name": "Acme",
                    "role_title": "Compiler Engineer",
                    "job_description": "Build modern compilers.",
                    "resume_upload_id": uuid::Uuid::new_v4().to_string()
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(missing_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&missing_body, StatusCode::CONFLICT);

    let (foreign_cv_status, foreign_cv_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/3001",
                    "company_name": "Acme",
                    "role_title": "Compiler Engineer",
                    "job_description": "Build modern compilers.",
                    "resume_upload_id": user_b.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(foreign_cv_status, StatusCode::FORBIDDEN);
    assert_error_dto_shape(&foreign_cv_body, StatusCode::FORBIDDEN);

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/3001",
                    "company_name": "Acme",
                    "role_title": "Compiler Engineer",
                    "job_description": "Build modern compilers.",
                    "resume_upload_id": user_a.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    let app_id = prep_body["id"].as_str().unwrap().to_string();
    let revision = prep_body["revision"].as_str().unwrap().to_string();

    db.client
        .query("UPDATE type::record('resume_upload', $id) SET status = 'deleting';")
        .bind(("id", user_a.upload_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let (del_appr_status, del_appr_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"revision": revision}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(del_appr_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&del_appr_body, StatusCode::CONFLICT);

    db.client
        .query("UPDATE type::record('resume_upload', $id) SET status = 'completed';")
        .bind(("id", user_a.upload_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let (ok_appr_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/approve"))
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"revision": revision}).to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(ok_appr_status, StatusCode::OK);

    db.client
        .query("UPDATE type::record('resume_upload', $id) SET status = 'deleting';")
        .bind(("id", user_a.upload_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let (del_start_status, del_start_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/start"))
            .header("authorization", format!("Bearer {}", user_a.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(del_start_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&del_start_body, StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_application_foreign_app_owner_404_and_isolation() {
    let (app, _, db) = setup_test_app().await;
    let user_a = setup_application_user_and_resume(&app, &db, "User Alpha", "app_iso_a").await;
    let user_b = setup_application_user_and_resume(&app, &db, "User Beta", "app_iso_b").await;

    let (prep_status, prep_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_a.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/alpha/jobs/1111",
                    "company_name": "Alpha Corp",
                    "role_title": "Security Lead",
                    "job_description": "Implement zero-trust security.",
                    "resume_upload_id": user_a.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(prep_status, StatusCode::CREATED);
    let app_id_a = prep_body["id"].as_str().unwrap().to_string();
    let revision_a = prep_body["revision"].as_str().unwrap().to_string();

    let foreign_attempts = [
        ("GET", format!("/api/v1/applications/{app_id_a}"), None),
        (
            "PUT",
            format!("/api/v1/applications/{app_id_a}/prepare"),
            Some(serde_json::json!({
                "site": "greenhouse",
                "source_url": "https://boards.greenhouse.io/alpha/jobs/1111",
                "company_name": "Alpha Corp",
                "role_title": "Security Lead",
                "job_description": "Hijack application.",
                "resume_upload_id": user_b.upload_id
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{app_id_a}/approve"),
            Some(serde_json::json!({
                "revision": revision_a
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{app_id_a}/start"),
            None,
        ),
        (
            "POST",
            format!("/api/v1/applications/{app_id_a}/block"),
            Some(serde_json::json!({
                "reason_code": "captcha"
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{app_id_a}/result"),
            Some(serde_json::json!({
                "outcome": "confirmed",
                "confirmation_url": "https://boards.greenhouse.io/alpha/1111/confirmation"
            })),
        ),
        (
            "POST",
            format!("/api/v1/applications/{app_id_a}/reconcile"),
            Some(serde_json::json!({
                "submitted": true
            })),
        ),
    ];

    for (method, uri, body) in foreign_attempts {
        let mut builder = Request::builder()
            .method(method)
            .uri(&uri)
            .header("authorization", format!("Bearer {}", user_b.token));
        let req_body = if let Some(b) = body {
            builder = builder.header("content-type", "application/json");
            Body::from(b.to_string())
        } else {
            Body::empty()
        };
        let (status, res) = send_app_request(&app, builder.body(req_body).unwrap()).await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "{method} {uri} by foreign user must return 404"
        );
        assert_error_dto_shape(&res, StatusCode::NOT_FOUND);
    }

    let (list_status, list_body) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user_b.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(list_status, StatusCode::OK);
    let list_arr = list_body.as_array().expect("list must be array");
    assert!(
        !list_arr
            .iter()
            .any(|item| item["id"].as_str() == Some(&app_id_a)),
        "foreign application must not appear in list"
    );

    let (not_found_status, not_found_body) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/applications/{}", uuid::Uuid::new_v4()))
            .header("authorization", format!("Bearer {}", user_b.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(not_found_status, StatusCode::NOT_FOUND);
    assert_error_dto_shape(&not_found_body, StatusCode::NOT_FOUND);

    let (malformed_app_status, malformed_app_body) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/applications/not-a-valid-uuid")
            .header("authorization", format!("Bearer {}", user_b.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(malformed_app_status, StatusCode::BAD_REQUEST);
    assert_error_dto_shape(&malformed_app_body, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_application_unsupported_private_host_and_malformed_profile_validation() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Valid Name", "app_val").await;

    let bad_urls = [
        (
            "http://boards.greenhouse.io/acme/jobs/123",
            "non-https scheme",
        ),
        (
            "https://evil.internal.attacker.com/acme/jobs/123",
            "unsupported host",
        ),
        ("https://127.0.0.1/acme/jobs/123", "loopback IPv4 host"),
        ("https://localhost/acme/jobs/123", "localhost host"),
        (
            "https://user:pass@boards.greenhouse.io/acme/jobs/123",
            "embedded credentials",
        ),
        (
            "https://boards.greenhouse.io:8443/acme/jobs/123",
            "custom port",
        ),
        ("https://boards.greenhouse.io/acme", "missing job ID"),
        (
            "https://linkedin.com/jobs/search?q=rust",
            "linkedin search instead of view/{id}",
        ),
        ("https://topcv.vn/company/about", "topcv non-job path"),
    ];

    for (url, reason) in bad_urls {
        let (status, res) = send_app_request(
            &app,
            Request::builder()
                .method("POST")
                .uri("/api/v1/applications")
                .header("authorization", format!("Bearer {}", user.token))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::json!({
                    "site": if url.contains("linkedin") { "linkedin" } else if url.contains("topcv") { "topcv" } else { "greenhouse" },
                    "source_url": url,
                    "company_name": "Test Company",
                    "role_title": "Test Role",
                    "job_description": "Valid job description.",
                    "resume_upload_id": user.upload_id
                }).to_string()))
                .unwrap(),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "URL should be rejected ({reason}): {url}"
        );
        assert_error_dto_shape(&res, StatusCode::BAD_REQUEST);
    }

    let huge_url = format!(
        "https://boards.greenhouse.io/acme/jobs/123?{}",
        "x=".repeat(1100)
    );
    let (huge_status, huge_res) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": huge_url,
                    "company_name": "Test Company",
                    "role_title": "Test Role",
                    "job_description": "Valid job description.",
                    "resume_upload_id": user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(huge_status, StatusCode::BAD_REQUEST);
    assert_error_dto_shape(&huge_res, StatusCode::BAD_REQUEST);

    let mononym_user = setup_application_user_and_resume(&app, &db, "Cher", "app_mono").await;
    let (mono_status, mono_res) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", mononym_user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/999",
                    "company_name": "Acme",
                    "role_title": "Solo Singer",
                    "job_description": "Sing songs.",
                    "resume_upload_id": mononym_user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(
        mono_status,
        StatusCode::BAD_REQUEST,
        "single word profile name must fail with 400 Bad Request"
    );
    assert_error_dto_shape(&mono_res, StatusCode::BAD_REQUEST);
    assert!(
        mono_res["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Profile lacks required application fields"),
        "error message must describe profile field requirement: {mono_res:?}"
    );

    let empty_checks = [
        ("", "Role Title", "Description", "empty company"),
        ("Company", "   ", "Description", "whitespace role"),
        ("Company", "Role", "", "empty description"),
    ];
    for (company, role, jd, reason) in empty_checks {
        let (status, res) = send_app_request(
            &app,
            Request::builder()
                .method("POST")
                .uri("/api/v1/applications")
                .header("authorization", format!("Bearer {}", user.token))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "site": "greenhouse",
                        "source_url": "https://boards.greenhouse.io/acme/jobs/999",
                        "company_name": company,
                        "role_title": role,
                        "job_description": jd,
                        "resume_upload_id": user.upload_id
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{reason} must return 400");
        assert_error_dto_shape(&res, StatusCode::BAD_REQUEST);
    }

    let (ok_prep_status, ok_prep) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "site": "greenhouse",
                    "source_url": "https://boards.greenhouse.io/acme/jobs/12345",
                    "company_name": "Acme Blocking",
                    "role_title": "QA Lead",
                    "job_description": "Validate edge cases.",
                    "resume_upload_id": user.upload_id
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(ok_prep_status, StatusCode::CREATED);
    let app_id = ok_prep["id"].as_str().unwrap();

    let (block_status, blocked) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/block"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "reason_code": "captcha"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(block_status, StatusCode::OK);
    assert_application_dto_shape(&blocked);
    assert_eq!(blocked["status"], "needs_review");

    let (reblock_status, reblock_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/applications/{app_id}/block"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "reason_code": "form_changed"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(reblock_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&reblock_body, StatusCode::CONFLICT);
}

#[tokio::test]
async fn resume_upload_listing_is_owner_scoped_and_completed_only() {
    let (app, _, db) = setup_test_app().await;
    let owner = setup_application_user_and_resume(&app, &db, "Grace Hopper", "cv_list_owner").await;
    let other =
        setup_application_user_and_resume(&app, &db, "Katherine Johnson", "cv_list_other").await;
    let pending_id = uuid::Uuid::new_v4().to_string();
    db.client
        .query("CREATE type::record('resume_upload',$id) SET user_id=type::record('user',$uid),object_key=$key,staging_key=$key,filename='pending.pdf',size_bytes=1024,content_type='application/pdf',status='pending',created_at=time::now(),updated_at=time::now();")
        .bind(("id", pending_id.clone()))
        .bind(("uid", owner.user_id.clone()))
        .bind(("key", format!("resumes/staging/{pending_id}.pdf")))
        .await
        .unwrap()
        .check()
        .unwrap();

    let (status, uploads) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/resume-uploads")
            .header("authorization", format!("Bearer {}", owner.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let uploads = uploads.as_array().expect("upload list must be an array");
    assert_eq!(uploads.len(), 1);
    assert_eq!(uploads[0]["upload_id"], owner.upload_id);
    assert_eq!(uploads[0]["filename"], "resume.pdf");
    assert_ne!(uploads[0]["upload_id"], other.upload_id);
    assert_ne!(uploads[0]["upload_id"], pending_id);

    let (unauthenticated, error) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/resume-uploads")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(unauthenticated, StatusCode::UNAUTHORIZED);
    assert_error_dto_shape(&error, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_completed_resume_upload_is_reusable_across_manual_workspaces() {
    let (app, _, db) = setup_test_app().await;
    let user =
        setup_application_user_and_resume(&app, &db, "Katherine Johnson", "manual_cv_reuse").await;

    // 1. Malformed upload ID rejection (400 Bad Request)
    let (malformed_status, malformed_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "company_name": "Test Co",
                    "role_title": "Engineer",
                    "company_urls": ["https://example.com"],
                    "workflow_mode": "hybrid",
                    "resume_upload_id": "not-a-valid-uuid",
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(malformed_status, StatusCode::BAD_REQUEST);
    assert_error_dto_shape(&malformed_body, StatusCode::BAD_REQUEST);

    // 2. Missing upload ID rejection (404 Not Found)
    let missing_uuid = uuid::Uuid::new_v4().to_string();
    let (missing_status, missing_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "company_name": "Test Co",
                    "role_title": "Engineer",
                    "company_urls": ["https://example.com"],
                    "workflow_mode": "hybrid",
                    "resume_upload_id": missing_uuid,
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(missing_status, StatusCode::NOT_FOUND);
    assert_error_dto_shape(&missing_body, StatusCode::NOT_FOUND);

    // 3. Foreign upload ID rejection (403 Forbidden)
    let user_b =
        setup_application_user_and_resume(&app, &db, "Dorothy Vaughan", "manual_cv_b").await;
    let (foreign_status, foreign_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces")
            .header("authorization", format!("Bearer {}", user_b.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "company_name": "Test Co",
                    "role_title": "Engineer",
                    "company_urls": ["https://example.com"],
                    "workflow_mode": "hybrid",
                    "resume_upload_id": user.upload_id,
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(foreign_status, StatusCode::FORBIDDEN);
    assert_error_dto_shape(&foreign_body, StatusCode::FORBIDDEN);

    // 4. Deleted/non-completed upload ID rejection (409 Conflict)
    db.client
        .query("UPDATE type::record('resume_upload', $id) SET status = 'deleting';")
        .bind(("id", user_b.upload_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();
    let (del_status, del_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/workspaces")
            .header("authorization", format!("Bearer {}", user_b.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "company_name": "Test Co",
                    "role_title": "Engineer",
                    "company_urls": ["https://example.com"],
                    "workflow_mode": "hybrid",
                    "resume_upload_id": user_b.upload_id,
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(del_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&del_body, StatusCode::CONFLICT);

    // 5. Sequential reuse across multiple workspaces for completed upload
    let mut workspace_ids = Vec::new();

    for (company, role) in [
        ("Orbit Dynamics", "Flight Software Engineer"),
        ("Northstar Navigation", "Distributed Systems Engineer"),
    ] {
        let (status, body) = send_app_request(
            &app,
            Request::builder()
                .method("POST")
                .uri("/api/v1/workspaces")
                .header("authorization", format!("Bearer {}", user.token))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "company_name": company,
                        "role_title": role,
                        "company_urls": ["https://example.com"],
                        "workflow_mode": "hybrid",
                        "resume_upload_id": user.upload_id,
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await;
        assert_eq!(
            status,
            StatusCode::CREATED,
            "workspace creation failed: {body:?}"
        );
        let workspace_id = body["id"]
            .as_str()
            .expect("workspace ID is returned")
            .to_string();
        workspace_ids.push(workspace_id);
    }

    assert_ne!(workspace_ids[0], workspace_ids[1]);
    for workspace_id in &workspace_ids {
        let mut resume = db.client
            .query("SELECT source_upload_id FROM resume WHERE workspace_id = type::record('workspace', $id);")
            .bind(("id", workspace_id.clone()))
            .await
            .unwrap();
        let rows: Vec<serde_json::Value> = resume.take(0).unwrap();
        assert_eq!(
            rows.len(),
            1,
            "each workspace must have its own resume record"
        );
        assert!(
            serde_json::to_string(&rows[0]["source_upload_id"])
                .unwrap()
                .contains(&user.upload_id),
            "each resume must link to the same library upload"
        );
    }

    let mut upload = db
        .client
        .query("SELECT status FROM type::record('resume_upload', $id);")
        .bind(("id", user.upload_id.clone()))
        .await
        .unwrap();
    let upload_rows: Vec<serde_json::Value> = upload.take(0).unwrap();
    assert_eq!(upload_rows[0]["status"], "completed");

    let mut linked_resumes = db.client
        .query("SELECT count() AS count FROM resume WHERE source_upload_id = type::record('resume_upload', $id) GROUP ALL;")
        .bind(("id", user.upload_id))
        .await
        .unwrap();
    let linked_rows: Vec<serde_json::Value> = linked_resumes.take(0).unwrap();
    let count = linked_rows
        .first()
        .and_then(|row| row["count"].as_u64())
        .unwrap_or(0);
    assert_eq!(
        count, 2,
        "one completed upload must be reusable for both workspaces"
    );
}

#[tokio::test]
async fn candidate_profile_http_routes_persist_valid_data_and_reject_invalid_updates() {
    let (app, _, db) = setup_test_app().await;
    let owner =
        setup_application_user_and_resume(&app, &db, "Dorothy Vaughan", "profile_owner").await;
    let other = setup_application_user_and_resume(&app, &db, "Mary Jackson", "profile_other").await;

    let (initial_status, initial) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/candidate-profile")
            .header("authorization", format!("Bearer {}", owner.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(initial_status, StatusCode::OK);
    assert_eq!(initial["required_keywords"], serde_json::json!([]));
    assert_eq!(initial["excluded_keywords"], serde_json::json!([]));

    let payload = serde_json::json!({
        "phone": "+1 (555) 123-4567",
        "location": "Washington, DC",
        "linkedin_url": "https://linkedin.com/in/dorothy-vaughan",
        "github_url": "https://github.com/dvaughan",
        "portfolio_url": "https://dorothy.example",
        "target_role": "Platform Engineer",
        "required_keywords": ["Rust", "Distributed Systems"],
        "excluded_keywords": ["COBOL"],
        "target_locations": ["Remote", "Washington, DC"]
    });
    let (update_status, updated) = send_app_request(
        &app,
        Request::builder()
            .method("PUT")
            .uri("/api/v1/candidate-profile")
            .header("authorization", format!("Bearer {}", owner.token))
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(updated["phone"], payload["phone"]);
    assert_eq!(updated["target_role"], "Platform Engineer");
    assert_eq!(updated["required_keywords"], payload["required_keywords"]);
    assert_eq!(updated["excluded_keywords"], payload["excluded_keywords"]);

    let invalid = serde_json::json!({
        "phone": "phone1234567",
        "location": "Washington, DC",
        "linkedin_url": "http://linkedin.com/in/dorothy-vaughan",
        "github_url": "https://github.com/dvaughan",
        "portfolio_url": "https://dorothy.example",
        "target_role": "Platform Engineer",
        "required_keywords": ["Rust"],
        "excluded_keywords": ["rust"],
        "target_locations": ["Remote"]
    });
    let (invalid_status, invalid_body) = send_app_request(
        &app,
        Request::builder()
            .method("PUT")
            .uri("/api/v1/candidate-profile")
            .header("authorization", format!("Bearer {}", owner.token))
            .header("content-type", "application/json")
            .body(Body::from(invalid.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::BAD_REQUEST);
    assert_error_dto_shape(&invalid_body, StatusCode::BAD_REQUEST);

    let (persisted_status, persisted) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/candidate-profile")
            .header("authorization", format!("Bearer {}", owner.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(persisted_status, StatusCode::OK);
    assert_eq!(persisted["phone"], payload["phone"]);
    assert_eq!(persisted["required_keywords"], payload["required_keywords"]);

    let (other_status, other_profile) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/candidate-profile")
            .header("authorization", format!("Bearer {}", other.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(other_status, StatusCode::OK);
    assert_eq!(other_profile["required_keywords"], serde_json::json!([]));
    assert_eq!(other_profile["phone"], serde_json::Value::Null);
}
#[tokio::test]
async fn test_direct_application_uses_saved_job_email_before_web() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Jamie Candidate", "saved_email").await;
    let saved_job_id = uuid::Uuid::new_v4().to_string();
    let canonical_url = "https://linkedin.com/jobs/view/987654".to_string();
    db.client
        .query("CREATE type::record('saved_job', $id) SET user_id = type::record('user', $uid), source_url = $source_url, canonical_url = $canonical_url, source_site = 'linkedin', status = 'awaiting_approval', contact_email = 'hr@example.com', created_at = time::now(), updated_at = time::now();")
        .bind(("id", saved_job_id))
        .bind(("uid", user.user_id.clone()))
        .bind(("source_url", canonical_url.clone()))
        .bind(("canonical_url", canonical_url.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let base_payload = serde_json::json!({
        "site": "linkedin",
        "source_url": canonical_url,
        "company_name": "Example Company",
        "role_title": "Software Engineer",
        "job_description": "Build reliable systems.",
        "resume_upload_id": user.upload_id
    });
    let mut web_payload = base_payload.clone();
    web_payload["application_method"] = serde_json::json!("web");
    let (web_status, web_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(web_payload.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(web_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&web_body, StatusCode::CONFLICT);

    let mut mismatched_email_payload = base_payload.clone();
    mismatched_email_payload["application_method"] = serde_json::json!("email");
    mismatched_email_payload["contact_email"] = serde_json::json!("other@example.com");
    let (mismatch_status, mismatch_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(mismatched_email_payload.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(mismatch_status, StatusCode::CONFLICT);
    assert_error_dto_shape(&mismatch_body, StatusCode::CONFLICT);

    let mut email_payload = base_payload;
    email_payload["application_method"] = serde_json::json!("email");
    let (email_status, email_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/applications")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(email_payload.to_string()))
            .unwrap(),
    )
    .await;
    assert_eq!(email_status, StatusCode::CREATED);
    assert_eq!(email_body["application_method"], "email");
    assert_eq!(email_body["contact_email"], "hr@example.com");
}

#[tokio::test]
async fn test_dispatch_migration_backfills_only_existing_unrun_submissions() {
    let (app, config, db) = setup_test_app().await;
    let user =
        setup_application_user_and_resume(&app, &db, "Migration Candidate", "migration_dispatch")
            .await;
    let mut migration_history = db
        .client
        .query(
            "SELECT VALUE version FROM schema_migration WHERE version IN ['015_saved_jobs_application_flow', '016_application_dispatch_and_saved_job_index'];",
        )
        .await
        .unwrap();
    let applied_versions: Vec<String> = migration_history.take(0).unwrap();
    assert!(applied_versions.contains(&"015_saved_jobs_application_flow".to_string()));
    assert!(applied_versions.contains(&"016_application_dispatch_and_saved_job_index".to_string()));
    let workspace_id = uuid::Uuid::new_v4().to_string();
    let application_id = uuid::Uuid::new_v4().to_string();
    let missing_workspace_id = uuid::Uuid::new_v4().to_string();
    let orphan_application_id = uuid::Uuid::new_v4().to_string();
    let existing_run_workspace_id = uuid::Uuid::new_v4().to_string();
    let existing_run_application_id = uuid::Uuid::new_v4().to_string();
    let existing_run_id = uuid::Uuid::new_v4().to_string();

    db.client
        .query(
            "CREATE type::record('workspace', $workspace_id) SET \
                user_id = type::record('user', $user_id), \
                company_name = 'Migration Fixture', role_title = 'Platform Engineer', \
                status = 'failed', current_phase = 'failed', progress_percent = 100; \
             CREATE type::record('job_application', $application_id) SET \
                user_id = type::record('user', $user_id), \
                resume_upload_id = type::record('resume_upload', $upload_id), \
                workspace_id = type::record('workspace', $workspace_id), \
                site = 'greenhouse', canonical_url = 'https://boards.greenhouse.io/example/jobs/1', \
                source_url = 'https://boards.greenhouse.io/example/jobs/1', \
                company_name = 'Migration Fixture', role_title = 'Platform Engineer', \
                job_description = 'Build reliable platform services.', \
                source_title = 'Platform Engineer', retrieved_at = time::now(), \
                revision = 'migration-revision', status = 'submitted', \
                prepared_fields = { first_name: 'Migration', last_name: 'Candidate', email: 'migration@example.com' }; \
             CREATE type::record('job_application', $orphan_application_id) SET \
                user_id = type::record('user', $user_id), \
                resume_upload_id = type::record('resume_upload', $upload_id), \
                workspace_id = type::record('workspace', $missing_workspace_id), \
                site = 'greenhouse', canonical_url = 'https://boards.greenhouse.io/example/jobs/2', \
                source_url = 'https://boards.greenhouse.io/example/jobs/2', \
                company_name = 'Missing Workspace', role_title = 'Platform Engineer', \
                job_description = 'Build reliable platform services.', \
                source_title = 'Platform Engineer', retrieved_at = time::now(), \
                revision = 'orphan-revision', status = 'submitted', \
                prepared_fields = { first_name: 'Migration', last_name: 'Candidate', email: 'migration@example.com' }; \
             CREATE type::record('workspace', $existing_run_workspace_id) SET \
                user_id = type::record('user', $user_id), \
                company_name = 'Existing Run Fixture', role_title = 'Platform Engineer', \
                status = 'failed', current_phase = 'failed', progress_percent = 100; \
             CREATE type::record('job_application', $existing_run_application_id) SET \
                user_id = type::record('user', $user_id), \
                resume_upload_id = type::record('resume_upload', $upload_id), \
                workspace_id = type::record('workspace', $existing_run_workspace_id), \
                site = 'greenhouse', canonical_url = 'https://boards.greenhouse.io/example/jobs/3', \
                source_url = 'https://boards.greenhouse.io/example/jobs/3', \
                company_name = 'Existing Run Fixture', role_title = 'Platform Engineer', \
                job_description = 'Build reliable platform services.', \
                source_title = 'Platform Engineer', retrieved_at = time::now(), \
                revision = 'existing-run-revision', status = 'submitted', \
                prepared_fields = { first_name: 'Migration', last_name: 'Candidate', email: 'migration@example.com' }; \
             CREATE type::record('workflow_run', $existing_run_id) SET \
                workspace_id = type::record('workspace', $existing_run_workspace_id), \
                user_id = $user_id, workflow_id = 'interview-preparation-hybrid', \
                workflow_name = 'Hybrid', type = 'hybrid', status = 'completed', current_phase = 'complete';",
        )
        .bind(("workspace_id", workspace_id.clone()))
        .bind(("application_id", application_id.clone()))
        .bind(("missing_workspace_id", missing_workspace_id.clone()))
        .bind(("orphan_application_id", orphan_application_id.clone()))
        .bind(("existing_run_workspace_id", existing_run_workspace_id.clone()))
        .bind(("existing_run_application_id", existing_run_application_id.clone()))
        .bind(("existing_run_id", existing_run_id.clone()))
        .bind(("user_id", user.user_id.clone()))
        .bind(("upload_id", user.upload_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let migration_sql = include_str!(
        "../../../infra/migrations/016_application_dispatch_and_saved_job_index.surql"
    )
    .replace(
        "USE NS mensetsu DB production;",
        &format!("USE NS {} DB {};", config.surreal_ns, config.surreal_db),
    );
    db.client
        .query(migration_sql.clone())
        .await
        .unwrap()
        .check()
        .unwrap();

    let mut dispatch = db
        .client
        .query(
            "SELECT * FROM application_dispatch WHERE workspace_id = type::record('workspace', $workspace_id);",
        )
        .bind(("workspace_id", workspace_id.clone()))
        .await
        .unwrap();
    let dispatch_rows: Vec<serde_json::Value> = dispatch.take(0).unwrap();
    assert_eq!(dispatch_rows.len(), 1);
    assert_eq!(dispatch_rows[0]["state"], "pending");
    assert_eq!(dispatch_rows[0]["attempts"], 0);

    let mut workspace = db
        .client
        .query("SELECT * FROM type::record('workspace', $workspace_id);")
        .bind(("workspace_id", workspace_id.clone()))
        .await
        .unwrap();
    let workspace_rows: Vec<serde_json::Value> = workspace.take(0).unwrap();
    assert_eq!(workspace_rows.len(), 1);
    assert_eq!(workspace_rows[0]["status"], "pending");
    assert_eq!(workspace_rows[0]["current_phase"], "preparing_input");
    assert_eq!(workspace_rows[0]["company_name"], "Migration Fixture");
    assert_eq!(workspace_rows[0]["role_title"], "Platform Engineer");

    let mut orphan_dispatch = db
        .client
        .query(
            "SELECT * FROM application_dispatch WHERE application_id = type::record('job_application', $application_id);",
        )
        .bind(("application_id", orphan_application_id.clone()))
        .await
        .unwrap();
    let orphan_rows: Vec<serde_json::Value> = orphan_dispatch.take(0).unwrap();
    assert!(
        orphan_rows.is_empty(),
        "an orphaned workspace reference must not dispatch"
    );
    let mut dispatched_existing_run = db
        .client
        .query(
            "SELECT * FROM application_dispatch WHERE application_id = type::record('job_application', $application_id);",
        )
        .bind(("application_id", existing_run_application_id))
        .await
        .unwrap();
    let existing_run_dispatch_rows: Vec<serde_json::Value> =
        dispatched_existing_run.take(0).unwrap();
    assert!(
        existing_run_dispatch_rows.is_empty(),
        "an existing workflow run must not be dispatched again"
    );
    let mut existing_run_workspace = db
        .client
        .query("SELECT * FROM type::record('workspace', $workspace_id);")
        .bind(("workspace_id", existing_run_workspace_id))
        .await
        .unwrap();
    let existing_run_workspace_rows: Vec<serde_json::Value> =
        existing_run_workspace.take(0).unwrap();
    assert_eq!(existing_run_workspace_rows.len(), 1);
    assert_eq!(existing_run_workspace_rows[0]["status"], "failed");

    db.client
        .query(migration_sql)
        .await
        .unwrap()
        .check()
        .unwrap();
    let mut replay = db
        .client
        .query(
            "SELECT * FROM application_dispatch WHERE workspace_id = type::record('workspace', $workspace_id);",
        )
        .bind(("workspace_id", workspace_id))
        .await
        .unwrap();
    let replay_rows: Vec<serde_json::Value> = replay.take(0).unwrap();
    assert_eq!(
        replay_rows.len(),
        1,
        "backfill reruns must not duplicate dispatch"
    );
    assert_eq!(replay_rows[0]["attempts"], 0);
}

#[tokio::test]
async fn test_saved_job_keyset_pages_include_linked_application_lifecycle() {
    let (app, _, db) = setup_test_app().await;
    let user = setup_application_user_and_resume(&app, &db, "Page Candidate", "saved_page").await;
    let foreign_user =
        setup_application_user_and_resume(&app, &db, "Foreign Candidate", "saved_page_foreign")
            .await;
    let workspace_id = uuid::Uuid::new_v4().to_string();
    let application_id = uuid::Uuid::new_v4().to_string();
    let linked_job_id = uuid::Uuid::new_v4().to_string();
    let second_job_id = uuid::Uuid::new_v4().to_string();
    let third_job_id = uuid::Uuid::new_v4().to_string();
    let dismissed_job_id = uuid::Uuid::new_v4().to_string();
    let foreign_job_id = uuid::Uuid::new_v4().to_string();

    db.client
        .query(
            "CREATE type::record('workspace', $workspace_id) SET \
                user_id = type::record('user', $user_id), \
                company_name = 'Pagination Fixture', role_title = 'Platform Engineer', \
                status = 'pending', current_phase = 'preparing_input', progress_percent = 0; \
             CREATE type::record('job_application', $application_id) SET \
                user_id = type::record('user', $user_id), \
                resume_upload_id = type::record('resume_upload', $upload_id), \
                workspace_id = type::record('workspace', $workspace_id), \
                site = 'linkedin', canonical_url = 'https://linkedin.com/jobs/view/7101', \
                source_url = 'https://linkedin.com/jobs/view/7101', \
                company_name = 'Pagination Fixture', role_title = 'Platform Engineer', \
                job_description = 'Build reliable platform services.', \
                source_title = 'Platform Engineer', retrieved_at = time::now(), \
                revision = 'pagination-revision', status = 'submitted', \
                prepared_fields = { first_name: 'Page', last_name: 'Candidate', email: 'page@example.com' }; \
             CREATE type::record('saved_job', $linked_job_id) SET \
                user_id = type::record('user', $user_id), \
                source_url = 'https://linkedin.com/jobs/view/7101', \
                canonical_url = 'https://linkedin.com/jobs/view/7101', \
                source_site = 'linkedin', status = 'approved', \
                job_application_id = type::record('job_application', $application_id); \
             CREATE type::record('saved_job', $second_job_id) SET \
                user_id = type::record('user', $user_id), \
                source_url = 'https://linkedin.com/jobs/view/7102', \
                canonical_url = 'https://linkedin.com/jobs/view/7102', \
                source_site = 'linkedin', status = 'awaiting_approval'; \
             CREATE type::record('saved_job', $third_job_id) SET \
                user_id = type::record('user', $user_id), \
                source_url = 'https://linkedin.com/jobs/view/7103', \
                canonical_url = 'https://linkedin.com/jobs/view/7103', \
                source_site = 'linkedin', status = 'awaiting_approval'; \
             CREATE type::record('saved_job', $dismissed_job_id) SET \
                user_id = type::record('user', $user_id), \
                source_url = 'https://linkedin.com/jobs/view/7104', \
                canonical_url = 'https://linkedin.com/jobs/view/7104', \
                source_site = 'linkedin', status = 'dismissed'; \
             CREATE type::record('saved_job', $foreign_job_id) SET \
                user_id = type::record('user', $foreign_user_id), \
                source_url = 'https://linkedin.com/jobs/view/7105', \
                canonical_url = 'https://linkedin.com/jobs/view/7105', \
                source_site = 'linkedin', status = 'awaiting_approval';",
        )
        .bind(("workspace_id", workspace_id))
        .bind(("application_id", application_id))
        .bind(("linked_job_id", linked_job_id.clone()))
        .bind(("second_job_id", second_job_id.clone()))
        .bind(("third_job_id", third_job_id.clone()))
        .bind(("dismissed_job_id", dismissed_job_id.clone()))
        .bind(("foreign_job_id", foreign_job_id.clone()))
        .bind(("user_id", user.user_id.clone()))
        .bind(("upload_id", user.upload_id.clone()))
        .bind(("foreign_user_id", foreign_user.user_id.clone()))
        .await
        .unwrap()
        .check()
        .unwrap();

    let (first_status, first_body) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/saved-jobs?limit=2")
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(first_status, StatusCode::OK);
    let first_items = first_body["items"].as_array().unwrap();
    assert_eq!(first_items.len(), 2);
    let cursor = first_body["next_cursor"].as_str().unwrap();

    let (second_status, second_body) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/saved-jobs?limit=2&cursor={cursor}"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(second_status, StatusCode::OK);
    let second_items = second_body["items"].as_array().unwrap();
    assert_eq!(second_items.len(), 1);
    assert!(second_body["next_cursor"].is_null());

    let mut all_items = first_items.to_vec();
    all_items.extend(second_items.iter().cloned());
    let mut page_ids: Vec<String> = all_items
        .iter()
        .map(|job| job["id"].as_str().unwrap().to_string())
        .collect();
    page_ids.sort();
    page_ids.dedup();
    let mut expected_ids = vec![linked_job_id.clone(), second_job_id, third_job_id];
    expected_ids.sort();
    assert_eq!(
        page_ids, expected_ids,
        "keyset pages must be complete and disjoint"
    );
    assert!(all_items.iter().all(|job| {
        job["id"].as_str() != Some(dismissed_job_id.as_str())
            && job["id"].as_str() != Some(foreign_job_id.as_str())
    }));
    let linked_job = all_items
        .iter()
        .find(|job| job["id"].as_str() == Some(linked_job_id.as_str()))
        .unwrap();
    assert_eq!(linked_job["application_status"], "submitted");
    assert_eq!(linked_job["workspace_status"], "pending");

    let (invalid_cursor_status, _) = send_app_request(
        &app,
        Request::builder()
            .uri(format!("/api/v1/saved-jobs?cursor={foreign_job_id}"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(invalid_cursor_status, StatusCode::BAD_REQUEST);
    let (zero_limit_status, _) = send_app_request(
        &app,
        Request::builder()
            .uri("/api/v1/saved-jobs?limit=0")
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(zero_limit_status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_saved_job_prepare_and_dismiss_race_is_atomic() {
    let (app, _, db) = setup_test_app().await;
    let user =
        setup_application_user_and_resume(&app, &db, "Atomic Candidate", "saved_atomic").await;
    let (save_status, save_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/saved-jobs")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "source_url": "https://www.linkedin.com/jobs/view/81101?tracking=1",
                    "role_title": "Platform Engineer",
                    "company_name": "Atomic Systems",
                    "job_description": "Build reliable platform services.",
                    "application_url": "https://jobs.lever.co/atomic/81101"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(save_status, StatusCode::CREATED);
    let saved_job_id = save_body["id"].as_str().unwrap().to_string();

    let (approve_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/saved-jobs/{saved_job_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK);

    let prepare_request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/saved-jobs/{saved_job_id}/prepare"))
        .header("authorization", format!("Bearer {}", user.token))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({
                "resume_upload_id": user.upload_id,
                "application_method": "web"
            })
            .to_string(),
        ))
        .unwrap();
    let dismiss_request = Request::builder()
        .method("POST")
        .uri(format!("/api/v1/saved-jobs/{saved_job_id}/dismiss"))
        .header("authorization", format!("Bearer {}", user.token))
        .body(Body::empty())
        .unwrap();
    let ((prepare_status, _prepare_body), (dismiss_status, _)) = tokio::join!(
        send_app_request(&app, prepare_request),
        send_app_request(&app, dismiss_request)
    );
    assert!(
        prepare_status.is_success() || prepare_status == StatusCode::CONFLICT,
        "prepare must either commit or observe the competing dismissal"
    );
    assert!(
        dismiss_status.is_success() || dismiss_status == StatusCode::CONFLICT,
        "dismiss must either commit or observe the competing preparation"
    );
    assert!(
        prepare_status.is_success() || dismiss_status.is_success(),
        "one transition must win"
    );

    let mut saved = db
        .client
        .query(
            "SELECT status, job_application_id, canonical_url FROM type::record('saved_job', $id) WHERE user_id = type::record('user', $uid);",
        )
        .bind(("id", saved_job_id.clone()))
        .bind(("uid", user.user_id.clone()))
        .await
        .unwrap();
    let saved_rows: Vec<serde_json::Value> = saved.take(0).unwrap();
    assert_eq!(saved_rows.len(), 1);
    let mut applications = db
        .client
        .query(
            "SELECT id, status FROM job_application WHERE user_id = type::record('user', $uid) AND canonical_url = $canonical_url;",
        )
        .bind(("uid", user.user_id.clone()))
        .bind(("canonical_url", saved_rows[0]["canonical_url"].as_str().unwrap().to_string()))
        .await
        .unwrap();
    let application_rows: Vec<serde_json::Value> = applications.take(0).unwrap();
    match saved_rows[0]["status"].as_str() {
        Some("dismissed") => {
            assert!(saved_rows[0]["job_application_id"].is_null());
            assert!(
                application_rows.is_empty(),
                "dismissal must not leave an unlinked prepared application"
            );
        }
        Some("approved") => {
            let application_id = saved_rows[0]["job_application_id"].as_str().unwrap();
            let linked_id = application_id.rsplit(':').next().unwrap();
            assert_eq!(application_rows[0]["status"], "approved");
            assert_eq!(
                application_rows[0]["id"]
                    .as_str()
                    .unwrap()
                    .rsplit(':')
                    .next(),
                Some(linked_id),
                "the saved job must link to the single prepared application"
            );

            let (retry_status, retry_body) = send_app_request(
                &app,
                Request::builder()
                    .method("POST")
                    .uri(format!("/api/v1/saved-jobs/{saved_job_id}/prepare"))
                    .header("authorization", format!("Bearer {}", user.token))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        serde_json::json!({
                            "resume_upload_id": user.upload_id,
                            "application_method": "web"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await;
            assert_eq!(retry_status, StatusCode::OK);
            assert_eq!(
                retry_body["id"].as_str().unwrap().rsplit(':').next(),
                Some(linked_id)
            );
        }
        status => panic!("unexpected saved-job state after competing transitions: {status:?}"),
    }
}

#[tokio::test]
async fn test_saved_job_material_update_requires_review_and_reenrichment() {
    let (app, _, db) = setup_test_app().await;
    let user =
        setup_application_user_and_resume(&app, &db, "Atomic Candidate", "saved_atomic").await;
    let source_url = "https://www.linkedin.com/jobs/view/81101?tracking=1";
    let (save_status, save_body) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/saved-jobs")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "source_url": source_url,
                    "role_title": "Platform Engineer",
                    "company_name": "Atomic Systems",
                    "job_description": "Build reliable platform services.",
                    "application_url": "https://jobs.lever.co/atomic/81101"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(save_status, StatusCode::CREATED);
    let saved_job_id = save_body["id"].as_str().unwrap().to_string();

    let (approve_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/saved-jobs/{saved_job_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(approve_status, StatusCode::OK);

    let mut saved_before_prepare = db
        .client
        .query("SELECT status, job_application_id FROM type::record('saved_job',$id);")
        .bind(("id", saved_job_id.clone()))
        .await
        .unwrap();
    let saved_before_prepare_rows: Vec<serde_json::Value> = saved_before_prepare.take(0).unwrap();
    assert_eq!(saved_before_prepare_rows.len(), 1);
    assert_eq!(saved_before_prepare_rows[0]["status"], "approved");
    assert!(saved_before_prepare_rows[0]["job_application_id"].is_null());

    let prepare_request = || {
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/saved-jobs/{saved_job_id}/prepare"))
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "resume_upload_id": user.upload_id,
                    "application_method": "web"
                })
                .to_string(),
            ))
            .unwrap()
    };
    let (prepare_status, prepared) = send_app_request(&app, prepare_request()).await;
    assert_eq!(
        prepare_status,
        StatusCode::OK,
        "prepare response: {prepared}"
    );
    assert_eq!(prepared["status"], "approved");
    let application_id = prepared["id"].as_str().unwrap().to_string();
    let initial_revision = prepared["revision"].as_str().unwrap().to_string();

    let (update_status, updated_saved_job) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri("/api/v1/saved-jobs")
            .header("authorization", format!("Bearer {}", user.token))
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({
                    "source_url": source_url,
                    "role_title": "Senior Platform Engineer",
                    "company_name": "New Systems",
                    "job_description": "Design and operate resilient platform services.",
                    "application_url": "https://jobs.lever.co/reenrich/81102"
                })
                .to_string(),
            ))
            .unwrap(),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(updated_saved_job["id"], saved_job_id);
    assert_eq!(updated_saved_job["job_application_id"], application_id);
    assert_eq!(updated_saved_job["status"], "awaiting_approval");

    let (reapprove_status, _) = send_app_request(
        &app,
        Request::builder()
            .method("POST")
            .uri(format!("/api/v1/saved-jobs/{saved_job_id}/approve"))
            .header("authorization", format!("Bearer {}", user.token))
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(reapprove_status, StatusCode::OK);

    let mut reviewed = db
        .client
        .query(
            "SELECT status, role_title, company_name, job_description, revision FROM type::record('job_application', $id);",
        )
        .bind(("id", application_id.clone()))
        .await
        .unwrap();
    let reviewed_rows: Vec<serde_json::Value> = reviewed.take(0).unwrap();
    assert_eq!(reviewed_rows.len(), 1);
    assert_eq!(reviewed_rows[0]["status"], "needs_review");
    assert_eq!(reviewed_rows[0]["role_title"], "Platform Engineer");
    assert_eq!(reviewed_rows[0]["revision"], initial_revision);

    let (reenrich_status, reenriched) = send_app_request(&app, prepare_request()).await;
    assert_eq!(reenrich_status, StatusCode::OK);
    assert_eq!(reenriched["id"], application_id);
    assert_eq!(reenriched["status"], "approved");
    assert_eq!(reenriched["role_title"], "Senior Platform Engineer");
    assert_eq!(reenriched["company_name"], "New Systems");
    assert_eq!(
        reenriched["job_description"],
        "Design and operate resilient platform services."
    );
    assert_ne!(reenriched["revision"], initial_revision);
}
