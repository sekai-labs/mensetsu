//
// MensetsuUITests.swift
// MensetsuUITests
//

import XCTest

final class MensetsuUITests: XCTestCase {

    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    func testRealAuthCreateListDetailLogoutFlow() throws {
        let app = XCUIApplication()
        let baseUrl = ProcessInfo.processInfo.environment["MENSETSU_API_BASE_URL"] ?? "http://localhost:8080"
        app.launchEnvironment["MENSETSU_API_BASE_URL"] = baseUrl
        app.launch()

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
        // Point to an invalid unreachable port
        app.launchEnvironment["MENSETSU_API_BASE_URL"] = "http://localhost:59999"
        app.launch()

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
}
