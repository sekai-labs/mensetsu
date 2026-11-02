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
                Text = "LLM Provider Settings",
                FontSize = 22,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold,
                VerticalAlignment = VerticalAlignment.Center
            };
            Grid.SetColumn(title, 1);
            headerGrid.Children.Add(title);

            Grid.SetRow(headerGrid, 0);
            rootGrid.Children.Add(headerGrid);

            // Form / Controls
            var formStack = new StackPanel { Spacing = 12, Margin = new Thickness(0, 16, 0, 16), MaxWidth = 500, HorizontalAlignment = HorizontalAlignment.Left };

            var errorBanner = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.SettingsErrorBanner);
            formStack.Children.Add(errorBanner);

            var loadingRing = new ProgressRing { IsActive = false, Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loadingRing, AutomationIds.SettingsLoadingIndicator);
            formStack.Children.Add(loadingRing);

            var providerBox = new TextBox { Header = "Provider Name", PlaceholderText = "e.g. anthropic, openai, groq", Text = "anthropic" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(providerBox, AutomationIds.SettingsProviderInput);
            formStack.Children.Add(providerBox);

            var keyBox = new PasswordBox { Header = "API Key" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(keyBox, AutomationIds.SettingsKeyInput);
            formStack.Children.Add(keyBox);

            var saveBtn = new Button { Content = "Save Provider Key" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(saveBtn, AutomationIds.SettingsSaveButton);
            formStack.Children.Add(saveBtn);

            Grid.SetRow(formStack, 1);
            rootGrid.Children.Add(formStack);

            // Providers List
            var listStack = new StackPanel { Spacing = 8 };
            listStack.Children.Add(new TextBlock { Text = "Configured Providers", FontSize = 18, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold });

            var listView = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(listView, AutomationIds.LlmProvidersList);
            listStack.Children.Add(listView);

            Grid.SetRow(listStack, 2);
            rootGrid.Children.Add(listStack);

            Content = rootGrid;

            // Events
            backBtn.Click += (s, e) => { ViewModel?.GoBack(); };
            providerBox.TextChanged += (s, e) => { if (ViewModel != null) ViewModel.InputProvider = providerBox.Text; };
            keyBox.PasswordChanged += (s, e) => { if (ViewModel != null) ViewModel.InputApiKey = keyBox.Password; };

            saveBtn.Click += async (s, e) =>
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
                    listView.ItemsSource = ViewModel.Providers;
                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(SettingsViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
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
