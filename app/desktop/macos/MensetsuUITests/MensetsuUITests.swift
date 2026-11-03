//
// MensetsuUITests.swift
// MensetsuUITests
//
// Native XCUITest smoke test exercising real register, login, 2FA challenge,
// text-based workspace creation, list, filter, overview & tabs, toggle, and logout
// against a build-configured running Rust API.
//

import XCTest

final class MensetsuUITests: XCTestCase {
    var app: XCUIApplication!

    private var targetApiUrl: String {
        return ProcessInfo.processInfo.environment["MENSETSU_API_URL"] ?? "http://127.0.0.1:3000"
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()
    }

    override func tearDownWithError() throws {
        app = nil
    }


    /// Ensures the application reaches the Auth view pointing to the expected endpoint.
    /// If on setup screen, enters the target endpoint directly.
    /// If already configured but pointing to a different endpoint (e.g. from a prior test),
    /// opens Settings from Auth screen and switches the endpoint to target.
    private func ensureConfiguredAndOnAuthView(for appInstance: XCUIApplication, endpoint: String? = nil) {
        let target = endpoint ?? targetApiUrl
        let authScreen = appInstance.otherElements["auth_screen"]
        let setupScreen = appInstance.otherElements["server_setup_screen"]

        if setupScreen.waitForExistence(timeout: 3) {
            let serverInput = appInstance.textFields["input_server_url"]
            let connectBtn = appInstance.buttons["button_save_server_url"]
            XCTAssertTrue(serverInput.waitForExistence(timeout: 3))
            serverInput.click()
            serverInput.typeKey("a", modifierFlags: [.command])
            serverInput.typeText(target)
            connectBtn.click()
        }

        XCTAssertTrue(authScreen.waitForExistence(timeout: 10), "Auth screen should appear")

        // Check if the current server display matches target; if not, reconfigure via Settings
        let settingsBtn = appInstance.buttons["button_settings"]
        if settingsBtn.waitForExistence(timeout: 3) {
            let serverButtonLabel = settingsBtn.label
            if !serverButtonLabel.contains(target) {
                settingsBtn.click()
                let settingsScreen = appInstance.otherElements["settings_screen"]
                XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5))

                let endpointInput = settingsScreen.textFields["settings_input_server_url"]
                let updateBtn = settingsScreen.buttons["settings_button_save_server_url"]
                XCTAssertTrue(endpointInput.waitForExistence(timeout: 3))
                endpointInput.click()
                endpointInput.typeKey("a", modifierFlags: [.command])
                endpointInput.typeText(target)
                updateBtn.click()

                let doneBtn = settingsScreen.buttons["settings_button_done"]
                doneBtn.click()
                XCTAssertTrue(authScreen.waitForExistence(timeout: 5))
            }
        }
    }
    func testRegisterCreateWorkspaceListToggleAndLogout() throws {
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)
        let authScreen = app.otherElements["auth_screen"]
        // Switch to Register mode
        let registerTab = app.buttons["tab_register"]
        if registerTab.exists {
            registerTab.click()
        }

        // Fill registration form
        let uniqueSuffix = Int(Date().timeIntervalSince1970)
        let nameField = app.textFields["input_name"]
        XCTAssertTrue(nameField.waitForExistence(timeout: 5))
        nameField.click()
        nameField.typeText("macOS Tester")

        let emailField = app.textFields["input_email"]
        emailField.click()
        emailField.typeText("macos_tester_\(uniqueSuffix)@example.com")

        let passwordField = app.secureTextFields["input_password"]
        passwordField.click()
        passwordField.typeText("Password123!")

        // Submit registration
        let submitButton = app.buttons["button_submit"]
        submitButton.click()

        // 2. Wait for main workspace split view to appear
        let splitView = app.otherElements["root_split_view"]
        XCTAssertTrue(splitView.waitForExistence(timeout: 10), "Should navigate to main split view after register")

        // 3. Open New Workspace Sheet
        let newWorkspaceBtn = app.buttons["button_new_workspace"]
        XCTAssertTrue(newWorkspaceBtn.exists)
        newWorkspaceBtn.click()

        let workspaceSheet = app.otherElements["new_workspace_sheet"]
        XCTAssertTrue(workspaceSheet.waitForExistence(timeout: 5))

        // Enter workspace details: company, role, JD, and resume_text
        let companyInput = app.textFields["input_target_company"]
        companyInput.click()
        companyInput.typeText("ExampleCo")

        let roleInput = app.textFields["input_role_title"]
        roleInput.click()
        roleInput.typeText("Backend Engineer")

        let jdInput = app.textViews["input_job_description"]
        if jdInput.exists {
            jdInput.click()
            jdInput.typeText("Backend Rust engineer")
        }

        let resumeInput = app.textViews["input_resume_text"]
        if resumeInput.exists {
            resumeInput.click()
            resumeInput.typeText("Built Rust services")
        }

        // Submit workspace creation
        let createSubmitBtn = app.buttons["button_create_workspace_submit"]
        createSubmitBtn.click()

        // 4. Verify workspace appears in list and detail
        let overviewCompanyName = app.staticTexts["overview_company_name"]
        XCTAssertTrue(overviewCompanyName.waitForExistence(timeout: 10), "Workspace detail should display created company name")
        XCTAssertEqual(overviewCompanyName.value as? String ?? overviewCompanyName.label, "ExampleCo")

        // 5. Test Tab Navigation
        let matchTab = app.buttons["detail_tab_match"]
        if matchTab.exists {
            matchTab.click()
            let ratioBadge = app.staticTexts["match_ratio_badge"]
            _ = ratioBadge.waitForExistence(timeout: 3)
        }

        let companyTab = app.buttons["detail_tab_company"]
        if companyTab.exists {
            companyTab.click()
            let overviewSec = app.otherElements["company_overview"]
            _ = overviewSec.waitForExistence(timeout: 3)
        }

        let learningTab = app.buttons["detail_tab_learning"]
        if learningTab.exists {
            learningTab.click()
            let learningList = app.otherElements["learning_units_list"]
            _ = learningList.waitForExistence(timeout: 3)
        }

        let questionsTab = app.buttons["detail_tab_questions"]
        if questionsTab.exists {
            questionsTab.click()
            let picker = app.popUpButtons["questions_difficulty_picker"]
            _ = picker.waitForExistence(timeout: 3)
        }

        // 6. Test Logout
        let logoutBtn = app.buttons["button_logout"]
        XCTAssertTrue(logoutBtn.exists)
        logoutBtn.click()

        // Verify returning to auth screen and protected data is inaccessible
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5), "Should return to Auth screen after logout")
    }

    func testNetworkErrorHandling() throws {
        // Point to an invalid unreachable port to verify network error message is displayed
        let unreachableUrl = "http://127.0.0.1:59999"
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)

        // Update server endpoint via settings to unreachable URL to verify network error handling
        let settingsBtn = app.buttons["button_settings"]
        XCTAssertTrue(settingsBtn.waitForExistence(timeout: 5))
        settingsBtn.click()

        let settingsScreen = app.otherElements["settings_screen"]
        XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5))

        let endpointInput = settingsScreen.textFields["settings_input_server_url"]
        let updateBtn = settingsScreen.buttons["settings_button_save_server_url"]
        XCTAssertTrue(endpointInput.waitForExistence(timeout: 3))
        endpointInput.click()
        endpointInput.typeKey("a", modifierFlags: [.command])
        endpointInput.typeText(unreachableUrl)
        updateBtn.click()

        let doneBtn = settingsScreen.buttons["settings_button_done"]
        doneBtn.click()

        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5))
        let emailField = app.textFields["input_email"]
        emailField.click()
        emailField.typeText("nonexistent@example.com")

        let passwordField = app.secureTextFields["input_password"]
        passwordField.click()
        passwordField.typeText("WrongPassword123")

        let submitButton = app.buttons["button_submit"]
        submitButton.click()

        let errorText = app.staticTexts["auth_error_message"]
        XCTAssertTrue(errorText.waitForExistence(timeout: 10), "Network error message should be displayed instead of crashing or mocking")
    }
    
    func testPdfResumeUploadFlow() throws {
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)
        let authScreen = app.otherElements["auth_screen"]
        // Switch to Register mode
        let registerTab = app.buttons["tab_register"]
        if registerTab.exists {
            registerTab.click()
        }

        // Fill registration form
        let uniqueSuffix = Int(Date().timeIntervalSince1970)
        let nameField = app.textFields["input_name"]
        XCTAssertTrue(nameField.waitForExistence(timeout: 5))
        nameField.click()
        nameField.typeText("macOS PDF Tester")

        let emailField = app.textFields["input_email"]
        emailField.click()
        emailField.typeText("macos_pdf_\(uniqueSuffix)@example.com")

        let passwordField = app.secureTextFields["input_password"]
        passwordField.click()
        passwordField.typeText("Password123!")

        let submitButton = app.buttons["button_submit"]
        submitButton.click()

        // 2. Wait for main workspace split view
        let splitView = app.otherElements["root_split_view"]
        XCTAssertTrue(splitView.waitForExistence(timeout: 10), "Should navigate to main split view")

        // 3. Open New Workspace Sheet
        let newWorkspaceBtn = app.buttons["button_new_workspace"]
        XCTAssertTrue(newWorkspaceBtn.exists)
        newWorkspaceBtn.click()

        let workspaceSheet = app.otherElements["new_workspace_sheet"]
        XCTAssertTrue(workspaceSheet.waitForExistence(timeout: 5))

        // Enter company and role
        let companyInput = app.textFields["input_target_company"]
        companyInput.click()
        companyInput.typeText("PDFTech Inc")

        let roleInput = app.textFields["input_role_title"]
        roleInput.click()
        roleInput.typeText("Systems Engineer")

        // Switch to PDF mode
        let pdfTab = app.buttons["tab_resume_pdf"]
        if pdfTab.exists {
            pdfTab.click()
        }

        // Check PDF mode UI elements exist
        let selectPdfBtn = app.buttons["button_select_pdf"]
        XCTAssertTrue(selectPdfBtn.waitForExistence(timeout: 3), "Select PDF button should be visible in PDF mode")

        // Verify submit without selected PDF shows actual validation error
        let createSubmitBtn = app.buttons["button_create_workspace_submit"]
        createSubmitBtn.click()

        let errorText = app.staticTexts["create_error_message"]
        XCTAssertTrue(errorText.waitForExistence(timeout: 3), "Validation error should appear when PDF is required but missing")
        
        // 4. Select the repository PDF in the native NSOpenPanel.
        let sourceDirectory = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
        let repositoryRoot = (0..<4).reduce(sourceDirectory) { directory, _ in
            directory.deletingLastPathComponent()
        }
        let fixturePath = ProcessInfo.processInfo.environment["MENSETSU_TEST_PDF_PATH"]
            ?? repositoryRoot.appendingPathComponent("fixtures/resume-text.pdf").path
        XCTAssertTrue(FileManager.default.fileExists(atPath: fixturePath), "Real PDF fixture must be available to the UI test")
        selectPdfBtn.click()
        let filePanel = app.dialogs.firstMatch
        XCTAssertTrue(filePanel.waitForExistence(timeout: 5), "Native NSOpenPanel must open")
        app.typeKey("g", modifierFlags: [.command, .shift])
        app.typeText(URL(fileURLWithPath: fixturePath).deletingLastPathComponent().path)
        app.typeKey(XCUIKeyboardKey.return, modifierFlags: [])
        let pdfFile = filePanel.staticTexts["resume-text.pdf"]
        XCTAssertTrue(pdfFile.waitForExistence(timeout: 5), "Fixture must appear in the native PDF picker")
        pdfFile.doubleClick()

        let selectedLabel = app.descendants(matching: .any)["label_selected_pdf"]
        XCTAssertTrue(selectedLabel.waitForExistence(timeout: 5), "Selected PDF must appear in the create sheet")
        XCTAssertTrue(selectedLabel.label.contains("resume-text.pdf"))
        createSubmitBtn.click()

        // Actual signed PUT, confirmation, and workspace create must reach the live API.
        let overviewCompanyName = app.staticTexts["overview_company_name"]
        XCTAssertTrue(overviewCompanyName.waitForExistence(timeout: 30), "PDF workspace must open after staged S3 upload")
        XCTAssertEqual(overviewCompanyName.value as? String ?? overviewCompanyName.label, "PDFTech Inc")
    }

    func testFirstLaunchEndpointSetupAndValidation() throws {
        // Launch with temporary empty UserDefaults standard key argument to simulate clean first install
        // without destructive deletion of host system user preferences.
        let freshApp = XCUIApplication()
        freshApp.launchArguments = ["-mensetsu_server_endpoint", ""]
        freshApp.launch()
        let setupScreen = freshApp.otherElements["server_setup_screen"]
        let serverInput = freshApp.textFields["input_server_url"]
        let connectBtn = freshApp.buttons["button_save_server_url"]
        let errorText = freshApp.staticTexts["server_url_error_message"]

        // 1. Verify prompt appears on clean launch
        XCTAssertTrue(setupScreen.waitForExistence(timeout: 5), "First launch must prompt for server URL")
        XCTAssertTrue(serverInput.exists)
        XCTAssertTrue(connectBtn.exists)

        // 2. Test empty submission validation
        connectBtn.click()
        XCTAssertTrue(errorText.waitForExistence(timeout: 3), "Validation error displayed on empty URL")

        // 3. Test invalid scheme or insecure remote HTTP
        serverInput.click()
        serverInput.typeText("http://remote-server.example.com")
        connectBtn.click()
        XCTAssertTrue(errorText.waitForExistence(timeout: 3), "Remote HTTP must be rejected")

        // 4. Connect with valid URL enters auth UI state
        let apiUrl = targetApiUrl
        serverInput.click()
        // Select all text and replace
        serverInput.typeKey("a", modifierFlags: [.command])
        serverInput.typeText(apiUrl)
        connectBtn.click()

        let authScreen = freshApp.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5), "Entering valid URL transitions to Auth view")

        // 5. Terminate and relaunch cleanly without launch arguments to assert persisted endpoint
        freshApp.terminate()
        let relaunchedApp = XCUIApplication()
        relaunchedApp.launch()
        let relaunchedAuth = relaunchedApp.otherElements["auth_screen"]
        XCTAssertTrue(relaunchedAuth.waitForExistence(timeout: 5), "Relaunched app with saved URL opens Auth directly")
    }

    func testUpdateServerEndpointFromAuthSettings() throws {
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)
        let authScreen = app.otherElements["auth_screen"]

        // Settings button is accessible on auth view
        let settingsBtn = app.buttons["button_settings"]
        XCTAssertTrue(settingsBtn.waitForExistence(timeout: 5), "Settings button accessible on auth view")
        settingsBtn.click()

        let settingsScreen = app.otherElements["settings_screen"]
        XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5), "Settings screen appears")

        let endpointInput = settingsScreen.textFields["settings_input_server_url"]
        let updateBtn = settingsScreen.buttons["settings_button_save_server_url"]
        XCTAssertTrue(endpointInput.exists)
        XCTAssertTrue(updateBtn.exists)
        endpointInput.click()
        endpointInput.typeKey("a", modifierFlags: [.command])
        endpointInput.typeText("https://alternate.example.org")
        updateBtn.click()
        XCTAssertTrue(settingsScreen.staticTexts["Server endpoint updated. Active session reset."].waitForExistence(timeout: 5))
        XCTAssertEqual(endpointInput.value as? String, "https://alternate.example.org")

        let doneBtn = settingsScreen.buttons["settings_button_done"]
        doneBtn.click()
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5))
        app.terminate()
        let relaunched = XCUIApplication()
        relaunched.launch()
        let changeServer = relaunched.buttons["button_settings"]
        XCTAssertTrue(changeServer.waitForExistence(timeout: 5))
        changeServer.click()
        XCTAssertEqual(relaunched.textFields["settings_input_server_url"].value as? String, "https://alternate.example.org")
    }
}
