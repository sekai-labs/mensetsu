//
// AccessibilityIdentifiers.swift
// Mensetsu macOS
//

import Foundation

public enum AccessibilityIdentifiers {
    // Auth Screen
    public static let authScreen = "auth_screen"
    public static let tabLogin = "tab_login"
    public static let tabRegister = "tab_register"
    public static let inputEmail = "input_email"
    public static let inputPassword = "input_password"
    public static let inputName = "input_name"
    public static let buttonSubmit = "button_submit"
    public static let authErrorMessage = "auth_error_message"
    
    // 2FA Challenge
    public static let challenge2faSheet = "challenge_2fa_sheet"
    public static let input2faCode = "input_2fa_code"
    public static let buttonSubmit2fa = "button_submit_2fa"
    public static let buttonCancel2fa = "button_cancel_2fa"
    
    // Main & Navigation
    public static let rootSplitView = "root_split_view"
    public static let sidebar = "sidebar"
    public static let buttonLogout = "button_logout"
    public static let buttonNewWorkspace = "button_new_workspace"
    public static let buttonSettings = "button_settings"
    public static let filterSegment = "filter_segment"
    public static let filterAll = "filter_all"
    public static let filterReady = "filter_ready"
    public static let filterProcessing = "filter_processing"
    public static let workspaceList = "workspace_list"
    public static let workspaceRowPrefix = "workspace_row_"
    public static let statusBadgePrefix = "status_badge_"
    
    // Workspace Creation Sheet / Form
    public static let newWorkspaceSheet = "new_workspace_sheet"
    public static let inputTargetCompany = "input_target_company"
    public static let inputRoleTitle = "input_role_title"
    public static let inputJobDescription = "input_job_description"
    public static let pickerResumeMode = "picker_resume_mode"
    public static let tabResumeText = "tab_resume_text"
    public static let tabResumePdf = "tab_resume_pdf"
    public static let inputResumeText = "input_resume_text"
    public static let buttonSelectPdf = "button_select_pdf"
    public static let labelSelectedPdf = "label_selected_pdf"
    public static let buttonClearPdf = "button_clear_pdf"
    public static let buttonCreateWorkspaceSubmit = "button_create_workspace_submit"
    public static let buttonCancelCreateWorkspace = "button_cancel_create_workspace"
    public static let createErrorMessage = "create_error_message"
    
    // Workspace Detail
    public static let detailContainer = "detail_container"
    public static let detailTabBar = "detail_tab_bar"
    public static let tabOverview = "detail_tab_overview"
    public static let tabMatch = "detail_tab_match"
    public static let tabCompany = "detail_tab_company"
    public static let tabLearning = "detail_tab_learning"
    public static let tabQuestions = "detail_tab_questions"
    public static let tabLlmSettings = "detail_tab_llm_settings"
    public static let detailLoadingIndicator = "detail_loading_indicator"
    public static let detailErrorMessage = "detail_error_message"
    public static let detailRetryButton = "detail_retry_button"
    
    // Overview Screen
    public static let overviewCompanyName = "overview_company_name"
    public static let overviewRoleTitle = "overview_role_title"
    public static let overviewStatusBadge = "overview_status_badge"
    public static let overviewReadiness = "overview_readiness"
    
    // Match Analysis
    public static let matchRatioBadge = "match_ratio_badge"
    public static let matchStrengthsList = "match_strengths_list"
    public static let matchGapsList = "match_gaps_list"
    public static let matchRisksList = "match_risks_list"
    
    // Company Dossier
    public static let companyOverview = "company_overview"
    public static let companyTechStack = "company_tech_stack"
    public static let companyCitationsList = "company_citations_list"
    
    // Learning Path
    public static let learningUnitsList = "learning_units_list"
    public static let learningUnitRowPrefix = "learning_unit_row_"
    public static let learningTogglePrefix = "learning_toggle_"
    
    // Question Book
    public static let questionsDifficultyPicker = "questions_difficulty_picker"
    public static let questionsList = "questions_list"
    public static let questionRowPrefix = "question_row_"
    
    // Settings Screen
    public static let settingsScreen = "settings_screen"
    public static let llmProvidersList = "llm_providers_list"
    public static let llmProviderRowPrefix = "llm_provider_row_"
    public static let llmConfigureKeyPrefix = "llm_configure_key_"
    public static let llmKeySheet = "llm_key_sheet"
    public static let inputLlmApiKey = "input_llm_api_key"
    public static let buttonSaveLlmKey = "button_save_llm_key"
    public static let buttonCancelLlmKey = "button_cancel_llm_key"
}
