#[tokio::test]
async fn test_prompt_based_workspace_and_byok_precedence() {
    // Tests that creating a workspace with prompt sets workflow_mode to hybrid and persists
    use mensetsu_api::contexts::workspace::domain::{Workspace, SeniorityLevel};
    use mensetsu_api::contexts::identity::domain::UserId;
    let uid = UserId("usr_test_123".into());
    let ws = Workspace::new(
        uid.clone(),
        "".into(),
        "".into(),
        Some(SeniorityLevel::Staff),
        None,
        Some(2),
        Some("Staff Infra at Stripe focusing on ClickHouse".into()),
        None,
    ).expect("Prompt provided without company/role should succeed");
    
    assert_eq!(ws.company_name, "Target Company");
    assert_eq!(ws.role_title, "Target Role");
    assert_eq!(ws.workflow_mode, "hybrid");
    assert_eq!(ws.user_prompt.as_deref(), Some("Staff Infra at Stripe focusing on ClickHouse"));
    assert_eq!(ws.current_objective.as_deref(), Some("Staff Infra at Stripe focusing on ClickHouse"));
}
