//
// MensetsuUITests.swift
// MensetsuUITests
//

import XCTest

final class MensetsuUITests: XCTestCase {

    private var targetApiUrl: String {
        return ProcessInfo.processInfo.environment["MENSETSU_API_BASE_URL"]
            ?? ProcessInfo.processInfo.environment["MENSETSU_API_URL"]
            ?? "http://localhost:8080"
    }

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    /// Ensures the application reaches the Auth view pointing to the expected endpoint.
    /// If on setup screen, enters the target endpoint directly.
    /// If already configured but pointing to a different endpoint (e.g. from a prior test),
    /// opens Settings from Auth screen and switches the endpoint to target.
    private func ensureConfiguredAndOnAuthView(for appInstance: XCUIApplication, endpoint: String? = nil) {
        let target = endpoint ?? targetApiUrl
        let authScreen = appInstance.otherElements["auth_screen"]
        let setupScreen = appInstance.otherElements["server_setup_screen"]

        if setupScreen.waitForExistence(timeout: 3.0) {
            let serverInput = appInstance.textFields["input_server_url"]
            let connectBtn = appInstance.buttons["button_save_server_url"]
            XCTAssertTrue(serverInput.waitForExistence(timeout: 3.0))
            serverInput.tap()
            let currentVal = serverInput.value as? String ?? ""
            if !currentVal.isEmpty {
                let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: currentVal.count)
                serverInput.typeText(deleteString)
            }
            serverInput.typeText(target)
            connectBtn.tap()
        }

        XCTAssertTrue(authScreen.waitForExistence(timeout: 10.0), "Auth screen should appear")

        // Check if active server matches target; if not, reconfigure via pre-auth settings
        let openServerBtn = appInstance.buttons["open_server_settings_button"]
        if openServerBtn.waitForExistence(timeout: 3.0) {
            let buttonLabel = openServerBtn.label
            if !buttonLabel.contains(target) {
                openServerBtn.tap()
                let settingsScreen = appInstance.otherElements["settings_screen"]
                XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5.0))

                let serverInput = appInstance.textFields["settings_input_server_url"]
                let saveBtn = appInstance.buttons["settings_button_save_server_url"]
                XCTAssertTrue(serverInput.waitForExistence(timeout: 3.0))
                serverInput.tap()
                let currentVal = serverInput.value as? String ?? ""
                if !currentVal.isEmpty {
                    let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: currentVal.count)
                    serverInput.typeText(deleteString)
                }
                serverInput.typeText(target)
                saveBtn.tap()

                let doneBtn = appInstance.buttons["settings_done_button"]
                if doneBtn.waitForExistence(timeout: 2.0) {
                    doneBtn.tap()
                }
                XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0))
            }
        }
    }

    func testRealAuthCreateListDetailLogoutFlow() throws {
        let app = XCUIApplication()
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)

        // 1. Verify Auth Screen is present
        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0), "Auth screen should appear")

        // 2. Switch to Register mode
        let registerTab = app.buttons["auth_register_tab"]
        if registerTab.exists {
            registerTab.tap()
        }
        let nameField = app.textFields["auth_name_field"]
        XCTAssertTrue(nameField.waitForExistence(timeout: 2.0))
        let randomSuffix = UUID().uuidString.lowercased()
        let testEmail = "ios_smoke_\(randomSuffix)@example.com"
        let testPassword = "Password123!"

        nameField.tap()
        nameField.typeText("iOS Test User")

        let emailField = app.textFields["auth_email_field"]
        emailField.tap()
        emailField.typeText(testEmail)

        let passwordField = app.secureTextFields["auth_password_field"]
        passwordField.tap()
        passwordField.typeText(testPassword)

        let registerButton = app.buttons["auth_register_button"]
        XCTAssertTrue(registerButton.isEnabled)
        registerButton.tap()

        // 3. Verify transition to Workspace List Screen
        let workspaceListScreen = app.otherElements["workspace_list_screen"]
        XCTAssertTrue(workspaceListScreen.waitForExistence(timeout: 10.0), "Workspace list screen should appear after register")

        // 4. Open Create Workspace Sheet
        let openCreateBtn = app.buttons["open_create_workspace_button"]
        XCTAssertTrue(openCreateBtn.waitForExistence(timeout: 3.0))
        openCreateBtn.tap()

        let createSheet = app.otherElements["create_workspace_sheet"]
        XCTAssertTrue(createSheet.waitForExistence(timeout: 3.0))

        let companyField = app.textFields["create_company_field"]
        companyField.tap()
        let testCompany = "ExampleCo_\(randomSuffix)"
        companyField.typeText(testCompany)

        let roleField = app.textFields["create_role_field"]
        roleField.tap()
        roleField.typeText("Backend Engineer")

        let resumeField = app.textViews["create_resume_text_field"]
        resumeField.tap()
        resumeField.typeText("Built Rust services and event streaming architectures.")

        let jdField = app.textViews["create_job_description_field"]
        jdField.tap()
        jdField.typeText("Backend Rust engineer with distributed systems background.")

        let submitCreateBtn = app.buttons["submit_create_workspace_button"]
        XCTAssertTrue(submitCreateBtn.isEnabled)
        submitCreateBtn.tap()

        // 5. Verify workspace appears in list
        let createdRow = app.staticTexts[testCompany]
        XCTAssertTrue(createdRow.waitForExistence(timeout: 10.0), "Newly created workspace should appear in list")

        // 6. Navigate to Workspace Detail
        createdRow.tap()

        let detailScreen = app.otherElements["workspace_detail_screen"]
        XCTAssertTrue(detailScreen.waitForExistence(timeout: 5.0), "Detail screen should appear")

        // Verify tabs can be switched
        let matchTab = app.buttons["detail_tab_match"]
        if matchTab.exists {
            matchTab.tap()
        }

        let companyTab = app.buttons["detail_tab_company"]
        if companyTab.exists {
            companyTab.tap()
        }

        let learningTab = app.buttons["detail_tab_learning"]
        if learningTab.exists {
            learningTab.tap()
        }

        let questionsTab = app.buttons["detail_tab_questions"]
        if questionsTab.exists {
            questionsTab.tap()
        }

        // Return to workspace list
        app.navigationBars.buttons.element(boundBy: 0).tap()
        XCTAssertTrue(workspaceListScreen.waitForExistence(timeout: 5.0))
        // 7. Import a real PDF from Files and create a second workspace.
        openCreateBtn.tap()
        let pdfCompany = "PDFCo_\(randomSuffix)"
        let pdfCompanyField = app.textFields["create_company_field"]
        XCTAssertTrue(pdfCompanyField.waitForExistence(timeout: 3.0))
        pdfCompanyField.tap()
        pdfCompanyField.typeText(pdfCompany)
        let pdfRoleField = app.textFields["create_role_field"]
        pdfRoleField.tap()
        pdfRoleField.typeText("Backend Engineer")
        app.segmentedControls["create_resume_source_picker"].buttons["PDF"].tap()
        app.buttons["create_resume_pdf_button"].tap()
        let browse = app.buttons["Browse"]
        if browse.waitForExistence(timeout: 3.0) {
            browse.tap()
        }
        let localFiles = app.staticTexts["On My iPhone"]
        XCTAssertTrue(localFiles.waitForExistence(timeout: 5.0))
        localFiles.tap()
        let appFolder = app.staticTexts["Mensetsu"]
        XCTAssertTrue(appFolder.waitForExistence(timeout: 5.0))
        appFolder.tap()
        let pdfFile = app.staticTexts["resume-text.pdf"]
        XCTAssertTrue(pdfFile.waitForExistence(timeout: 5.0))
        pdfFile.tap()
        XCTAssertTrue(app.staticTexts["create_resume_pdf_name"].waitForExistence(timeout: 3.0))
        let pdfCreateButton = app.buttons["submit_create_workspace_button"]
        XCTAssertTrue(pdfCreateButton.isEnabled)
        pdfCreateButton.tap()
        XCTAssertTrue(app.staticTexts[pdfCompany].waitForExistence(timeout: 30.0), "PDF workspace should appear in the list")

        // 8. Verify logout.

        let logoutBtn = app.buttons["logout_button"]
        XCTAssertTrue(logoutBtn.exists)
        logoutBtn.tap()

        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0), "Auth screen should appear after logout")
    }

    func testNetworkErrorHandling() throws {
        let app = XCUIApplication()
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)

        // Reconfigure to an unreachable port via settings UI to test real network failure handling
        let openServerBtn = app.buttons["open_server_settings_button"]
        XCTAssertTrue(openServerBtn.waitForExistence(timeout: 5.0))
        openServerBtn.tap()

        let settingsScreen = app.otherElements["settings_screen"]
        XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5.0))

        let serverInput = app.textFields["settings_input_server_url"]
        XCTAssertTrue(serverInput.waitForExistence(timeout: 3.0))
        serverInput.tap()
        let currentVal = serverInput.value as? String ?? ""
        let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: currentVal.count)
        serverInput.typeText(deleteString)
        serverInput.typeText("http://localhost:59999")

        let saveBtn = app.buttons["settings_button_save_server_url"]
        XCTAssertTrue(saveBtn.waitForExistence(timeout: 3.0))
        saveBtn.tap()

        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0))

        let emailField = app.textFields["auth_email_field"]
        emailField.tap()
        emailField.typeText("invalid@example.com")

        let passwordField = app.secureTextFields["auth_password_field"]
        passwordField.tap()
        passwordField.typeText("Secret123!")

        let loginButton = app.buttons["auth_login_button"]
        loginButton.tap()

        // Verify error banner is displayed
        let errorBanner = app.otherElements["auth_error_banner"]
        XCTAssertTrue(errorBanner.waitForExistence(timeout: 10.0), "Error banner should be displayed on network failure")
    }

    func testFirstLaunchEndpointSetupAndValidation() throws {
        let app = XCUIApplication()
        // Clear saved endpoint preference on launch to simulate clean first launch
        app.launchArguments = ["-mensetsu_server_endpoint", ""]
        app.launch()

        // 1. Verify server setup screen is presented on first launch
        let setupScreen = app.otherElements["server_setup_screen"]
        XCTAssertTrue(setupScreen.waitForExistence(timeout: 5.0), "Server setup screen should appear when no endpoint is configured")

        let inputField = app.textFields["input_server_url"]
        XCTAssertTrue(inputField.waitForExistence(timeout: 3.0))
        let connectBtn = app.buttons["button_save_server_url"]

        // 2. Test validation: try entering insecure remote URL
        inputField.tap()
        inputField.typeText("http://remote.example.com")
        connectBtn.tap()

        let errorMsg = app.staticTexts["server_url_error_message"]
        XCTAssertTrue(errorMsg.waitForExistence(timeout: 3.0), "Error should appear for insecure remote HTTP URL")

        // 3. Enter a valid loopback dev URL (strips trailing slash)
        inputField.tap()
        let currentVal = inputField.value as? String ?? ""
        let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: currentVal.count)
        inputField.typeText(deleteString)
        inputField.typeText(targetApiUrl + "/")
        connectBtn.tap()

        // 4. Verify transition to auth screen once valid URL is saved
        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0), "Auth screen should appear after saving valid server URL")

        // 5. Terminate and relaunch cleanly without launch arguments to verify persistence
        app.terminate()
        let relaunchedApp = XCUIApplication()
        relaunchedApp.launch()
        let relaunchedAuth = relaunchedApp.otherElements["auth_screen"]
        XCTAssertTrue(relaunchedAuth.waitForExistence(timeout: 5.0), "Saved endpoint must persist across app relaunch")
    }

    func testUpdateServerFromPreAuthSettings() throws {
        let app = XCUIApplication()
        app.launch()
        ensureConfiguredAndOnAuthView(for: app)

        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0))

        // Open pre-auth server settings
        let openServerBtn = app.buttons["open_server_settings_button"]
        XCTAssertTrue(openServerBtn.waitForExistence(timeout: 3.0), "Pre-auth server settings button must exist on auth screen")
        openServerBtn.tap()

        let settingsScreen = app.otherElements["settings_screen"]
        XCTAssertTrue(settingsScreen.waitForExistence(timeout: 5.0), "Settings screen should appear")

        let serverInput = app.textFields["settings_input_server_url"]
        XCTAssertTrue(serverInput.waitForExistence(timeout: 3.0), "Server URL input must be visible on settings screen")

        // Change server to another valid URL
        serverInput.tap()
        let currentVal = serverInput.value as? String ?? ""
        let deleteString = String(repeating: XCUIKeyboardKey.delete.rawValue, count: currentVal.count)
        serverInput.typeText(deleteString)
        serverInput.typeText("http://localhost:8081")

        let saveBtn = app.buttons["settings_button_save_server_url"]
        XCTAssertTrue(saveBtn.waitForExistence(timeout: 3.0), "Save server URL button must exist")
        saveBtn.tap()

        // Verify settings dismisses and auth screen is visible with clean session
        XCTAssertTrue(authScreen.waitForExistence(timeout: 5.0), "Auth screen must remain visible in unauthenticated state after server change")

        app.terminate()
        let relaunched = XCUIApplication()
        relaunched.launch()
        let changeServer = relaunched.buttons["open_server_settings_button"]
        XCTAssertTrue(changeServer.waitForExistence(timeout: 5.0))
        changeServer.tap()
        XCTAssertEqual(relaunched.textFields["settings_input_server_url"].value as? String, "http://localhost:8081")

        // Restore target API URL so subsequent test runs have expected endpoint
        let restoreInput = relaunched.textFields["settings_input_server_url"]
        restoreInput.tap()
        let restoreVal = restoreInput.value as? String ?? ""
        let restoreDelete = String(repeating: XCUIKeyboardKey.delete.rawValue, count: restoreVal.count)
        restoreInput.typeText(restoreDelete)
        restoreInput.typeText(targetApiUrl)
        let restoreSave = relaunched.buttons["settings_button_save_server_url"]
        restoreSave.tap()
        let doneBtn = relaunched.buttons["settings_done_button"]
        if doneBtn.waitForExistence(timeout: 2.0) {
            doneBtn.tap()
        }
    }
}
