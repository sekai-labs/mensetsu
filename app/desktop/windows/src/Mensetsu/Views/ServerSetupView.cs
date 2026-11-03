using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.Common;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class ServerSetupView : UserControl
    {
        public ServerSetupViewModel? ViewModel => DataContext as ServerSetupViewModel;

        public ServerSetupView()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            var rootGrid = new Grid
            {
                HorizontalAlignment = HorizontalAlignment.Center,
                VerticalAlignment = VerticalAlignment.Center,
                Width = 460,
                Padding = new Thickness(24)
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(rootGrid, AutomationIds.ServerSetupScreen);

            var stack = new StackPanel { Spacing = 16 };

            var title = new TextBlock
            {
                Text = "Connect to Rust API",
                FontSize = 24,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold,
                HorizontalAlignment = HorizontalAlignment.Center
            };
            stack.Children.Add(title);

            var subtitle = new TextBlock
            {
                Text = "Enter the base URL of your self-hosted Mensetsu Rust API server. For example: https://api.example.org or http://127.0.0.1:3000 for local testing.",
                TextWrapping = TextWrapping.Wrap,
                HorizontalAlignment = HorizontalAlignment.Center,
                TextAlignment = TextAlignment.Center,
                Opacity = 0.8
            };
            stack.Children.Add(subtitle);

            var errorBanner = new InfoBar
            {
                Severity = InfoBarSeverity.Error,
                IsOpen = false
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.ServerUrlErrorBanner);
            stack.Children.Add(errorBanner);

            var urlBox = new TextBox
            {
                Header = "Rust API Base URL",
                PlaceholderText = "https://api.example.org"
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(urlBox, AutomationIds.ServerUrlInput);
            stack.Children.Add(urlBox);

            var buttonsPanel = new StackPanel
            {
                Orientation = Orientation.Horizontal,
                Spacing = 12,
                HorizontalAlignment = HorizontalAlignment.Right
            };

            var cancelBtn = new Button
            {
                Content = "Cancel",
                Visibility = Visibility.Collapsed
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(cancelBtn, AutomationIds.CancelServerUrlButton);
            buttonsPanel.Children.Add(cancelBtn);

            var saveBtn = new Button
            {
                Content = "Save and Connect",
                Style = Application.Current.Resources.TryGetValue("AccentButtonStyle", out var style) ? style as Style : null
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(saveBtn, AutomationIds.SaveServerUrlButton);
            buttonsPanel.Children.Add(saveBtn);

            stack.Children.Add(buttonsPanel);
            rootGrid.Children.Add(stack);
            Content = rootGrid;

            // Events
            urlBox.TextChanged += (s, e) =>
            {
                if (ViewModel != null) ViewModel.ServerUrl = urlBox.Text;
            };

            saveBtn.Click += (s, e) =>
            {
                if (ViewModel != null)
                {
                    ViewModel.ServerUrl = urlBox.Text;
                    ViewModel.SaveEndpoint();
                }
            };

            cancelBtn.Click += (s, e) =>
            {
                ViewModel?.Cancel();
            };

            DataContextChanged += (s, e) =>
            {
                if (ViewModel != null)
                {
                    urlBox.Text = ViewModel.ServerUrl ?? string.Empty;
                    cancelBtn.Visibility = ViewModel.CanCancel ? Visibility.Visible : Visibility.Collapsed;

                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(ServerSetupViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(ServerSetupViewModel.ServerUrl))
                        {
                            if (urlBox.Text != ViewModel.ServerUrl)
                            {
                                urlBox.Text = ViewModel.ServerUrl ?? string.Empty;
                            }
                        }
                        else if (pe.PropertyName == nameof(ServerSetupViewModel.CanCancel))
                        {
                            cancelBtn.Visibility = ViewModel.CanCancel ? Visibility.Visible : Visibility.Collapsed;
                        }
                    };
                }
            };
        }
    }
}
