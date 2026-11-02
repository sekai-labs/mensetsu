package org.mensetsu.app.core.tags

/**
 * Stable accessibility & testTag identifiers for UI automation and CI smoke testing.
 */
object AppTestTags {
    // Auth & Navigation
    const val AUTH_SCREEN = "auth_screen"
    const val AUTH_TAB_LOGIN = "auth_tab_login"
    const val AUTH_TAB_REGISTER = "auth_tab_register"
    const val AUTH_INPUT_NAME = "auth_input_name"
    const val AUTH_INPUT_EMAIL = "auth_input_email"
    const val AUTH_INPUT_PASSWORD = "auth_input_password"
    const val AUTH_BUTTON_SUBMIT = "auth_button_submit"
    const val AUTH_ERROR_TEXT = "auth_error_text"

    // 2FA Dialog
    const val DIALOG_2FA = "dialog_2fa"
    const val INPUT_2FA_CODE = "input_2fa_code"
    const val BUTTON_2FA_VERIFY = "button_2fa_verify"

    // Workspace List
    const val WORKSPACE_LIST_SCREEN = "workspace_list_screen"
    const val WORKSPACE_LIST_LOADING = "workspace_list_loading"
    const val WORKSPACE_LIST_ERROR = "workspace_list_error"
    const val WORKSPACE_LIST_EMPTY = "workspace_list_empty"
    const val WORKSPACE_LIST_RETRY_BUTTON = "workspace_list_retry_button"
    const val WORKSPACE_ITEM_PREFIX = "workspace_item_"
    const val FILTER_TAB_ALL = "filter_tab_all"
    const val FILTER_TAB_READY = "filter_tab_ready"
    const val FILTER_TAB_PROCESSING = "filter_tab_processing"
    const val BUTTON_NEW_WORKSPACE = "button_new_workspace"
    const val BUTTON_SETTINGS = "button_settings"
    const val BUTTON_LOGOUT = "button_logout"

    // Create Workspace Dialog
    const val DIALOG_CREATE_WORKSPACE = "dialog_create_workspace"
    const val INPUT_CREATE_COMPANY = "input_create_company"
    const val INPUT_CREATE_ROLE = "input_create_role"
    const val INPUT_CREATE_RESUME_TEXT = "input_create_resume_text"
    const val BUTTON_CREATE_PDF_MODE = "button_create_pdf_mode"
    const val BUTTON_CREATE_TEXT_MODE = "button_create_text_mode"
    const val BUTTON_CREATE_PICK_PDF = "button_create_pick_pdf"
    const val CREATE_PDF_FILENAME = "create_pdf_filename"
    const val INPUT_CREATE_JOB_DESC = "input_create_job_desc"
    const val BUTTON_CREATE_SUBMIT = "button_create_submit"
    const val BUTTON_CREATE_CANCEL = "button_create_cancel"
    const val CREATE_WORKSPACE_ERROR = "create_workspace_error"

    // Workspace Detail
    const val WORKSPACE_DETAIL_SCREEN = "workspace_detail_screen"
    const val DETAIL_LOADING = "detail_loading"
    const val DETAIL_ERROR = "detail_error"
    const val DETAIL_TAB_OVERVIEW = "detail_tab_overview"
    const val DETAIL_TAB_MATCH = "detail_tab_match"
    const val DETAIL_TAB_COMPANY = "detail_tab_company"
    const val DETAIL_TAB_LEARNING = "detail_tab_learning"
    const val DETAIL_TAB_QUESTIONS = "detail_tab_questions"
    const val DETAIL_TAB_SETTINGS = "detail_tab_settings"
    const val BUTTON_DETAIL_BACK = "button_detail_back"

    // Detail - Learning & Toggle
    const val LEARNING_UNIT_PREFIX = "learning_unit_"
    const val LEARNING_TOGGLE_PREFIX = "learning_toggle_"

    // Detail - Questions
    const val QUESTIONS_FILTER_DIFFICULTY = "questions_filter_difficulty"
    const val QUESTION_CARD_PREFIX = "question_card_"

    // Settings
    const val SETTINGS_SCREEN = "settings_screen"
    const val SETTINGS_PROVIDER_PREFIX = "settings_provider_"
    const val SETTINGS_KEY_INPUT = "settings_key_input"
    const val SETTINGS_SAVE_BUTTON = "settings_save_button"
}
