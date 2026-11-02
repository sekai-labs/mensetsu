using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.Common;
using Mensetsu.Contracts.Model;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class WorkspaceListView : UserControl
    {
        public WorkspaceListViewModel? ViewModel => DataContext as WorkspaceListViewModel;

        public WorkspaceListView()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            var rootGrid = new Grid { Padding = new Thickness(24) };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(rootGrid, AutomationIds.WorkspaceListScreen);

            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Header
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Error & Filters
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) }); // List

            // --- Header Row ---
            var headerGrid = new Grid();
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });

            var title = new TextBlock
            {
                Text = "Preparation Workspaces",
                FontSize = 24,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold
            };
            Grid.SetColumn(title, 0);
            headerGrid.Children.Add(title);

            var actionButtons = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
            var refreshBtn = new Button { Content = "Refresh" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(refreshBtn, AutomationIds.RefreshWorkspacesButton);

            var newWsBtn = new Button { Content = "+ New Workspace" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(newWsBtn, AutomationIds.OpenCreateWorkspaceButton);

            var settingsBtn = new Button { Content = "Settings" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(settingsBtn, AutomationIds.OpenSettingsButton);

            var logoutBtn = new Button { Content = "Log Out" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(logoutBtn, AutomationIds.LogoutButton);

            actionButtons.Children.Add(refreshBtn);
            actionButtons.Children.Add(newWsBtn);
            actionButtons.Children.Add(settingsBtn);
            actionButtons.Children.Add(logoutBtn);
            Grid.SetColumn(actionButtons, 1);
            headerGrid.Children.Add(actionButtons);

            Grid.SetRow(headerGrid, 0);
            rootGrid.Children.Add(headerGrid);

            // --- Filters & Error Row ---
            var filterStack = new StackPanel { Spacing = 12, Margin = new Thickness(0, 16, 0, 16) };

            var errorBanner = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.WorkspaceErrorBanner);
            filterStack.Children.Add(errorBanner);

            var filterPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
            var btnAll = new Button { Content = "All" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(btnAll, AutomationIds.FilterAllButton);

            var btnReady = new Button { Content = "Ready" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(btnReady, AutomationIds.FilterReadyButton);

            var btnProcessing = new Button { Content = "Processing" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(btnProcessing, AutomationIds.FilterProcessingButton);

            filterPanel.Children.Add(btnAll);
            filterPanel.Children.Add(btnReady);
            filterPanel.Children.Add(btnProcessing);
            filterStack.Children.Add(filterPanel);

            var loadingRing = new ProgressRing { IsActive = false, Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loadingRing, AutomationIds.WorkspaceLoadingIndicator);
            filterStack.Children.Add(loadingRing);

            Grid.SetRow(filterStack, 1);
            rootGrid.Children.Add(filterStack);

            // --- List Row ---
            var listView = new ListView { SelectionMode = ListViewSelectionMode.Single };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(listView, AutomationIds.WorkspaceList);

            Grid.SetRow(listView, 2);
            rootGrid.Children.Add(listView);

            // --- Create Workspace Dialog ---
            var createDialog = new ContentDialog
            {
                Title = "New Preparation Workspace",
                PrimaryButtonText = "Create Workspace",
                CloseButtonText = "Cancel",
                DefaultButton = ContentDialogButton.Primary
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(createDialog, AutomationIds.CreateWorkspaceDialog);

            var dialogStack = new StackPanel { Spacing = 12, Width = 400 };
            var createError = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(createError, AutomationIds.CreateWorkspaceErrorBanner);
            dialogStack.Children.Add(createError);

            var companyBox = new TextBox { Header = "Target Company", PlaceholderText = "e.g. ScaleVector, Acme Systems" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(companyBox, AutomationIds.CreateCompanyInput);
            dialogStack.Children.Add(companyBox);

            var roleBox = new TextBox { Header = "Role Title", PlaceholderText = "e.g. Staff Systems Architect" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(roleBox, AutomationIds.CreateRoleInput);
            dialogStack.Children.Add(roleBox);

            // Resume Mode selector: Text vs PDF
            var modePanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 16 };
            var textModeRadio = new RadioButton { Content = "Paste Text", IsChecked = true, GroupName = "ResumeMode" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(textModeRadio, AutomationIds.ResumeModeTextRadio);
            var pdfModeRadio = new RadioButton { Content = "Upload PDF", IsChecked = false, GroupName = "ResumeMode" };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(pdfModeRadio, AutomationIds.ResumeModePdfRadio);
            modePanel.Children.Add(textModeRadio);
            modePanel.Children.Add(pdfModeRadio);
            dialogStack.Children.Add(modePanel);

            // Text Resume input
            var resumeBox = new TextBox { Header = "Résumé Text", AcceptsReturn = true, Height = 100, PlaceholderText = "Paste candidate experience or resume text..." };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(resumeBox, AutomationIds.CreateResumeTextInput);
            dialogStack.Children.Add(resumeBox);

            // PDF Resume input panel
            var pdfPanel = new StackPanel { Spacing = 8, Visibility = Visibility.Collapsed };
            var pickPdfBtn = new Button { Content = "Choose PDF File..." };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(pickPdfBtn, AutomationIds.PickPdfFileButton);
            var selectedFileText = new TextBlock
            {
                Text = "No file chosen (Max 5 MiB, .pdf only)",
                TextWrapping = TextWrapping.Wrap,
                Foreground = new Microsoft.UI.Xaml.Media.SolidColorBrush(Windows.UI.Color.FromArgb(180, 128, 128, 128))
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(selectedFileText, AutomationIds.SelectedPdfFileText);
            pdfPanel.Children.Add(pickPdfBtn);
            pdfPanel.Children.Add(selectedFileText);
            dialogStack.Children.Add(pdfPanel);

            var jdBox = new TextBox { Header = "Job Description Text", AcceptsReturn = true, Height = 100, PlaceholderText = "Paste job requirements..." };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(jdBox, AutomationIds.CreateJobDescriptionInput);
            dialogStack.Children.Add(jdBox);

            createDialog.Content = dialogStack;

            Content = rootGrid;

            // Wire Events
            btnAll.Click += (s, e) => { if (ViewModel != null) ViewModel.SelectedFilter = "all"; };
            btnReady.Click += (s, e) => { if (ViewModel != null) ViewModel.SelectedFilter = "ready"; };
            btnProcessing.Click += (s, e) => { if (ViewModel != null) ViewModel.SelectedFilter = "processing"; };

            refreshBtn.Click += async (s, e) => { if (ViewModel != null) await ViewModel.LoadWorkspacesAsync(); };
            textModeRadio.Checked += (s, e) =>
            {
                resumeBox.Visibility = Visibility.Visible;
                pdfPanel.Visibility = Visibility.Collapsed;
                if (ViewModel != null) ViewModel.CreateResumeMode = "text";
            };

            pdfModeRadio.Checked += (s, e) =>
            {
                resumeBox.Visibility = Visibility.Collapsed;
                pdfPanel.Visibility = Visibility.Visible;
                if (ViewModel != null) ViewModel.CreateResumeMode = "pdf";
            };

            pickPdfBtn.Click += async (s, e) =>
            {
                try
                {
                    var testPdfPath = Environment.GetEnvironmentVariable("MENSETSU_TEST_PDF_PATH");
                    if (!string.IsNullOrEmpty(testPdfPath) && System.IO.File.Exists(testPdfPath))
                    {
                        var fileInfo = new System.IO.FileInfo(testPdfPath);
                        if (fileInfo.Length > 5 * 1024 * 1024)
                        {
                            createError.IsOpen = true;
                            createError.Message = "Selected PDF exceeds 5 MiB limit.";
                            return;
                        }
                        var testBytes = await System.IO.File.ReadAllBytesAsync(testPdfPath);
                        selectedFileText.Text = $"{fileInfo.Name} ({fileInfo.Length:N0} bytes)";
                        createError.IsOpen = false;
                        ViewModel?.SetSelectedPdf(testPdfPath, fileInfo.Name, testBytes);
                        return;
                    }
                    var picker = new Windows.Storage.Pickers.FileOpenPicker();
                    picker.SuggestedStartLocation = Windows.Storage.Pickers.PickerLocationId.DocumentsLibrary;
                    picker.FileTypeFilter.Add(".pdf");

                    // In unpackaged WinUI 3, InitializeWithWindow is required for FileOpenPicker
                    if (App.MainWindow != null)
                    {
                        var hwnd = WinRT.Interop.WindowNative.GetWindowHandle(App.MainWindow);
                        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);
                    }

                    var file = await picker.PickSingleFileAsync();
                    if (file != null)
                    {
                        var props = await file.GetBasicPropertiesAsync();
                        if (props.Size > 5 * 1024 * 1024)
                        {
                            createError.IsOpen = true;
                            createError.Message = "Selected PDF exceeds 5 MiB limit.";
                            return;
                        }

                        using var stream = await file.OpenStreamForReadAsync();
                        using var ms = new System.IO.MemoryStream();
                        await stream.CopyToAsync(ms);
                        var bytes = ms.ToArray();

                        selectedFileText.Text = $"{file.Name} ({props.Size:N0} bytes)";
                        createError.IsOpen = false;
                        ViewModel?.SetSelectedPdf(file.Path, file.Name, bytes);
                    }
                }
                catch (Exception ex)
                {
                    createError.IsOpen = true;
                    createError.Message = $"Failed to pick PDF file: {ex.Message}";
                }
            };

            // PrimaryButton click handler with async execution & deferral
            createDialog.PrimaryButtonClick += async (dialog, args) =>
            {
                if (ViewModel == null) return;
                var deferral = args.GetDeferral();
                try
                {
                    ViewModel.CreateCompany = companyBox.Text;
                    ViewModel.CreateRole = roleBox.Text;
                    ViewModel.CreateResumeText = resumeBox.Text;
                    ViewModel.CreateJobDescription = jdBox.Text;

                    await ViewModel.SubmitCreateWorkspaceAsync();
                    if (!string.IsNullOrEmpty(ViewModel.CreateErrorMessage))
                    {
                        createError.IsOpen = true;
                        createError.Message = ViewModel.CreateErrorMessage;
                        args.Cancel = true; // Keep dialog open for retry
                    }
                }
                catch (Exception ex)
                {
                    createError.IsOpen = true;
                    createError.Message = ex.Message;
                    args.Cancel = true; // Keep dialog open for retry
                }
                finally
                {
                    deferral.Complete();
                }
            };

            newWsBtn.Click += async (s, e) =>
            {
                if (ViewModel == null) return;
                ViewModel.OpenCreateDialog();
                companyBox.Text = "";
                roleBox.Text = "";
                resumeBox.Text = "";
                textModeRadio.IsChecked = true;
                resumeBox.Visibility = Visibility.Visible;
                pdfPanel.Visibility = Visibility.Collapsed;
                selectedFileText.Text = "No file chosen (Max 5 MiB, .pdf only)";
                jdBox.Text = "";
                createError.IsOpen = false;

                createDialog.XamlRoot = XamlRoot;
                await createDialog.ShowAsync();
            };

            settingsBtn.Click += (s, e) => { ViewModel?.OpenSettings(); };
            logoutBtn.Click += async (s, e) => { if (ViewModel != null) await ViewModel.LogoutAsync(); };

            listView.ItemClick += (s, e) =>
            {
                if (e.ClickedItem is WorkspaceResponseDto ws)
                {
                    ViewModel?.SelectWorkspace(ws);
                }
            };
            listView.IsItemClickEnabled = true;

            DataContextChanged += (s, e) =>
            {
                if (ViewModel != null)
                {
                    listView.ItemsSource = ViewModel.FilteredWorkspaces;
                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(WorkspaceListViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(WorkspaceListViewModel.IsLoading))
                        {
                            loadingRing.IsActive = ViewModel.IsLoading;
                            loadingRing.Visibility = ViewModel.IsLoading ? Visibility.Visible : Visibility.Collapsed;
                        }
                    };
                }
            };
        }
    }
}
