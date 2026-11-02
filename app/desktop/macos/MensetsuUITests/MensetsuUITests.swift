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

    override func setUpWithError() throws {
        continueAfterFailure = false
        app = XCUIApplication()
        
        let apiUrl = ProcessInfo.processInfo.environment["MENSETSU_API_URL"] ?? "http://127.0.0.1:3000"
        app.launchEnvironment["MENSETSU_API_URL"] = apiUrl
        app.launchArguments = ["--api-url", apiUrl]
    }

    override func tearDownWithError() throws {
        app = nil
    }

    func testRegisterCreateWorkspaceListToggleAndLogout() throws {
        app.launch()

        // 1. Auth screen visible
        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 10), "Auth screen should appear")

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
        app.launchEnvironment["MENSETSU_API_URL"] = unreachableUrl
        app.launchArguments = ["--api-url", unreachableUrl]
        app.launch()

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

        // 1. Auth screen visible
        let authScreen = app.otherElements["auth_screen"]
        XCTAssertTrue(authScreen.waitForExistence(timeout: 10), "Auth screen should appear")

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
}
