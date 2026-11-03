using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.Common;
using Mensetsu.Contracts.Model;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class SettingsView : UserControl
    {
        public SettingsViewModel? ViewModel => DataContext as SettingsViewModel;

        public SettingsView()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            var rootGrid = new Grid { Padding = new Thickness(24) };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(rootGrid, AutomationIds.SettingsScreen);

            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Header
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Status / Form
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) }); // Providers List

            // Header
            var headerGrid = new Grid();
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

            var backBtn = new Button { Content = "← Back", Margin = new Thickness(0, 0, 16, 0) };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(backBtn, AutomationIds.SettingsBackButton);
            Grid.SetColumn(backBtn, 0);
            headerGrid.Children.Add(backBtn);

            var title = new TextBlock
            {
                Text = "Settings",
                FontSize = 22,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold,
                VerticalAlignment = VerticalAlignment.Center
            };
            Grid.SetColumn(title, 1);
            headerGrid.Children.Add(title);

            Grid.SetRow(headerGrid, 0);
            rootGrid.Children.Add(headerGrid);

            // Scrollable Settings Content
            var scrollViewer = new ScrollViewer { VerticalScrollBarVisibility = ScrollBarVisibility.Auto };
            var contentStack = new StackPanel { Spacing = 24, Margin = new Thickness(0, 16, 0, 16), MaxWidth = 600, HorizontalAlignment = HorizontalAlignment.Left };

            var errorBanner = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.SettingsErrorBanner);
            contentStack.Children.Add(errorBanner);

            var statusBanner = new InfoBar { Severity = InfoBarSeverity.Success, IsOpen = false };
            contentStack.Children.Add(statusBanner);

            var loadingRing = new ProgressRing { IsActive = false, Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loadingRing, AutomationIds.SettingsLoadingIndicator);
            contentStack.Children.Add(loadingRing);

            // Section 1: Server Endpoint Configuration
            var serverSection = new StackPanel { Spacing = 8 };
            serverSection.Children.Add(new TextBlock { Text = "Rust API Server Endpoint", FontSize = 18, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });
            serverSection.Children.Add(new TextBlock { Text = "Configure the base URL of your self-hosted API. Changing this resets your session.", TextWrapping = TextWrapping.Wrap, Opacity = 0.8 });

            var serverUrlBox = new TextBox { Header = "API Base URL", PlaceholderText = "https://api.example.org" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(serverUrlBox, AutomationIds.SettingsServerUrlInput);
            serverSection.Children.Add(serverUrlBox);

            var saveServerBtn = new Button { Content = "Update Server URL" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(saveServerBtn, AutomationIds.SettingsServerUrlSaveButton);
            serverSection.Children.Add(saveServerBtn);

            contentStack.Children.Add(serverSection);

            // Section 2: LLM Provider Keys
            var llmSection = new StackPanel { Spacing = 12 };
            llmSection.Children.Add(new TextBlock { Text = "LLM Provider Configuration", FontSize = 18, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });

            var providerBox = new TextBox { Header = "Provider Name", PlaceholderText = "e.g. anthropic, openai, groq", Text = "anthropic" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(providerBox, AutomationIds.SettingsProviderInput);
            llmSection.Children.Add(providerBox);

            var keyBox = new PasswordBox { Header = "API Key" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(keyBox, AutomationIds.SettingsKeyInput);
            llmSection.Children.Add(keyBox);

            var saveLlmBtn = new Button { Content = "Save Provider Key" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(saveLlmBtn, AutomationIds.SettingsSaveButton);
            llmSection.Children.Add(saveLlmBtn);

            // Providers List
            var listStack = new StackPanel { Spacing = 8, Margin = new Thickness(0, 8, 0, 0) };
            listStack.Children.Add(new TextBlock { Text = "Configured Providers", FontSize = 16, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });

            var listView = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(listView, AutomationIds.LlmProvidersList);
            listStack.Children.Add(listView);
            llmSection.Children.Add(listStack);

            contentStack.Children.Add(llmSection);

            scrollViewer.Content = contentStack;
            Grid.SetRow(scrollViewer, 1);
            rootGrid.Children.Add(scrollViewer);

            Content = rootGrid;

            // Events
            backBtn.Click += (s, e) => { ViewModel?.GoBack(); };
            serverUrlBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.ServerUrl = serverUrlBox.Text; };
            saveServerBtn.Click += (s, e) =>
            {
                if (ViewModel != null)
                {
                    ViewModel.ServerUrl = serverUrlBox.Text;
                    ViewModel.SaveServerUrl();
                }
            };

            providerBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.InputProvider = providerBox.Text; };
            keyBox.PasswordChanged += (s, e) => { if (ViewModel != null) ViewModel.InputApiKey = keyBox.Password; };

            saveLlmBtn.Click += async (s, e) =>
            {
                if (ViewModel != null)
                {
                    ViewModel.InputProvider = providerBox.Text;
                    ViewModel.InputApiKey = keyBox.Password;
                    await ViewModel.SaveProviderKeyAsync();
                    keyBox.Password = string.Empty;
                }
            };

            DataContextChanged += (s, e) =>
            {
                if (ViewModel != null)
                {
                    serverUrlBox.Text = ViewModel.ServerUrl ?? string.Empty;
                    listView.ItemsSource = ViewModel.Providers;
                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(SettingsViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(SettingsViewModel.StatusMessage))
                        {
                            statusBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.StatusMessage);
                            statusBanner.Message = ViewModel.StatusMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(SettingsViewModel.ServerUrl))
                        {
                            if (serverUrlBox.Text != ViewModel.ServerUrl)
                            {
                                serverUrlBox.Text = ViewModel.ServerUrl ?? string.Empty;
                            }
                        }
                        else if (pe.PropertyName == nameof(SettingsViewModel.IsLoading) || pe.PropertyName == nameof(SettingsViewModel.IsSaving))
                        {
                            var active = ViewModel.IsLoading || ViewModel.IsSaving;
                            loadingRing.IsActive = active;
                            loadingRing.Visibility = active ? Visibility.Visible : Visibility.Collapsed;
                        }
                    };
                }
            };
        }
    }
}
