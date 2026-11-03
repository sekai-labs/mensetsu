using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.Common;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class AuthView : UserControl
    {
        public AuthViewModel? ViewModel => DataContext as AuthViewModel;

        public AuthView()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            // Build WinUI3 visual tree code-behind with AutomationProperties.AutomationId
            var rootGrid = new Grid
            {
                HorizontalAlignment = HorizontalAlignment.Center,
                VerticalAlignment = VerticalAlignment.Center,
                Width = 420
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(rootGrid, AutomationIds.AuthScreen);

            var stack = new StackPanel { Spacing = 16 };

            // Title
            var title = new TextBlock
            {
                Text = "Mensetsu Interview Studio",
                FontSize = 24,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold,
                HorizontalAlignment = HorizontalAlignment.Center
            };
            stack.Children.Add(title);

            // Error banner
            var errorBanner = new InfoBar
            {
                Severity = InfoBarSeverity.Error,
                IsOpen = false
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.AuthErrorBanner);
            stack.Children.Add(errorBanner);

            // Tab bar for Mode (Login / Register)
            var modePanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8, HorizontalAlignment = HorizontalAlignment.Center };
            var loginTabBtn = new Button { Content = "Login" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loginTabBtn, AutomationIds.AuthLoginTab);
            var registerTabBtn = new Button { Content = "Register" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(registerTabBtn, AutomationIds.AuthRegisterTab);
            modePanel.Children.Add(loginTabBtn);
            modePanel.Children.Add(registerTabBtn);
            stack.Children.Add(modePanel);

            // Inputs
            var nameBox = new TextBox { Header = "Full Name", PlaceholderText = "Jane Doe", Visibility = Visibility.Collapsed };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(nameBox, AutomationIds.AuthNameInput);
            stack.Children.Add(nameBox);

            var emailBox = new TextBox { Header = "Email Address", PlaceholderText = "user@example.com" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(emailBox, AutomationIds.AuthEmailInput);
            stack.Children.Add(emailBox);

            var passwordBox = new PasswordBox { Header = "Password" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(passwordBox, AutomationIds.AuthPasswordInput);
            stack.Children.Add(passwordBox);

            // 2FA challenge panel
            var twoFaPanel = new StackPanel { Spacing = 8, Visibility = Visibility.Collapsed };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(twoFaPanel, AutomationIds.TwoFaScreen);
            var twoFaErrorBanner = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(twoFaErrorBanner, AutomationIds.TwoFaErrorBanner);
            twoFaPanel.Children.Add(twoFaErrorBanner);

            var twoFaBox = new TextBox { Header = "Enter 6-Digit TOTP Code", PlaceholderText = "123456" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(twoFaBox, AutomationIds.TwoFaCodeInput);
            twoFaPanel.Children.Add(twoFaBox);

            var verifyBtn = new Button { Content = "Verify Code", HorizontalAlignment = HorizontalAlignment.Stretch };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(verifyBtn, AutomationIds.TwoFaVerifyButton);
            twoFaPanel.Children.Add(verifyBtn);
            stack.Children.Add(twoFaPanel);

            // Action Buttons
            var loginBtn = new Button { Content = "Sign In", HorizontalAlignment = HorizontalAlignment.Stretch };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loginBtn, AutomationIds.AuthLoginButton);
            stack.Children.Add(loginBtn);

            var registerBtn = new Button { Content = "Create Account", HorizontalAlignment = HorizontalAlignment.Stretch, Visibility = Visibility.Collapsed };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(registerBtn, AutomationIds.AuthRegisterButton);
            stack.Children.Add(registerBtn);

            // Change Server button (allows pre-auth reconfiguration if server is unreachable)
            var changeServerBtn = new HyperlinkButton
            {
                Content = "Change Server URL",
                HorizontalAlignment = HorizontalAlignment.Center,
                Margin = new Thickness(0, 8, 0, 0)
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(changeServerBtn, AutomationIds.AuthChangeServerButton);
            stack.Children.Add(changeServerBtn);

            // Loading indicator
            var progress = new ProgressRing { IsActive = false, Visibility = Visibility.Collapsed };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(progress, AutomationIds.AuthLoadingIndicator);
            stack.Children.Add(progress);

            rootGrid.Children.Add(stack);
            Content = rootGrid;

            changeServerBtn.Click += (s, e) =>
            {
                ViewModel?.ChangeServer();
            };

            // Wire UI events
            loginTabBtn.Click += (s, e) =>
            {
                if (ViewModel != null) ViewModel.IsRegisterMode = false;
                nameBox.Visibility = Visibility.Collapsed;
                loginBtn.Visibility = Visibility.Visible;
                registerBtn.Visibility = Visibility.Collapsed;
            };

            registerTabBtn.Click += (s, e) =>
            {
                if (ViewModel != null) ViewModel.IsRegisterMode = true;
                nameBox.Visibility = Visibility.Visible;
                loginBtn.Visibility = Visibility.Collapsed;
                registerBtn.Visibility = Visibility.Visible;
            };

            emailBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.Email = emailBox.Text; };
            nameBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.Name = nameBox.Text; };
            passwordBox.PasswordChanged += (s, e) => { if (ViewModel != null) ViewModel.Password = passwordBox.Password; };
            twoFaBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.TwoFaCode = twoFaBox.Text; };

            loginBtn.Click += async (s, e) =>
            {
                if (ViewModel != null) await ViewModel.LoginAsync();
            };

            registerBtn.Click += async (s, e) =>
            {
                if (ViewModel != null) await ViewModel.RegisterAsync();
            };

            verifyBtn.Click += async (s, e) =>
            {
                if (ViewModel != null) await ViewModel.Verify2faAsync();
            };

            DataContextChanged += (s, e) =>
            {
                if (ViewModel != null)
                {
                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(AuthViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(AuthViewModel.IsLoading))
                        {
                            progress.IsActive = ViewModel.IsLoading;
                            progress.Visibility = ViewModel.IsLoading ? Visibility.Visible : Visibility.Collapsed;
                        }
                        else if (pe.PropertyName == nameof(AuthViewModel.Is2faChallenge))
                        {
                            twoFaPanel.Visibility = ViewModel.Is2faChallenge ? Visibility.Visible : Visibility.Collapsed;
                            loginBtn.Visibility = ViewModel.Is2faChallenge ? Visibility.Collapsed : Visibility.Visible;
                        }
                    };
                }
            };
        }
    }
}
