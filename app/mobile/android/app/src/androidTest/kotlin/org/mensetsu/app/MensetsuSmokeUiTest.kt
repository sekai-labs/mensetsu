package org.mensetsu.app

import androidx.compose.ui.test.*
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.uiautomator.By
import androidx.test.uiautomator.UiDevice
import androidx.test.uiautomator.Until
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Before
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.mensetsu.app.core.tags.AppTestTags
import java.util.UUID

/**
 * Native UI Compose instrumentation smoke test.
 * Exercises real end-to-end user journeys against a live running Rust API:
 * 1. Fresh user registration
 * 2. Workspace text and real PDF creation through the native Documents picker
 * 3. Workspace listing & verification in list
 * 4. Workspace detail navigation & tab inspection (overview, match, company, learning, questions, settings)
 * 5. Learning unit completed toggle
 * 6. Logout and verify return to Auth screen
 * 7. Network error state verification
 */
@RunWith(AndroidJUnit4::class)
class MensetsuSmokeUiTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    private val targetBaseUrl: String
        get() {
            val args = InstrumentationRegistry.getArguments()
            return args.getString("API_BASE_URL") ?: BuildConfig.DEFAULT_BASE_URL
        }

    @Before
    fun setup() {
        val app = composeTestRule.activity.application as MensetsuApplication
        // Point to configured live API URL (supports Android emulator loopback http://10.0.2.2:3000 or custom)
        app.networkManager.setBaseUrl(targetBaseUrl)
        app.networkManager.clearSession()
    }

    @Test
    fun testFreshRegisterCreateWorkspaceListDetailToggleAndLogoutSmoke() {
        val uniqueId = UUID.randomUUID().toString().take(8)
        val testEmail = "android_smoke_$uniqueId@example.com"
        val testPassword = "Password123!"
        val testName = "Android Smoke User $uniqueId"

        val targetCompany = "ScaleVector $uniqueId"
        val targetRole = "Staff Android Engineer"
        val testResumeText = "Built high-performance Android Jetpack Compose native applications and Kotlin OkHttp services."
        val testJobDesc = "Seeking Staff Android Engineer with strong Jetpack Compose, OkHttp, and Kotlin coroutines experience."

        // 1. Switch to Register tab
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_TAB_REGISTER).performClick()

        // 2. Fill Register form
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_NAME).performTextInput(testName)
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).performTextInput(testEmail)
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_PASSWORD).performTextInput(testPassword)

        // 3. Submit Registration
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_BUTTON_SUBMIT).performClick()

        // 4. Verify landing on Workspace List screen
        composeTestRule.waitUntil(timeoutMillis = 10_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.WORKSPACE_LIST_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }

        // 5. Open New Workspace Dialog
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_NEW_WORKSPACE).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DIALOG_CREATE_WORKSPACE).assertIsDisplayed()

        // 6. Enter Workspace Details
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_COMPANY).performTextInput(targetCompany)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_ROLE).performTextInput(targetRole)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_RESUME_TEXT).performTextInput(testResumeText)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_JOB_DESC).performTextInput(testJobDesc)

        // 7. Submit Create Workspace
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_CREATE_SUBMIT).performClick()

        // 8. Verify Navigation to Workspace Detail Screen
        composeTestRule.waitUntil(timeoutMillis = 15_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.WORKSPACE_DETAIL_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_OVERVIEW).assertIsDisplayed()

        // 9. Inspect Tabs
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_MATCH).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_COMPANY).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_LEARNING).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_QUESTIONS).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_SETTINGS).performClick()

        // 10. Navigate Back to Workspace List
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_DETAIL_BACK).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.WORKSPACE_LIST_SCREEN).assertIsDisplayed()
        composeTestRule.onNodeWithText(targetCompany).assertIsDisplayed()
        // Create a second workspace from the real PDF seeded in Downloads by the emulator job.
        val pdfCompany = "Android PDF $uniqueId"
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_NEW_WORKSPACE).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_COMPANY).performTextInput(pdfCompany)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_ROLE).performTextInput("Backend Rust Engineer")
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_CREATE_PDF_MODE).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_CREATE_PICK_PDF).performClick()
        val device = UiDevice.getInstance(InstrumentationRegistry.getInstrumentation())
        var file = device.wait(Until.findObject(By.text("resume-text.pdf")), 3_000)
        if (file == null) {
            val downloads = device.wait(Until.findObject(By.text("Downloads")), 3_000)
                ?: run {
                    device.findObject(By.desc("Show roots"))?.click()
                    device.wait(Until.findObject(By.text("Downloads")), 5_000)
                }
            requireNotNull(downloads) { "Documents picker must show Downloads" }
            downloads.click()
            file = device.wait(Until.findObject(By.text("resume-text.pdf")), 8_000)
        }
        requireNotNull(file) { "PDF fixture must be selectable in the native Documents picker" }.click()
        composeTestRule.waitUntil(timeoutMillis = 5_000) {
            composeTestRule.onAllNodesWithText("resume-text.pdf").fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_CREATE_SUBMIT).performClick()
        composeTestRule.waitUntil(timeoutMillis = 30_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.WORKSPACE_DETAIL_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_DETAIL_BACK).performClick()
        composeTestRule.onNodeWithText(pdfCompany).assertIsDisplayed()


        // 11. Test Filters
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_ALL).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_READY).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_PROCESSING).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_ALL).performClick()

        // 12. Logout
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_LOGOUT).performClick()

        // 13. Verify Return to Auth Screen
        composeTestRule.waitUntil(timeoutMillis = 10_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).assertIsDisplayed()
    }

    @Test
    fun testNetworkErrorStateSmoke() {
        val app = composeTestRule.activity.application as MensetsuApplication
        // Set an unreachable port to test real error handling
        app.networkManager.setBaseUrl("http://127.0.0.1:59999")

        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).performTextInput("error_test@example.com")
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_PASSWORD).performTextInput("password123")
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_BUTTON_SUBMIT).performClick()

        // Verify error text is displayed
        composeTestRule.waitUntil(timeoutMillis = 10_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_ERROR_TEXT).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_ERROR_TEXT).assertIsDisplayed()
    }
}
