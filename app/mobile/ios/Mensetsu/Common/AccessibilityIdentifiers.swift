
import Foundation

public enum AccessibilityID {
    public static let serverSetupScreen = "server_setup_screen"
    public static let inputServerUrl = "input_server_url"
    public static let buttonSaveServerUrl = "button_save_server_url"
    public static let serverUrlErrorMessage = "server_url_error_message"

    public static let settingsServerUrlInput = "settings_input_server_url"
    public static let settingsSaveServerUrlButton = "settings_button_save_server_url"
    public static let openServerSettingsButton = "open_server_settings_button"
    public static let serverSettingsSheet = "server_settings_sheet"

    public static let authScreen = "auth_screen"
    public static let authModeSegment = "auth_mode_segment"
    public static let authLoginTab = "auth_login_tab"
    public static let authRegisterTab = "auth_register_tab"
    public static let emailField = "auth_email_field"
    public static let passwordField = "auth_password_field"
    public static let nameField = "auth_name_field"
    public static let loginButton = "auth_login_button"
    public static let registerButton = "auth_register_button"
    public static let authErrorBanner = "auth_error_banner"
    public static let authLoadingIndicator = "auth_loading_indicator"

    public static let twoFaScreen = "2fa_screen"
    public static let twoFaCodeField = "2fa_code_field"
    public static let twoFaVerifyButton = "2fa_verify_button"
    public static let twoFaErrorBanner = "2fa_error_banner"

    public static let workspaceListScreen = "workspace_list_screen"
    public static let workspaceLoadingIndicator = "workspace_loading_indicator"
    public static let workspaceErrorBanner = "workspace_error_banner"
    public static let workspaceList = "workspace_list"
    public static let workspaceFilterSegment = "workspace_filter_segment"
    public static let filterAll = "workspace_filter_all"
    public static let filterReady = "workspace_filter_ready"
    public static let filterProcessing = "workspace_filter_processing"
    public static let openCreateWorkspaceButton = "open_create_workspace_button"
    public static let openSettingsButton = "open_settings_button"
    public static let logoutButton = "logout_button"

    public static func workspaceRow(id: String) -> String {
        return "workspace_row_\(id)"
    }

    public static let createWorkspaceSheet = "create_workspace_sheet"
    public static let createCompanyField = "create_company_field"
    public static let createRoleField = "create_role_field"
    public static let createResumeTextField = "create_resume_text_field"
    public static let createResumeSourcePicker = "create_resume_source_picker"
    public static let createResumePDFButton = "create_resume_pdf_button"
    public static let createResumePDFName = "create_resume_pdf_name"
    public static let createJobDescriptionField = "create_job_description_field"
    public static let submitCreateWorkspaceButton = "submit_create_workspace_button"
    public static let cancelCreateWorkspaceButton = "cancel_create_workspace_button"
    public static let createWorkspaceErrorBanner = "create_workspace_error_banner"

    public static let workspaceDetailScreen = "workspace_detail_screen"
    public static let detailLoadingIndicator = "detail_loading_indicator"
    public static let detailErrorBanner = "detail_error_banner"
    public static let detailTabBar = "detail_tab_bar"
    public static let tabOverview = "detail_tab_overview"
    public static let tabMatch = "detail_tab_match"
    public static let tabCompany = "detail_tab_company"
    public static let tabLearning = "detail_tab_learning"
    public static let tabQuestions = "detail_tab_questions"

    public static let overviewCompanyName = "overview_company_name"
    public static let overviewRoleTitle = "overview_role_title"
    public static let overviewStatusBadge = "overview_status_badge"
    public static let overviewMatchMetric = "overview_match_metric"
    public static let overviewGapsMetric = "overview_gaps_metric"
    public static let overviewQuestionsMetric = "overview_questions_metric"

    public static let matchRatioText = "match_ratio_text"
    public static let matchStrengthsList = "match_strengths_list"
    public static let matchGapsList = "match_gaps_list"
    public static let matchRisksList = "match_risks_list"

    public static let companyOverviewText = "company_overview_text"
    public static let companyTechStackList = "company_tech_stack_list"
    public static let companyCultureList = "company_culture_list"
    public static let companyCitationsList = "company_citations_list"

    public static let learningUnitsList = "learning_units_list"
    public static func lessonRow(id: String) -> String {
        return "lesson_row_\(id)"
    }
    public static func lessonToggle(id: String) -> String {
        return "lesson_toggle_\(id)"
    }

    public static let questionsDifficultyPicker = "questions_difficulty_picker"
    public static let questionsList = "questions_list"
    public static func questionRow(id: String) -> String {
        return "question_row_\(id)"
    }

    public static let settingsScreen = "settings_screen"
    public static let settingsLoadingIndicator = "settings_loading_indicator"
    public static let settingsErrorBanner = "settings_error_banner"
    public static let llmProvidersList = "llm_providers_list"
    public static let settingsDoneButton = "settings_done_button"
    public static func providerRow(provider: String) -> String {
        return "provider_row_\(provider)"
    }
    public static func providerKeyField(provider: String) -> String {
        return "provider_key_field_\(provider)"
    }
    public static func providerSaveButton(provider: String) -> String {
        return "provider_save_button_\(provider)"
    }
}
