namespace Mensetsu.Common
{
    public static class AutomationIds
    {
        // Server setup screen (first launch & change server)
        public const string ServerSetupScreen = "server_setup_screen";
        public const string ServerUrlInput = "input_server_url";
        public const string SaveServerUrlButton = "button_save_server_url";
        public const string ServerUrlErrorBanner = "server_url_error_message";
        public const string CancelServerUrlButton = "button_cancel_server_url";

        // Auth screen
        public const string AuthScreen = "auth_screen";
        public const string AuthLoginTab = "auth_login_tab";
        public const string AuthRegisterTab = "auth_register_tab";
        public const string AuthEmailInput = "auth_email_field";
        public const string AuthPasswordInput = "auth_password_field";
        public const string AuthNameInput = "auth_name_field";
        public const string AuthLoginButton = "auth_login_button";
        public const string AuthRegisterButton = "auth_register_button";
        public const string AuthErrorBanner = "auth_error_banner";
        public const string AuthLoadingIndicator = "auth_loading_indicator";
        public const string AuthChangeServerButton = "auth_change_server_button";

        // 2FA challenge
        public const string TwoFaScreen = "2fa_screen";
        public const string TwoFaCodeInput = "2fa_code_field";
        public const string TwoFaVerifyButton = "2fa_verify_button";
        public const string TwoFaErrorBanner = "2fa_error_banner";

        // Workspace list
        public const string WorkspaceListScreen = "workspace_list_screen";
        public const string WorkspaceLoadingIndicator = "workspace_loading_indicator";
        public const string WorkspaceErrorBanner = "workspace_error_banner";
        public const string WorkspaceList = "workspace_list";
        public const string FilterAllButton = "workspace_filter_all";
        public const string FilterReadyButton = "workspace_filter_ready";
        public const string FilterProcessingButton = "workspace_filter_processing";
        public const string OpenCreateWorkspaceButton = "open_create_workspace_button";
        public const string OpenSettingsButton = "open_settings_button";
        public const string LogoutButton = "logout_button";
        public const string RefreshWorkspacesButton = "refresh_workspaces_button";
        public static string WorkspaceRow(string id) => $"workspace_row_{id}";

        // Create workspace dialog / form
        public const string CreateWorkspaceDialog = "create_workspace_sheet";
        public const string CreateCompanyInput = "create_company_field";
        public const string CreateRoleInput = "create_role_field";
        public const string CreateResumeTextInput = "create_resume_text_field";
        public const string CreateJobDescriptionInput = "create_job_description_field";
        public const string SubmitCreateWorkspaceButton = "submit_create_workspace_button";
        public const string CancelCreateWorkspaceButton = "cancel_create_workspace_button";
        public const string CreateWorkspaceErrorBanner = "create_workspace_error_banner";
        public const string ResumeModeTextRadio = "resume_mode_text_radio";
        public const string ResumeModePdfRadio = "resume_mode_pdf_radio";
        public const string PickPdfFileButton = "pick_pdf_file_button";
        public const string SelectedPdfFileText = "selected_pdf_file_text";

        // Detail shell & tabs
        public const string WorkspaceDetailScreen = "workspace_detail_screen";
        public const string DetailLoadingIndicator = "detail_loading_indicator";
        public const string DetailErrorBanner = "detail_error_banner";
        public const string DetailBackButton = "detail_back_button";
        public const string TabOverview = "detail_tab_overview";
        public const string TabMatch = "detail_tab_match";
        public const string TabCompany = "detail_tab_company";
        public const string TabLearning = "detail_tab_learning";
        public const string TabQuestions = "detail_tab_questions";

        // Overview
        public const string OverviewCompanyName = "overview_company_name";
        public const string OverviewRoleTitle = "overview_role_title";
        public const string OverviewStatusBadge = "overview_status_badge";
        public const string OverviewMatchMetric = "overview_match_metric";
        public const string OverviewGapsMetric = "overview_gaps_metric";
        public const string OverviewQuestionsMetric = "overview_questions_metric";

        // Match
        public const string MatchRatioText = "match_ratio_text";
        public const string MatchStrengthsList = "match_strengths_list";
        public const string MatchGapsList = "match_gaps_list";
        public const string MatchRisksList = "match_risks_list";

        // Company
        public const string CompanyOverviewText = "company_overview_text";
        public const string CompanyTechStackList = "company_tech_stack_list";
        public const string CompanyCultureList = "company_culture_list";
        public const string CompanyCitationsList = "company_citations_list";

        // Learning
        public const string LearningUnitsList = "learning_units_list";
        public static string LessonRow(string id) => $"lesson_row_{id}";
        public static string LessonToggle(string id) => $"lesson_toggle_{id}";

        // Questions
        public const string QuestionsDifficultyPicker = "questions_difficulty_picker";
        public const string QuestionsList = "questions_list";
        public static string QuestionRow(string id) => $"question_row_{id}";

        // Settings
        public const string SettingsScreen = "settings_screen";
        public const string SettingsLoadingIndicator = "settings_loading_indicator";
        public const string SettingsErrorBanner = "settings_error_banner";
        public const string SettingsBackButton = "settings_back_button";
        public const string SettingsServerUrlInput = "settings_input_server_url";
        public const string SettingsServerUrlSaveButton = "settings_button_save_server_url";
        public const string LlmProvidersList = "llm_providers_list";
        public const string SettingsProviderInput = "settings_provider_input";
        public const string SettingsKeyInput = "settings_key_input";
        public const string SettingsSaveButton = "settings_save_button";
        public static string ProviderRow(string provider) => $"provider_row_{provider}";
        public static string ProviderKeyField(string provider) => $"provider_key_field_{provider}";
        public static string ProviderSaveButton(string provider) => $"provider_save_button_{provider}";
    }
}
