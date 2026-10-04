#[cfg(test)]
mod tests {
    use mensetsu_api::contexts::{
        identity::domain::UserId,
        workspace::domain::{PipelinePhase, SeniorityLevel, Workspace, WorkspaceStatus},
    };

    #[test]
    fn test_workspace_creation_valid() {
        let user_id = UserId("user_123".into());
        let ws = Workspace::new(
            user_id.clone(),
            "Anthropic".into(),
            "Systems Engineer".into(),
            Some(SeniorityLevel::Staff),
            None,
            Some(4),
        )
        .expect("Valid workspace should be created");

        assert_eq!(ws.company_name, "Anthropic");
        assert_eq!(ws.role_title, "Systems Engineer");
        assert_eq!(ws.status, WorkspaceStatus::Pending);
        assert_eq!(ws.current_phase, PipelinePhase::PreparingInput);
        assert_eq!(ws.progress_percent, 0);
        assert_eq!(ws.seniority, Some(SeniorityLevel::Staff));
    }

    #[test]
    fn test_workspace_creation_empty_company_fails() {
        let user_id = UserId("user_123".into());
        let err = Workspace::new(user_id, "   ".into(), "Systems Engineer".into(), None, None, None);
        assert!(err.is_err(), "Empty company should fail validation");
    }

    #[test]
    fn test_workspace_creation_empty_role_fails() {
        let user_id = UserId("user_123".into());
        let err = Workspace::new(user_id, "Anthropic".into(), "   ".into(), None, None, None);
        assert!(err.is_err(), "Empty role title should fail validation");
    }

    #[test]
    fn test_workspace_progress_transitions() {
        let user_id = UserId("user_123".into());
        let mut ws = Workspace::new(
            user_id,
            "Anthropic".into(),
            "Systems Engineer".into(),
            None,
            None,
            None,
        )
        .unwrap();

        ws.advance_progress(PipelinePhase::ResearchingCompany, 35);
        assert_eq!(ws.status, WorkspaceStatus::Processing);
        assert_eq!(ws.current_phase, PipelinePhase::ResearchingCompany);
        assert_eq!(ws.progress_percent, 35);

        ws.advance_progress(PipelinePhase::Ready, 100);
        assert_eq!(ws.status, WorkspaceStatus::Ready);
        assert_eq!(ws.current_phase, PipelinePhase::Ready);
        assert_eq!(ws.progress_percent, 100);
    }
}
