//! Real GTK4 Accessibility-Driven UI Automation & API Integration Tests
//!
//! Provides:
//! 1. `gtk_ui_automation_smoke_test`: Actually launches GTK application window and views,
//!    drives the GLib main loop, fills inputs for register -> login -> create workspace -> refresh list -> logout,
//!    and asserts workspace and view state transitions directly on the live UI tree with real Rust API.
//! 2. `api_integration_smoke_test`: Low-level headless REST client integration suite verifying
//!    detailed contract invariants (match, company, learning unit completion toggle, LLM settings, dead port network error).

use mensetsu_linux::api_service::ApiService;
use mensetsu_linux::app_root::AppRoot;
use mensetsu_linux::test_ids::ids;
use mensetsu_contracts::models::UpsertLlmConfigRequestDto;
use gtk4::prelude::*;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static UI_TEST_MUTEX: Mutex<()> = Mutex::new(());

/// Helper to recursively search for a widget by widget_name (accessibility ID)
fn find_widget_by_name(parent: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
    if parent.widget_name() == name {
        return Some(parent.clone());
    }
    let mut child = parent.first_child();
    while let Some(c) = child {
        if let Some(found) = find_widget_by_name(&c, name) {
            return Some(found);
        }
        child = c.next_sibling();
    }
    None
}

/// Pump the GLib main context until condition is met or timeout expires
fn wait_for<F>(context: &glib::MainContext, timeout: Duration, condition: F) -> bool
where
    F: Fn() -> bool,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        while context.iteration(false) {}
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

#[test]
fn gtk_ui_automation_smoke_test() {
    let _ui_lock = UI_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Tokio runtime required for native UI requests");
    let _runtime_guard = runtime.enter();
    gtk4::init().expect("GTK4 display required for UI smoke");

    let main_context = glib::MainContext::default();
    let _guard = main_context.acquire();
    // The test runner reads the target API URL from test environment fixture,
    // but configures the app endpoint exclusively via persisted configuration fixture
    // rather than runtime env injection.
    let fixture_url = std::env::var("MENSETSU_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let temp_dir = tempfile::tempdir().unwrap();
    let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    mensetsu_linux::endpoint_config::set_persisted_server_url(&fixture_url).unwrap();

    let base_url = mensetsu_linux::endpoint_config::resolve_initial_url().expect("Persisted fixture must resolve");
    let api = ApiService::new(base_url);

    // Build the full root app
    let app_root = AppRoot::new(api);
    app_root.window.present();

    let stack = app_root.window.child().unwrap().downcast::<gtk4::Stack>().unwrap();
    let root_widget = app_root.window.upcast::<gtk4::Widget>();

    // Pump loop to process initial auth checks
    let _ = wait_for(&main_context, Duration::from_millis(500), || stack.visible_child_name() == Some("auth".into()));
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let email = format!("gtk_ui_{unique}@example.com");
    let name = "GTK UI Smoke Tester";
    let password = "Password123!Secure";

    let mode_switch = find_widget_by_name(&root_widget, ids::AUTH_MODE_SWITCH)
        .expect("AUTH_MODE_SWITCH must exist")
        .downcast::<gtk4::Button>()
        .unwrap();

    // Switch to Register mode
    mode_switch.emit_clicked();
    while main_context.iteration(false) {}

    let name_input = find_widget_by_name(&root_widget, ids::AUTH_NAME_INPUT)
        .expect("AUTH_NAME_INPUT must exist in register mode")
        .downcast::<gtk4::Entry>()
        .unwrap();
    name_input.set_text(name);

    let email_input = find_widget_by_name(&root_widget, ids::AUTH_EMAIL_INPUT)
        .expect("AUTH_EMAIL_INPUT must exist")
        .downcast::<gtk4::Entry>()
        .unwrap();
    email_input.set_text(&email);

    let password_input = find_widget_by_name(&root_widget, ids::AUTH_PASSWORD_INPUT)
        .expect("AUTH_PASSWORD_INPUT must exist")
        .downcast::<gtk4::PasswordEntry>()
        .unwrap();
    password_input.set_text(password);

    let submit_btn = find_widget_by_name(&root_widget, ids::AUTH_SUBMIT_BUTTON)
        .expect("AUTH_SUBMIT_BUTTON must exist")
        .downcast::<gtk4::Button>()
        .unwrap();

    // Submit registration
    submit_btn.emit_clicked();

    // Wait until transition to "main" view
    let in_main_view = wait_for(&main_context, Duration::from_secs(5), || {
        stack.visible_child_name() == Some("main".into())
    });
    assert!(in_main_view, "UI should transition to main workspace view after registration");

    // 2. Perform Logout
    let logout_btn = find_widget_by_name(&root_widget, ids::LOGOUT_BUTTON)
        .expect("LOGOUT_BUTTON must exist")
        .downcast::<gtk4::Button>()
        .unwrap();
    logout_btn.emit_clicked();

    let in_auth_view = wait_for(&main_context, Duration::from_secs(3), || {
        stack.visible_child_name() == Some("auth".into())
    });
    assert!(in_auth_view, "UI should transition back to auth view after logout");

    // 3. Perform Login with the registered user
    // Toggle back to login mode
    mode_switch.emit_clicked();
    while main_context.iteration(false) {}

    email_input.set_text(&email);
    password_input.set_text(password);
    submit_btn.emit_clicked();

    let in_main_after_login = wait_for(&main_context, Duration::from_secs(5), || {
        stack.visible_child_name() == Some("main".into())
    });
    assert!(in_main_after_login, "UI should transition to main workspace view after login");

    // 4. Open New Workspace Dialog and create workspace with text
    let new_ws_btn = find_widget_by_name(&root_widget, ids::NEW_WORKSPACE_BUTTON)
        .expect("NEW_WORKSPACE_BUTTON must exist")
        .downcast::<gtk4::Button>()
        .unwrap();
    new_ws_btn.emit_clicked();

    let dialog = gtk4::Window::list_toplevels()
        .into_iter()
        .find_map(|widget| widget.downcast::<gtk4::Dialog>().ok())
        .expect("Create dialog must appear after clicking New Workspace");
    let dialog_widget = dialog.upcast_ref::<gtk4::Widget>();
    let company_name = "GTK Systems Inc";
    let role_title = "Senior Linux Systems Engineer";
    find_widget_by_name(dialog_widget, ids::CREATE_COMPANY_INPUT)
        .expect("Company input must appear in create dialog")
        .downcast::<gtk4::Entry>().unwrap().set_text(company_name);
    find_widget_by_name(dialog_widget, ids::CREATE_ROLE_INPUT)
        .expect("Role input must appear in create dialog")
        .downcast::<gtk4::Entry>().unwrap().set_text(role_title);
    find_widget_by_name(dialog_widget, ids::CREATE_RESUME_INPUT)
        .expect("Resume input must appear in create dialog")
        .downcast::<gtk4::TextView>().unwrap().buffer()
        .set_text("Proficient in Rust, GTK4, Linux internals, multithreaded systems.");
    find_widget_by_name(dialog_widget, ids::CREATE_JD_INPUT)
        .expect("Job description input must appear in create dialog")
        .downcast::<gtk4::TextView>().unwrap().buffer()
        .set_text("Seeking Senior Linux Engineer for high performance desktop application.");
    find_widget_by_name(dialog_widget, ids::CREATE_SUBMIT_BUTTON)
        .expect("Create action must appear in dialog")
        .downcast::<gtk4::Button>().unwrap().emit_clicked();

    let list_container = find_widget_by_name(&root_widget, ids::WORKSPACE_LIST)
        .expect("WORKSPACE_LIST must exist");
    let created_row = wait_for(&main_context, Duration::from_secs(8), || {
        let mut child = list_container.first_child();
        while let Some(row) = child {
            if let Some(content) = row.first_child() {
                if let Some(label) = content.first_child().and_downcast::<gtk4::Label>() {
                    if label.text().contains("GTK Systems Inc — Senior Linux Systems Engineer") {
                        return true;
                    }
                }
            }
            child = row.next_sibling();
        }
        false
    });
    assert!(created_row, "Newly created company and role must appear in workspace list");

    list_container.first_child().expect("Created workspace row must exist")
        .downcast::<gtk4::ListBoxRow>().unwrap().emit_activate();
    assert!(wait_for(&main_context, Duration::from_secs(3), || {
        stack.visible_child_name() == Some("detail".into())
    }), "Selecting the workspace must open its real detail view");
    for tab in [
        ids::DETAIL_TAB_OVERVIEW, ids::DETAIL_TAB_MATCH, ids::DETAIL_TAB_COMPANY,
        ids::DETAIL_TAB_LEARNING, ids::DETAIL_TAB_QUESTIONS,
    ] {
        assert!(find_widget_by_name(&root_widget, tab).is_some(), "Missing detail tab: {tab}");
    }
    find_widget_by_name(&root_widget, ids::DETAIL_BACK_BUTTON)
        .expect("Detail back action required")
        .downcast::<gtk4::Button>().unwrap().emit_clicked();
    // Exercise the GTK PDF chooser and the real signed PUT -> complete -> workspace path.
    new_ws_btn.emit_clicked();
    let pdf_dialog = gtk4::Window::list_toplevels().into_iter()
        .filter(|window| window.is_visible())
        .find_map(|window| window.downcast::<gtk4::Dialog>().ok())
        .expect("PDF workspace creation dialog must be visible");
    let pdf_dialog_widget = pdf_dialog.upcast_ref::<gtk4::Widget>();
    find_widget_by_name(pdf_dialog_widget, ids::CREATE_COMPANY_INPUT)
        .unwrap().downcast::<gtk4::Entry>().unwrap().set_text("GTK PDF Systems");
    find_widget_by_name(pdf_dialog_widget, ids::CREATE_ROLE_INPUT)
        .unwrap().downcast::<gtk4::Entry>().unwrap().set_text("Backend Engineer");
    find_widget_by_name(pdf_dialog_widget, ids::CREATE_JD_INPUT)
        .unwrap().downcast::<gtk4::TextView>().unwrap().buffer()
        .set_text("Backend Rust engineer");
    find_widget_by_name(pdf_dialog_widget, ids::CREATE_PDF_BUTTON)
        .expect("Native PDF chooser action required")
        .downcast::<gtk4::Button>().unwrap().emit_clicked();
    let chooser = gtk4::Window::list_toplevels().into_iter()
        .filter(|window| window.is_visible())
        .find_map(|window| window.downcast::<gtk4::FileChooserDialog>().ok())
        .expect("GTK PDF file chooser must open");
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../fixtures/resume-text.pdf")
        .canonicalize().expect("PDF fixture must exist");
    chooser.set_file(&gio::File::for_path(&fixture))
        .expect("PDF fixture should be selectable");
    assert!(wait_for(&main_context, Duration::from_secs(5), || {
        chooser.file().and_then(|file| file.path()).as_deref() == Some(fixture.as_path())
    }), "GTK file chooser must resolve the selected fixture before confirmation");
    chooser.response(gtk4::ResponseType::Accept);
    assert!(wait_for(&main_context, Duration::from_secs(3), || {
        find_widget_by_name(pdf_dialog_widget, ids::CREATE_PDF_LABEL)
            .and_then(|widget| widget.downcast::<gtk4::Label>().ok())
            .is_some_and(|label| label.text().contains("resume-text.pdf"))
    }), "File chooser must retain the selected PDF");
    find_widget_by_name(pdf_dialog_widget, ids::CREATE_SUBMIT_BUTTON)
        .unwrap().downcast::<gtk4::Button>().unwrap().emit_clicked();
    assert!(wait_for(&main_context, Duration::from_secs(20), || {
        let mut child = list_container.first_child();
        while let Some(row) = child {
            if let Some(label) = row.first_child()
                .and_then(|content| content.first_child())
                .and_then(|child| child.downcast::<gtk4::Label>().ok())
            {
                if label.text().contains("GTK PDF Systems — Backend Engineer") {
                    return true;
                }
            }
            child = row.next_sibling();
        }
        false
    }), "PDF workspace must appear in live API workspace list");
    find_widget_by_name(&root_widget, ids::LOGOUT_BUTTON)
        .expect("Logout action required")
        .downcast::<gtk4::Button>().unwrap().emit_clicked();
    assert!(wait_for(&main_context, Duration::from_secs(3), || {
        stack.visible_child_name() == Some("auth".into())
    }), "Logout must hide protected workspace details");

    let offline = AppRoot::new(ApiService::new("http://127.0.0.1:59999".to_string()));
    offline.window.present();
    let offline_root = offline.window.upcast_ref::<gtk4::Widget>();
    find_widget_by_name(offline_root, ids::AUTH_MODE_SWITCH)
        .unwrap().downcast::<gtk4::Button>().unwrap().emit_clicked();
    find_widget_by_name(offline_root, ids::AUTH_NAME_INPUT)
        .unwrap().downcast::<gtk4::Entry>().unwrap().set_text("Offline Tester");
    find_widget_by_name(offline_root, ids::AUTH_EMAIL_INPUT)
        .unwrap().downcast::<gtk4::Entry>().unwrap().set_text(&format!("offline_{unique}@example.com"));
    find_widget_by_name(offline_root, ids::AUTH_PASSWORD_INPUT)
        .unwrap().downcast::<gtk4::PasswordEntry>().unwrap().set_text(password);
    find_widget_by_name(offline_root, ids::AUTH_SUBMIT_BUTTON)
        .unwrap().downcast::<gtk4::Button>().unwrap().emit_clicked();
    let error = find_widget_by_name(offline_root, ids::AUTH_ERROR_LABEL)
        .unwrap().downcast::<gtk4::Label>().unwrap();
    assert!(wait_for(&main_context, Duration::from_secs(3), || error.is_visible()),
        "API outage must show authentication error instead of sample content");
    let offline_stack = offline.window.child().unwrap().downcast::<gtk4::Stack>().unwrap();
    assert_eq!(offline_stack.visible_child_name(), Some("auth".into()));

    if let Some(val) = prev_xdg {
        std::env::set_var("XDG_CONFIG_HOME", val);
    } else {
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}

#[tokio::test]
async fn api_integration_smoke_test() {
    let base_url = std::env::var("MENSETSU_API_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());
    let api = ApiService::new(base_url.clone());

    // 1. Fresh user registration
    let unique = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
    let email = format!("linux_native_{unique}@example.com");
    let name = "Linux Smoke Test User";
    let password = "Password123!Secure";

    api.register(name, &email, password)
        .await
        .expect("Registration should succeed against running Rust API");

    assert!(api.is_logged_in().await, "User should be logged in after registration");

    // 2. Logout
    api.logout().await.expect("Logout should succeed");
    assert!(!api.is_logged_in().await, "User should not be logged in after logout");

    // 3. Login
    api.login(&email, password).await.expect("Login should succeed");
    assert!(api.is_logged_in().await, "User should be logged in after login");

    // 4. List Workspaces
    let _ = api.list_workspaces().await.expect("Listing workspaces should succeed");

    // 5. Create Workspace with company, role, resume_text, and job_description
    let company = "LinuxScale Systems";
    let role = "Principal Systems Engineer";
    let resume = "Senior Linux Systems Engineer with 10 years experience in Rust, GTK, kernel internals.";
    let jd = "Looking for Principal Systems Engineer proficient in Rust, Linux low-level systems, high concurrency.";

    let new_ws = api.create_workspace(company, role, resume, jd)
        .await
        .expect("Creating workspace should succeed with real text");

    assert_eq!(new_ws.company_name, company);
    assert_eq!(new_ws.role_title, role);

    // 6. Verify newly created workspace appears in workspace list
    let list_after = api.list_workspaces().await.expect("Listing workspaces after create should succeed");
    assert!(list_after.iter().any(|w| w.id == new_ws.id), "New workspace ID must be present in workspace list");

    // 7. Workspace detail checks: Match, Company, Learning, Questions
    let match_res = api.get_workspace_match(&new_ws.id).await;
    assert!(match_res.is_ok(), "Match response should return Ok: {:?}", match_res.err());

    let comp_res = api.get_workspace_company(&new_ws.id).await;
    assert_eq!(comp_res.expect("Company response should load")["company_name"], company);

    let learning_res = api.get_workspace_learning(&new_ws.id).await;
    assert!(learning_res.is_ok(), "Learning response should return Ok: {:?}", learning_res.err());
    let lessons = learning_res.unwrap();
    if let Some(first_lesson) = lessons.first() {
        let prev_state = first_lesson.completed.unwrap_or(false);
        api.toggle_learning_unit(&new_ws.id, &first_lesson.id)
            .await
            .expect("Toggling learning unit should succeed");

        // Verify updated state
        let updated_lessons = api.get_workspace_learning(&new_ws.id).await.expect("Reloading lessons should succeed");
        let updated = updated_lessons.iter().find(|l| l.id == first_lesson.id).expect("Lesson must exist");
        assert_eq!(updated.completed.unwrap_or(false), !prev_state, "Completed status must toggle");
    }

    let questions_res = api.get_workspace_questions(&new_ws.id).await;
    assert!(questions_res.is_ok(), "Questions response should return Ok: {:?}", questions_res.err());

    // 8. Settings LLM provider key update using a valid provider (e.g. openai or custom)
    let mut req = UpsertLlmConfigRequestDto::default();
    req.provider = "openai".to_string();
    req.api_key = Some(Some("sk-test-key-linux-1234".to_string()));
    let update_res = api.update_llm_setting(req).await;
    assert!(update_res.is_ok(), "Updating valid LLM provider setting should succeed: {:?}", update_res.err());

    let settings = api.get_llm_settings().await.expect("Getting LLM settings should succeed");
    let openai_setting = settings.iter().find(|s| s.provider.eq_ignore_ascii_case("openai"));
    assert!(openai_setting.is_some(), "Configured 'openai' provider must be in settings list");
    assert!(openai_setting.unwrap().has_key, "Configured provider must have has_key = true");

    // 9. Network error verification
    let bad_api = ApiService::new("http://127.0.0.1:9999".to_string()); // Non-existent port
    let err = bad_api.login("foo@example.com", "bar").await;
    assert!(err.is_err(), "Network failure must return error rather than mock success");

    api.logout().await.expect("Final logout should revoke credentials");
}

#[test]
fn endpoint_validation_and_normalization_test() {
    use mensetsu_linux::endpoint_config::{validate_and_normalize_url, EndpointConfigError};

    // Empty
    assert!(matches!(validate_and_normalize_url(""), Err(EndpointConfigError::EmptyUrl)));
    assert!(matches!(validate_and_normalize_url("   "), Err(EndpointConfigError::EmptyUrl)));

    // Insecure HTTP on remote host
    assert!(matches!(
        validate_and_normalize_url("http://remote.api.mensetsu.com"),
        Err(EndpointConfigError::InsecureHttp)
    ));

    // 0.0.0.0 rejected for HTTP
    assert!(matches!(
        validate_and_normalize_url("http://0.0.0.0:3000"),
        Err(EndpointConfigError::InsecureHttp)
    ));

    // Userinfo, query, fragment, subpath rejected
    assert!(matches!(
        validate_and_normalize_url("https://user:pass@api.mensetsu.com"),
        Err(EndpointConfigError::UserinfoNotAllowed)
    ));
    assert!(matches!(
        validate_and_normalize_url("https://api.mensetsu.com?foo=bar"),
        Err(EndpointConfigError::QueryNotAllowed)
    ));
    assert!(matches!(
        validate_and_normalize_url("https://api.mensetsu.com#hash"),
        Err(EndpointConfigError::FragmentNotAllowed)
    ));
    assert!(matches!(
        validate_and_normalize_url("https://api.mensetsu.com/api/v1"),
        Err(EndpointConfigError::PathNotAllowed)
    ));

    // Loopback allows HTTP
    assert_eq!(
        validate_and_normalize_url("http://127.0.0.1:3000/").unwrap(),
        "http://127.0.0.1:3000"
    );
    assert_eq!(
        validate_and_normalize_url("http://localhost:8080/").unwrap(),
        "http://localhost:8080"
    );

    // HTTPS normalized (trailing slashes stripped)
    assert_eq!(
        validate_and_normalize_url("https://api.mensetsu.com/").unwrap(),
        "https://api.mensetsu.com"
    );
    assert_eq!(
        validate_and_normalize_url("https://api.mensetsu.com///").unwrap(),
        "https://api.mensetsu.com"
    );
}

#[test]
fn first_launch_prompts_for_url_when_unconfigured_and_resumes_when_persisted() {
    let _ui_lock = UI_TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    // Point XDG_CONFIG_HOME to empty temp dir so no saved file is found
    let temp_dir = tempfile::tempdir().unwrap();
    let prev_xdg = std::env::var("XDG_CONFIG_HOME").ok();
    std::env::set_var("XDG_CONFIG_HOME", temp_dir.path());
    gtk4::init().expect("GTK4 display required");
    let main_context = glib::MainContext::default();
    let _guard = main_context.acquire();

    assert_eq!(
        mensetsu_linux::endpoint_config::resolve_initial_url(),
        None,
        "Unconfigured first launch must have no initial URL"
    );

    let api = ApiService::new(String::new());
    let app_root = AppRoot::new(api);
    let stack = app_root.window.child().unwrap().downcast::<gtk4::Stack>().unwrap();
    let root_widget = app_root.window.upcast::<gtk4::Widget>();

    while main_context.iteration(false) {}

    // Fresh install with no config must show server_setup_screen
    assert_eq!(
        stack.visible_child_name(),
        Some("setup".into()),
        "Unconfigured first launch must present server_setup_screen before auth"
    );
    assert!(
        find_widget_by_name(&root_widget, ids::SERVER_SETUP_SCREEN).is_some(),
        "SERVER_SETUP_SCREEN must exist on first launch"
    );
    assert!(
        find_widget_by_name(&root_widget, ids::INPUT_SERVER_URL).is_some(),
        "INPUT_SERVER_URL must exist on server setup screen"
    );

    // Persist a server URL to disk (as done via setup UI)
    mensetsu_linux::endpoint_config::set_persisted_server_url("https://api.example.org///").unwrap();
    assert_eq!(
        mensetsu_linux::endpoint_config::resolve_initial_url(),
        Some("https://api.example.org".to_string()),
        "a saved endpoint must be restored on subsequent launch"
    );
    assert_eq!(
        mensetsu_linux::endpoint_config::get_persisted_server_url().as_deref(),
        Some("https://api.example.org"),
        "persisted server URL must match normalized value"
    );

    // Restore environment
    if let Some(val) = prev_xdg {
        std::env::set_var("XDG_CONFIG_HOME", val);
    } else {
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
