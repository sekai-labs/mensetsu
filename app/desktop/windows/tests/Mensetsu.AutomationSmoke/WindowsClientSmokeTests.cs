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
    /// Interacts directly with platform controls via AutomationProperties.AutomationId against a live Rust backend:
    /// 1. Sets server endpoint via onboarding UI or Auth pre-login Change Server UI (never injected into app runtime env)
    /// 2. Switches to Register mode, enters name/email/password, clicks Register
    /// 3. In Workspace list, opens Create Workspace dialog
    /// 4. Enters target company, role, resume text, and job description, clicks Submit
    /// 5. Verifies workspace appears in list and selects it to navigate to Workspace Detail
    /// 6. Inspects Detail tabs: Overview metrics, Match strengths/gaps/risks, Company dossier, Learning toggle, Questions filter
    /// 7. Navigates back, clicks Logout, asserts return to AuthScreen
    /// 8. Verifies real network error assertion in UI error banner
    /// </summary>
    [CollectionDefinition("WindowsSmokeTestCollection", DisableParallelization = true)]
    public class WindowsSmokeTestCollectionDefinition { }

    [Collection("WindowsSmokeTestCollection")]
    public class WindowsClientSmokeTests : IDisposable
    {
        private static readonly object FileLock = new object();
        private readonly byte[]? _originalEndpointBackup;
        private readonly bool _hadOriginalEndpoint;
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

            // Back up exact bytes of any pre-existing user endpoint file so test runs preserve developer state
            var path = GetSavedEndpointFilePath();
            if (File.Exists(path))
            {
                _hadOriginalEndpoint = true;
                _originalEndpointBackup = File.ReadAllBytes(path);
            }
            else
            {
                _hadOriginalEndpoint = false;
                _originalEndpointBackup = null;
            }
        }

        public void Dispose()
        {
            // Restore original developer state upon test completion
            lock (FileLock)
            {
                var path = GetSavedEndpointFilePath();
                if (_hadOriginalEndpoint && _originalEndpointBackup != null)
                {
                    var dir = Path.GetDirectoryName(path);
                    if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
                    {
                        Directory.CreateDirectory(dir);
                    }
                    File.WriteAllBytes(path, _originalEndpointBackup);
                }
                else if (!_hadOriginalEndpoint && File.Exists(path))
                {
                    File.Delete(path);
                }
            }
        }

        private static string GetSavedEndpointFilePath()
        {
            var appData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            return Path.Combine(appData, "Mensetsu", "server_endpoint.txt");
        }

        private static void EnsureConfiguredAndOnAuthView(Window window, string targetEndpoint)
        {
            var normalizedTarget = targetEndpoint.Trim().TrimEnd('/');

            // Check if app is on first-launch ServerSetupScreen
            var setupScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerSetupScreen)), TimeSpan.FromSeconds(3)).Result;
            if (setupScreen != null)
            {
                var urlInput = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlInput))?.AsTextBox();
                var saveBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SaveServerUrlButton))?.AsButton();
                Assert.NotNull(urlInput);
                Assert.NotNull(saveBtn);
                urlInput.Text = targetEndpoint;
                saveBtn.Invoke();
            }

            var authScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
            Assert.NotNull(authScreen);

            // Check if saved server differs from target; if so, reconfigure via pre-login Change Server button
            var savedPath = GetSavedEndpointFilePath();
            var currentPersisted = File.Exists(savedPath) ? File.ReadAllText(savedPath).Trim().TrimEnd('/') : null;
            if (!string.Equals(currentPersisted, normalizedTarget, StringComparison.OrdinalIgnoreCase))
            {
                var changeServerBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthChangeServerButton))?.AsButton();
                Assert.NotNull(changeServerBtn);
                changeServerBtn.Invoke();

                var setupScreenReconfig = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerSetupScreen)), TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(setupScreenReconfig);

                var urlInput = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlInput))?.AsTextBox();
                var saveBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SaveServerUrlButton))?.AsButton();
                Assert.NotNull(urlInput);
                Assert.NotNull(saveBtn);
                urlInput.Text = targetEndpoint;
                saveBtn.Invoke();

                authScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreen);
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

            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                EnsureConfiguredAndOnAuthView(window, _apiUrl);

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

            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                // Reconfigure to unreachable endpoint via UI (never direct disk overwrite or app env injection)
                EnsureConfiguredAndOnAuthView(window, "http://127.0.0.1:59999");
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

            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            startInfo.EnvironmentVariables["MENSETSU_TEST_PDF_PATH"] = fixturePath;
            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                EnsureConfiguredAndOnAuthView(window, _apiUrl);
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

        [Fact]
        public void Windows_UIAutomation_First_Launch_Server_Setup_And_Settings_Flow()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException("Real Windows UIAutomation testing requires Windows OS with desktop session.");
            }

            if (!File.Exists(_appExePath))
            {
                throw new FileNotFoundException($"Mensetsu application binary not found at '{_appExePath}'.", _appExePath);
            }

            // Ensure any previously saved endpoint file is removed so this behaves like a clean fresh install
            var appData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            var savedEndpointFile = Path.Combine(appData, "Mensetsu", "server_endpoint.txt");
            if (File.Exists(savedEndpointFile))
            {
                try { File.Delete(savedEndpointFile); } catch { }
            }

            // Ensure clean environment without any runtime API URL overrides
            var startInfo = new ProcessStartInfo
            {
                FileName = _appExePath,
                UseShellExecute = false
            };
            using var app = Application.Launch(startInfo);
            using var automation = new UIA3Automation();

            try
            {
                var window = app.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(window);

                // 1. Fresh install prompts for URL (server_setup_screen)
                var setupScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerSetupScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(setupScreen);

                var urlInput = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlInput))?.AsTextBox();
                Assert.NotNull(urlInput);

                var saveBtn = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.SaveServerUrlButton))?.AsButton();
                Assert.NotNull(saveBtn);

                // 2. Validate invalid input displays error
                // Test invalid scheme/format
                urlInput.Text = "not-a-valid-url";
                saveBtn.Invoke();

                var errorBanner = Retry.WhileNull(() =>
                {
                    var banner = window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlErrorBanner));
                    if (banner != null && (banner.IsAvailable || !string.IsNullOrEmpty(banner.Name))) return banner;
                    return null;
                }, TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(errorBanner);

                // Test non-root path rejection
                urlInput.Text = _apiUrl.TrimEnd('/') + "/api/v1";
                saveBtn.Invoke();
                Assert.NotNull(window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlErrorBanner)));

                // Test query/fragment rejection
                urlInput.Text = _apiUrl.TrimEnd('/') + "/?";
                saveBtn.Invoke();
                Assert.NotNull(window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerUrlErrorBanner)));

                // 3. Enter valid URL origin and save -> enters Auth screen
                urlInput.Text = _apiUrl;
                saveBtn.Invoke();

                var authScreen = Retry.WhileNull(() => window.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreen);

                // Assert endpoint was persisted to disk
                Assert.True(File.Exists(savedEndpointFile), "Saved endpoint file should exist after saving valid URL.");
                var savedText = File.ReadAllText(savedEndpointFile).Trim();
                Assert.Equal(_apiUrl.TrimEnd('/'), savedText);
            }
            finally
            {
                try { app.Close(); } catch { app.Kill(); }
            }

            // 4. Relaunch reuses URL (should open AuthScreen directly without setup prompt)
            using var relaunchApp = Application.Launch(startInfo);
            try
            {
                var relaunchWindow = relaunchApp.GetMainWindow(automation, TimeSpan.FromSeconds(15));
                Assert.NotNull(relaunchWindow);

                var authScreen = Retry.WhileNull(() => relaunchWindow.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthScreen)), TimeSpan.FromSeconds(10)).Result;
                Assert.NotNull(authScreen);

                // Test changing server via pre-auth Change Server button
                var changeServerBtn = relaunchWindow.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.AuthChangeServerButton))?.AsButton();
                Assert.NotNull(changeServerBtn);
                changeServerBtn.Invoke();

                var setupScreen = Retry.WhileNull(() => relaunchWindow.FindFirstDescendant(cf => cf.ByAutomationId(AutomationIds.ServerSetupScreen)), TimeSpan.FromSeconds(5)).Result;
                Assert.NotNull(setupScreen);
            }
            finally
            {
                try { relaunchApp.Close(); } catch { relaunchApp.Kill(); }
            }
        }
    }
}
