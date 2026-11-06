
use mensetsu_desktop::services::api::ApiService;
use mensetsu_desktop::services::endpoint_config::{set_persisted_server_url, validate_and_normalize_url, EndpointConfigError};
use mensetsu_contracts::models::UpsertLlmConfigRequestDto;

#[test]
fn test_endpoint_validation_and_normalization() {
    assert_eq!(
        validate_and_normalize_url("http://127.0.0.1:3000").unwrap(),
        "http://127.0.0.1:3000"
    );
    assert_eq!(
        validate_and_normalize_url("http://localhost:3000/").unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        validate_and_normalize_url("https://api.mensetsu.example.com").unwrap(),
        "https://api.mensetsu.example.com"
    );


    match validate_and_normalize_url("http://api.mensetsu.example.com") {
        Err(EndpointConfigError::InsecureHttp) => (),
        other => panic!("Expected InsecureHttp error, got {:?}", other),
    }


    match validate_and_normalize_url("https://api.mensetsu.example.com/api/v1") {
        Err(EndpointConfigError::PathNotAllowed) => (),
        other => panic!("Expected PathNotAllowed error, got {:?}", other),
    }


    match validate_and_normalize_url("https://user:pass@api.mensetsu.example.com") {
        Err(EndpointConfigError::UserinfoNotAllowed) => (),
        other => panic!("Expected UserinfoNotAllowed error, got {:?}", other),
    }
}

#[tokio::test]
async fn test_api_integration_smoke() {
    let api_url = std::env::var("MENSETSU_API_URL").unwrap_or_default();
    if api_url.is_empty() {
        eprintln!("MENSETSU_API_URL not set; skipping live server integration checks.");
        return;
    }

    let temp_dir = tempfile::tempdir().unwrap();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    set_persisted_server_url(&api_url).unwrap();

    let api = ApiService::new(api_url);

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let email = format!("dioxus_smoke_{unique}@example.com");
    let name = "Dioxus Smoke Tester";
    let password = "Password123!Secure";


    api.register(name, &email, password).await.expect("Registration should succeed");


    let workspaces = api.list_workspaces().await.expect("List workspaces should succeed");
    let initial_count = workspaces.len();


    let ws = api.create_workspace(
        "Dioxus Corp",
        "Staff Rust Engineer",
        "Built high-performance cross-platform applications in Rust with Dioxus.",
        "Looking for an expert Rust engineer with cross-platform UI experience.",
    ).await.expect("Workspace creation should succeed");

    assert_eq!(ws.company_name, "Dioxus Corp");
    assert_eq!(ws.role_title, "Staff Rust Engineer");


    let updated = api.list_workspaces().await.expect("List workspaces after create");
    assert_eq!(updated.len(), initial_count + 1);


    let fetched = api.get_workspace(&ws.id).await.expect("Get workspace");
    assert_eq!(fetched.id, ws.id);


    let mut llm_req = UpsertLlmConfigRequestDto::new("openai".to_string());
    llm_req.api_key = Some(Some("sk-test-fake-key-for-smoke".to_string()));
    let _ = api.update_llm_setting(llm_req).await;
    let settings = api.get_llm_settings().await.unwrap_or_default();
    assert!(settings.iter().any(|s| s.provider == "openai"));


    api.logout().await.expect("Logout should succeed");
}
