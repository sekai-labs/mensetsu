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

@RunWith(AndroidJUnit4::class)
class MensetsuSmokeUiTest {

    @get:Rule
    val composeTestRule = createAndroidComposeRule<MainActivity>()

    private val targetBaseUrl: String
        get() {
            val args = InstrumentationRegistry.getArguments()
            return args.getString("API_BASE_URL")
                ?: "http://10.0.2.2:3000"
        }

    @Before
    fun setup() {
        val app = composeTestRule.activity.application as MensetsuApplication
        app.networkManager.clearSession()
    }

    private fun ensureConnected(endpointUrl: String = targetBaseUrl) {
        val app = composeTestRule.activity.application as MensetsuApplication
        if (composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_SCREEN).fetchSemanticsNodes().isNotEmpty()) {
            if (app.networkManager.getBaseUrl() != endpointUrl) {
                composeTestRule.onNodeWithTag(AppTestTags.BUTTON_SETTINGS).performClick()
                composeTestRule.waitUntil(timeoutMillis = 5_000) {
                    composeTestRule.onAllNodesWithTag(AppTestTags.SERVER_SETUP_SCREEN).fetchSemanticsNodes().isNotEmpty()
                }
            }
        }
        if (composeTestRule.onAllNodesWithTag(AppTestTags.SERVER_SETUP_SCREEN).fetchSemanticsNodes().isNotEmpty()) {
            composeTestRule.onNodeWithTag(AppTestTags.INPUT_SERVER_URL).performTextReplacement(endpointUrl)
            composeTestRule.onNodeWithTag(AppTestTags.BUTTON_SAVE_SERVER_URL).performClick()
            composeTestRule.waitUntil(timeoutMillis = 10_000) {
                composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_SCREEN).fetchSemanticsNodes().isNotEmpty()
            }
        }
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
        ensureConnected()

        composeTestRule.onNodeWithTag(AppTestTags.AUTH_TAB_REGISTER).performClick()

        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_NAME).performTextInput(testName)
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).performTextInput(testEmail)
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_PASSWORD).performTextInput(testPassword)

        composeTestRule.onNodeWithTag(AppTestTags.AUTH_BUTTON_SUBMIT).performClick()

        composeTestRule.waitUntil(timeoutMillis = 10_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.WORKSPACE_LIST_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }

        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_NEW_WORKSPACE).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DIALOG_CREATE_WORKSPACE).assertIsDisplayed()

        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_COMPANY).performTextInput(targetCompany)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_ROLE).performTextInput(targetRole)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_RESUME_TEXT).performTextInput(testResumeText)
        composeTestRule.onNodeWithTag(AppTestTags.INPUT_CREATE_JOB_DESC).performTextInput(testJobDesc)

        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_CREATE_SUBMIT).performClick()

        composeTestRule.waitUntil(timeoutMillis = 15_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.WORKSPACE_DETAIL_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_OVERVIEW).assertIsDisplayed()

        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_MATCH).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_COMPANY).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_LEARNING).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_QUESTIONS).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.DETAIL_TAB_SETTINGS).performClick()

        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_DETAIL_BACK).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.WORKSPACE_LIST_SCREEN).assertIsDisplayed()
        composeTestRule.onNodeWithText(targetCompany).assertIsDisplayed()
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


        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_ALL).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_READY).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_PROCESSING).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.FILTER_TAB_ALL).performClick()

        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_LOGOUT).performClick()

        composeTestRule.waitUntil(timeoutMillis = 10_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).assertIsDisplayed()
    }

    @Test
    fun testNetworkErrorStateSmoke() {
        val unreachableUrl = "http://127.0.0.1:59999"
        ensureConnected(unreachableUrl)

        try {
            composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_EMAIL).performTextInput("error_test@example.com")
            composeTestRule.onNodeWithTag(AppTestTags.AUTH_INPUT_PASSWORD).performTextInput("password123")
            composeTestRule.onNodeWithTag(AppTestTags.AUTH_BUTTON_SUBMIT).performClick()

            composeTestRule.waitUntil(timeoutMillis = 10_000) {
                composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_ERROR_TEXT).fetchSemanticsNodes().isNotEmpty()
            }
            composeTestRule.onNodeWithTag(AppTestTags.AUTH_ERROR_TEXT).assertIsDisplayed()
        } finally {
            ensureConnected(targetBaseUrl)
        }
    }

    @Test
    fun testEndpointValidationAndFirstLaunchFlow() {
        val app = composeTestRule.activity.application as MensetsuApplication
        app.networkManager.setBaseUrl(null)
        app.networkManager.clearSession()

        composeTestRule.activityRule.scenario.recreate()

        composeTestRule.waitUntil(timeoutMillis = 5_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.SERVER_SETUP_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.SERVER_SETUP_SCREEN).assertIsDisplayed()

        composeTestRule.onNodeWithTag(AppTestTags.INPUT_SERVER_URL).performTextReplacement("http://api.insecure.com")
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_SAVE_SERVER_URL).performClick()
        composeTestRule.onNodeWithTag(AppTestTags.SERVER_URL_ERROR_MESSAGE).assertIsDisplayed()

        composeTestRule.onNodeWithTag(AppTestTags.INPUT_SERVER_URL).performTextReplacement(targetBaseUrl)
        composeTestRule.onNodeWithTag(AppTestTags.BUTTON_SAVE_SERVER_URL).performClick()

        composeTestRule.waitUntil(timeoutMillis = 5_000) {
            composeTestRule.onAllNodesWithTag(AppTestTags.AUTH_SCREEN).fetchSemanticsNodes().isNotEmpty()
        }
        composeTestRule.onNodeWithTag(AppTestTags.AUTH_SCREEN).assertIsDisplayed()
    }
}
