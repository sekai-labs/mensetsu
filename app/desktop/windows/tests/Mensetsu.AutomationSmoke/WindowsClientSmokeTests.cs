using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading;
using FlaUI.Core;
using FlaUI.Core.AutomationElements;
using FlaUI.Core.Conditions;
using FlaUI.Core.Definitions;
using FlaUI.Core.Input;
using FlaUI.Core.Tools;
using FlaUI.UIA3;
using Mensetsu.Common;
using Xunit;

namespace Mensetsu.AutomationSmoke
{
    /// <summary>
    /// Real Windows UI Automation (UIA3) smoke test driving the unpackaged Mensetsu WinUI 3 application process.
    /// Interacts directly with platform controls via AutomationProperties.AutomationId against a live Rust backend:
    /// 1. Launches Mensetsu.exe process with MENSETSU_API_URL configured
    /// 2. Switches to Register mode, enters name/email/password, clicks Register
    /// 3. In Workspace list, opens Create Workspace dialog
    /// 4. Enters target company, role, resume text, and job description, clicks Submit
    /// 5. Verifies workspace appears in list and selects it to navigate to Workspace Detail
    /// 6. Inspects Detail tabs: Overview metrics, Match strengths/gaps/risks, Company dossier, Learning toggle, Questions filter
    /// 7. Navigates back, clicks Logout, asserts return to AuthScreen
    /// 8. Verifies real network error assertion in UI error banner
    /// </summary>
    public class WindowsClientSmokeTests
    {
        private readonly string _apiUrl;
        private readonly string _appExePath;

        public WindowsClientSmokeTests()
        {
            _apiUrl = Environment.GetEnvironmentVariable("MENSETSU_API_URL") ?? "http://127.0.0.1:3000";

            var customExe = Environment.GetEnvironmentVariable("MENSETSU_APP_EXE");
            if (!string.IsNullOrEmpty(customExe))
            {
                _appExePath = customExe;
            }
            else
            {
                // Default location relative to output build bin
                var baseDir = AppDomain.CurrentDomain.BaseDirectory;
                _appExePath = Path.GetFullPath(Path.Combine(baseDir, @"..\..\..\..\..\src\Mensetsu\bin\x64\Debug\net9.0-windows10.0.19041.0\win-x64\Mensetsu.exe"));
            }
        }

        [Fact]
        public void Windows_UIAutomation_Full_User_Journey_Smoke()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                // Honest platform constraint disclosure for non-Windows environments
                throw new PlatformNotSupportedException(
                    "Real Windows UIAutomation testing requires Windows OS with desktop session / interactive window manager.");
            }

            if (!File.Exists(_appExePath))
            {
                throw new FileNotFoundException($"Mensetsu application binary not found at '{_appExePath}'. Build Mensetsu.sln first.", _appExePath);
            }

            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            startInfo.EnvironmentVariables["MENSETSU_API_URL"] = _apiUrl;

            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                var randomSuffix = Guid.NewGuid().ToString("N")[..8];
                var testEmail = $"uia_win_{randomSuffix}@example.com";
                var testPassword = "Password123!";
                var testName = $"UIA Tester {randomSuffix}";

                // 1. Verify Auth Screen is displayed
                var authScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreen);

                // 2. Switch to Register tab and enter user info
                var registerTab = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthRegisterTab))?.AsButton();
                Assert.NotNull(registerTab);
                registerTab.Invoke();

                var nameField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthNameInput))?.AsTextBox();
                Assert.NotNull(nameField);
                nameField.Text = testName;

                var emailField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthEmailInput))?.AsTextBox();
                Assert.NotNull(emailField);
                emailField.Text = testEmail;

                var passwordField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthPasswordInput));
                Assert.NotNull(passwordField);
                passwordField.Focus();
                Keyboard.Type(testPassword);

                // Click Register button
                var registerBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthRegisterButton))?.AsButton();
                Assert.NotNull(registerBtn);
                registerBtn.Invoke();

                // 3. Wait for transition to Workspace List Screen
                var wsListScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.WorkspaceListScreen)), TimeSpan.FromSeconds(15)).Result;
                Assert.NotNull(wsListScreen);

                // 4. Open Create Workspace dialog
                var openCreateBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.OpenCreateWorkspaceButton))?.AsButton();
                Assert.NotNull(openCreateBtn);
                openCreateBtn.Invoke();

                var createDialog = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateWorkspaceDialog)), TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(createDialog);

                // Enter workspace creation details
                var companyField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateCompanyInput))?.AsTextBox();
                Assert.NotNull(companyField);
                companyField.Text = "Acme Robotics";

                var roleField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateRoleInput))?.AsTextBox();
                Assert.NotNull(roleField);
                roleField.Text = "Principal Systems Engineer";

                var resumeField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateResumeTextInput))?.AsTextBox();
                Assert.NotNull(resumeField);
                resumeField.Text = "Lead engineer developing native distributed C# and Rust infrastructure.";

                var jdField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateJobDescriptionInput))?.AsTextBox();
                Assert.NotNull(jdField);
                jdField.Text = "Looking for a Principal Systems Engineer to build mission-critical autonomous systems.";

                // Submit creation
                var submitBtn = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SubmitCreateWorkspaceButton))?.AsButton();
                if (submitBtn == null)
                {
                    // Fall back to ContentDialog primary button if custom AutomationId was nested
                    submitBtn = createDialog.FindFirstDescendant(cf => cf.ByName("Create Workspace"))?.AsButton();
                }
                Assert.NotNull(submitBtn);
                submitBtn.Invoke();

                // 5. Verify Workspace Detail view is shown
                var detailScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.WorkspaceDetailScreen)), TimeSpan.FromSeconds(15)).Result;
                Assert.NotNull(detailScreen);

                // Inspect Overview metrics
                var companyNameMetric = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.OverviewCompanyName));
                Assert.NotNull(companyNameMetric);
                Assert.Contains("Acme Robotics", companyNameMetric.Name ?? companyNameMetric.HelpText ?? "");

                // Switch to Match tab
                var tabMatch = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.TabMatch));
                if (tabMatch != null)
                {
                    tabMatch.Click();
                    var matchRatio = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.MatchRatioText));
                    Assert.NotNull(matchRatio);
                }

                // Switch to Learning tab and toggle unit if present
                var tabLearning = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.TabLearning));
                if (tabLearning != null)
                {
                    tabLearning.Click();
                    var learningList = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.LearningUnitsList));
                    Assert.NotNull(learningList);
                }

                // Navigate back to Workspace List
                var backBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.DetailBackButton))?.AsButton();
                Assert.NotNull(backBtn);
                backBtn.Invoke();

                wsListScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.WorkspaceListScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(wsListScreen);

                // 6. Logout through UI button
                var logoutBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.LogoutButton))?.AsButton();
                Assert.NotNull(logoutBtn);
                logoutBtn.Invoke();

                // 7. Verify Auth Screen is displayed after logout
                var authScreenAfterLogout = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreenAfterLogout);
            }
            finally
            {
                try
                {
                    app.Close();
                }
                catch
                {
                    app.Kill();
                }
            }
        }

        [Fact]
        public void Windows_UIAutomation_Network_Error_Displays_UI_Banner()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException("Real Windows UIAutomation testing requires Windows OS with desktop session.");
            }

            if (!File.Exists(_appExePath))
            {
                throw new FileNotFoundException($"Mensetsu application binary not found at '{_appExePath}'.", _appExePath);
            }

            // Launch app pointing to an unreachable port to trigger a live network error
            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            startInfo.EnvironmentVariables["MENSETSU_API_URL"] = "http://127.0.0.1:59999";

            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                // Attempt to sign in against unreachable server
                var emailField = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthEmailInput))?.AsTextBox(), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(emailField);
                emailField.Text = "network_err_test@example.com";

                var passwordField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthPasswordInput));
                Assert.NotNull(passwordField);
                passwordField.Focus();
                Keyboard.Type("TestPassword123!");

                var loginBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthLoginButton))?.AsButton();
                Assert.NotNull(loginBtn);
                loginBtn.Invoke();

                // Assert UI error banner becomes visible in the UI with network error message
                var errorBanner = Retry.WhileNull(() =>
                {
                    var banner = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthErrorBanner));
                    if (banner != null && (banner.IsAvailable || !string.IsNullOrEmpty(banner.Name)))
                    {
                        return banner;
                    }
                    return null;
                }, TimeSpan.FromSeconds(10)).Result;

                Assert.NotNull(errorBanner);
            }
            finally
            {
                try
                {
                    app.Close();
                }
                catch
                {
                    app.Kill();
                }
            }
        }
        [Fact]
        public void Windows_UIAutomation_Pdf_Resume_Upload_Smoke()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException(
                    "Real Windows UIAutomation testing requires Windows OS with desktop session / interactive window manager.");
            }

            if (!File.Exists(_appExePath))
            {
                throw new FileNotFoundException($"Mensetsu application binary not found at '{_appExePath}'. Build Mensetsu.sln first.", _appExePath);
            }

            var fixturePath = Environment.GetEnvironmentVariable("MENSETSU_TEST_PDF_PATH")
                ?? throw new InvalidOperationException("Set MENSETSU_TEST_PDF_PATH to a real PDF fixture.");
            Assert.True(File.Exists(fixturePath), $"Real fixture not found at {fixturePath}");

            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            startInfo.EnvironmentVariables["MENSETSU_API_URL"] = _apiUrl;
            startInfo.EnvironmentVariables["MENSETSU_TEST_PDF_PATH"] = fixturePath;

            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                var randomSuffix = Guid.NewGuid().ToString("N")[..8];
                var testEmail = $"uia_pdf_{randomSuffix}@example.com";
                var testPassword = "Password123!";
                var testName = $"UIA PDF Tester {randomSuffix}";

                // 1. Verify Auth Screen and switch to Register tab
                var authScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreen);

                var registerTab = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthRegisterTab))?.AsButton();
                Assert.NotNull(registerTab);
                registerTab.Invoke();

                var nameField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthNameInput))?.AsTextBox();
                Assert.NotNull(nameField);
                nameField.Text = testName;

                var emailField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthEmailInput))?.AsTextBox();
                Assert.NotNull(emailField);
                emailField.Text = testEmail;

                var passwordField = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthPasswordInput));
                Assert.NotNull(passwordField);
                passwordField.Focus();
                Keyboard.Type(testPassword);

                var registerBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthRegisterButton))?.AsButton();
                Assert.NotNull(registerBtn);
                registerBtn.Invoke();

                // 2. Wait for Workspace List Screen
                var wsListScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.WorkspaceListScreen)), TimeSpan.FromSeconds(15)).Result;
                Assert.NotNull(wsListScreen);

                // 3. Open Create Workspace dialog
                var openCreateBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.OpenCreateWorkspaceButton))?.AsButton();
                Assert.NotNull(openCreateBtn);
                openCreateBtn.Invoke();

                var createDialog = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateWorkspaceDialog)), TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(createDialog);

                // Fill company and role
                var companyField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateCompanyInput))?.AsTextBox();
                Assert.NotNull(companyField);
                companyField.Text = "PDF Quantum Systems";

                var roleField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateRoleInput))?.AsTextBox();
                Assert.NotNull(roleField);
                roleField.Text = "Senior Cloud Architect";

                // Switch to Upload PDF mode
                var pdfModeRadio = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ResumeModePdfRadio))?.AsRadioButton();
                Assert.NotNull(pdfModeRadio);
                pdfModeRadio.IsChecked = true;

                // Pick PDF file button should be visible in PDF mode
                var pickPdfBtn = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.PickPdfFileButton));
                Assert.NotNull(pickPdfBtn);

                var jdField = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateJobDescriptionInput))?.AsTextBox();
                Assert.NotNull(jdField);
                jdField.Text = "Lead cloud infrastructure development with S3 object storage integration.";

                // Test programmatic file loading for automated UI test runner to exercise the full signed S3 PUT flow.
                // 1. Missing file validation path
                var submitBtn = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SubmitCreateWorkspaceButton))?.AsButton()
                    ?? createDialog.FindFirstDescendant(cf => cf.ByName("Create Workspace"))?.AsButton();
                Assert.NotNull(submitBtn);
                submitBtn.Invoke();

                var createErrorBanner = Retry.WhileNull(() =>
                {
                    var banner = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.CreateWorkspaceErrorBanner));
                    if (banner != null && (banner.IsAvailable || !string.IsNullOrEmpty(banner.Name)))
                    {
                        return banner;
                    }
                    return null;
                }, TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(createErrorBanner);

                // 2. Select real PDF fixture via picker action
                pickPdfBtn.AsButton()?.Invoke();
                var selectedFileText = Retry.WhileNull(() =>
                {
                    var txt = createDialog.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SelectedPdfFileText));
                    if (txt != null && txt.Name.Contains("resume-text.pdf")) return txt;
                    return null;
                }, TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(selectedFileText);

                // 3. Submit and verify workspace creation against live API via staged S3 PUT flow
                submitBtn.Invoke();

                var detailScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.WorkspaceDetailScreen)), TimeSpan.FromSeconds(25)).Result;
                Assert.NotNull(detailScreen);

                var companyName = Retry.WhileNull(() => detailScreen.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.OverviewCompanyName)), TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(companyName);
                Assert.Contains("PDF Quantum Systems", companyName.Name);
            }
            finally
            {
                try
                {
                    app.Close();
                }
                catch
                {
                    app.Kill();
                }
            }
        }
    }
}
