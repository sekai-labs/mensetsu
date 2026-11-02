using System;
using System.Linq;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Mensetsu.Common;
using Mensetsu.Contracts.Model;
using Mensetsu.ViewModels;

namespace Mensetsu.Views
{
    public sealed partial class WorkspaceDetailView : UserControl
    {
        public WorkspaceDetailViewModel? ViewModel => DataContext as WorkspaceDetailViewModel;

        public WorkspaceDetailView()
        {
            InitializeComponent();
        }

        private void InitializeComponent()
        {
            var rootGrid = new Grid { Padding = new Thickness(24) };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(rootGrid, AutomationIds.WorkspaceDetailScreen);

            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Header
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = GridLength.Auto }); // Error / Loading
            rootGrid.RowDefinitions.Add(new RowDefinition { Height = new GridLength(1, GridUnitType.Star) }); // Tabs Content

            // --- Header ---
            var headerGrid = new Grid();
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = GridLength.Auto });
            headerGrid.ColumnDefinitions.Add(new ColumnDefinition { Width = new GridLength(1, GridUnitType.Star) });

            var backBtn = new Button { Content = "← Back to Workspaces", Margin = new Thickness(0, 0, 16, 0) };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(backBtn, AutomationIds.DetailBackButton);
            Grid.SetColumn(backBtn, 0);
            headerGrid.Children.Add(backBtn);

            var titleBlock = new TextBlock
            {
                FontSize = 22,
                FontWeight = Microsoft.UI.Text.FontWeights.Bold,
                VerticalAlignment = VerticalAlignment.Center
            };
            Grid.SetColumn(titleBlock, 1);
            headerGrid.Children.Add(titleBlock);

            Grid.SetRow(headerGrid, 0);
            rootGrid.Children.Add(headerGrid);

            // --- Status / Error ---
            var statusStack = new StackPanel { Spacing = 8, Margin = new Thickness(0, 12, 0, 12) };
            var errorBanner = new InfoBar { Severity = InfoBarSeverity.Error, IsOpen = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(errorBanner, AutomationIds.DetailErrorBanner);
            statusStack.Children.Add(errorBanner);

            var loadingRing = new ProgressRing { IsActive = false, Visibility = Visibility.Collapsed, HorizontalAlignment = HorizontalAlignment.Left };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(loadingRing, AutomationIds.DetailLoadingIndicator);
            statusStack.Children.Add(loadingRing);

            Grid.SetRow(statusStack, 1);
            rootGrid.Children.Add(statusStack);

            // --- Tab Control (TabView / Pivot) ---
            var tabView = new TabView { IsAddTabButtonVisible = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabView, AutomationIds.WorkspaceDetailScreen + "_tabs");

            // Tab 1: Overview
            var tabOverview = new TabViewItem { Header = "Overview", IsClosable = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabOverview, AutomationIds.TabOverview);
            var overviewScroll = new ScrollViewer();
            var overviewStack = new StackPanel { Spacing = 16, Padding = new Thickness(12) };

            var compNameText = new TextBlock { FontSize = 20, FontWeight = Microsoft.UI.Text.FontWeights.SemiBold };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(compNameText, AutomationIds.OverviewCompanyName);
            overviewStack.Children.Add(compNameText);

            var roleTitleText = new TextBlock { FontSize = 16 };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(roleTitleText, AutomationIds.OverviewRoleTitle);
            overviewStack.Children.Add(roleTitleText);

            var statusBadge = new Border
            {
                Padding = new Thickness(8, 4, 8, 4),
                CornerRadius = new CornerRadius(4),
                HorizontalAlignment = HorizontalAlignment.Left,
                Child = new TextBlock { FontWeight = Microsoft.UI.Text.FontWeights.Bold }
            };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(statusBadge, AutomationIds.OverviewStatusBadge);
            overviewStack.Children.Add(statusBadge);

            var matchMetric = new TextBlock { FontSize = 16 };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(matchMetric, AutomationIds.OverviewMatchMetric);
            overviewStack.Children.Add(matchMetric);

            var gapsMetric = new TextBlock { FontSize = 16 };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(gapsMetric, AutomationIds.OverviewGapsMetric);
            overviewStack.Children.Add(gapsMetric);

            var questionsMetric = new TextBlock { FontSize = 16 };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(questionsMetric, AutomationIds.OverviewQuestionsMetric);
            overviewStack.Children.Add(questionsMetric);

            overviewScroll.Content = overviewStack;
            tabOverview.Content = overviewScroll;
            tabView.TabItems.Add(tabOverview);

            // Tab 2: Match Analysis
            var tabMatch = new TabViewItem { Header = "Match Analysis", IsClosable = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabMatch, AutomationIds.TabMatch);
            var matchScroll = new ScrollViewer();
            var matchStack = new StackPanel { Spacing = 16, Padding = new Thickness(12) };

            var matchRatioText = new TextBlock { FontSize = 24, FontWeight = Microsoft.UI.Text.FontWeights.Bold };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(matchRatioText, AutomationIds.MatchRatioText);
            matchStack.Children.Add(matchRatioText);

            matchStack.Children.Add(new TextBlock { Text = "Strengths Evidence", FontWeight = Microsoft.UI.Text.FontWeights.Bold, Margin = new Thickness(0, 8, 0, 4) });
            var strengthsList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(strengthsList, AutomationIds.MatchStrengthsList);
            matchStack.Children.Add(strengthsList);

            matchStack.Children.Add(new TextBlock { Text = "Gaps & Areas to Address", FontWeight = Microsoft.UI.Text.FontWeights.Bold, Margin = new Thickness(0, 8, 0, 4) });
            var gapsList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(gapsList, AutomationIds.MatchGapsList);
            matchStack.Children.Add(gapsList);

            matchStack.Children.Add(new TextBlock { Text = "Interview Risks", FontWeight = Microsoft.UI.Text.FontWeights.Bold, Margin = new Thickness(0, 8, 0, 4) });
            var risksList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(risksList, AutomationIds.MatchRisksList);
            matchStack.Children.Add(risksList);

            matchScroll.Content = matchStack;
            tabMatch.Content = matchScroll;
            tabView.TabItems.Add(tabMatch);

            // Tab 3: Company Dossier
            var tabCompany = new TabViewItem { Header = "Company Dossier", IsClosable = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabCompany, AutomationIds.TabCompany);
            var companyScroll = new ScrollViewer();
            var companyStack = new StackPanel { Spacing = 16, Padding = new Thickness(12) };

            var companyOverviewText = new TextBlock { TextWrapping = TextWrapping.Wrap };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(companyOverviewText, AutomationIds.CompanyOverviewText);
            companyStack.Children.Add(companyOverviewText);

            companyStack.Children.Add(new TextBlock { Text = "Verified Tech Stack", FontWeight = Microsoft.UI.Text.FontWeights.Bold });
            var techStackList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(techStackList, AutomationIds.CompanyTechStackList);
            companyStack.Children.Add(techStackList);

            companyStack.Children.Add(new TextBlock { Text = "Engineering Culture", FontWeight = Microsoft.UI.Text.FontWeights.Bold });
            var cultureList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(cultureList, AutomationIds.CompanyCultureList);
            companyStack.Children.Add(cultureList);

            companyStack.Children.Add(new TextBlock { Text = "Source Citations", FontWeight = Microsoft.UI.Text.FontWeights.Bold });
            var citationsList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(citationsList, AutomationIds.CompanyCitationsList);
            companyStack.Children.Add(citationsList);

            companyScroll.Content = companyStack;
            tabCompany.Content = companyScroll;
            tabView.TabItems.Add(tabCompany);

            // Tab 4: Learning Path
            var tabLearning = new TabViewItem { Header = "Learning Path", IsClosable = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabLearning, AutomationIds.TabLearning);
            var learningStack = new StackPanel { Spacing = 12, Padding = new Thickness(12) };

            var learningList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(learningList, AutomationIds.LearningUnitsList);
            learningStack.Children.Add(learningList);

            tabLearning.Content = learningStack;
            tabView.TabItems.Add(tabLearning);

            // Tab 5: Questions Book
            var tabQuestions = new TabViewItem { Header = "Question Book", IsClosable = false };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(tabQuestions, AutomationIds.TabQuestions);
            var questionsStack = new StackPanel { Spacing = 12, Padding = new Thickness(12) };

            var diffPanel = new StackPanel { Orientation = Orientation.Horizontal, Spacing = 8 };
            diffPanel.Children.Add(new TextBlock { Text = "Difficulty:", VerticalAlignment = VerticalAlignment.Center });
            var diffCombo = new ComboBox();
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(diffCombo, AutomationIds.QuestionsDifficultyPicker);
            diffCombo.Items.Add("all");
            diffCombo.Items.Add("easy");
            diffCombo.Items.Add("medium");
            diffCombo.Items.Add("hard");
            diffCombo.SelectedIndex = 0;
            diffPanel.Children.Add(diffCombo);
            questionsStack.Children.Add(diffPanel);

            var questionsList = new ListView { SelectionMode = ListViewSelectionMode.None };
            Microsoft.UI.Xaml.Automation.AutomationProperties.SetAutomationId(questionsList, AutomationIds.QuestionsList);
            questionsStack.Children.Add(questionsList);

            tabQuestions.Content = questionsStack;
            tabView.TabItems.Add(tabQuestions);

            Grid.SetRow(tabView, 2);
            rootGrid.Children.Add(tabView);

            Content = rootGrid;

            // Wire events
            backBtn.Click += (s, e) => { ViewModel?.GoBack(); };
            diffCombo.SelectionChanged += (s, e) =>
            {
                if (ViewModel != null && diffCombo.SelectedItem is string selected)
                {
                    ViewModel.SelectedDifficulty = selected;
                }
            };

            DataContextChanged += (s, e) =>
            {
                if (ViewModel != null)
                {
                    learningList.ItemsSource = ViewModel.FilteredLearningUnits;
                    questionsList.ItemsSource = ViewModel.FilteredQuestions;

                    ViewModel.PropertyChanged += (ps, pe) =>
                    {
                        if (pe.PropertyName == nameof(WorkspaceDetailViewModel.ErrorMessage))
                        {
                            errorBanner.IsOpen = !string.IsNullOrEmpty(ViewModel.ErrorMessage);
                            errorBanner.Message = ViewModel.ErrorMessage ?? "";
                        }
                        else if (pe.PropertyName == nameof(WorkspaceDetailViewModel.IsLoading))
                        {
                            loadingRing.IsActive = ViewModel.IsLoading;
                            loadingRing.Visibility = ViewModel.IsLoading ? Visibility.Visible : Visibility.Collapsed;
                        }
                        else if (pe.PropertyName == nameof(WorkspaceDetailViewModel.Workspace))
                        {
                            var ws = ViewModel.Workspace;
                            if (ws != null)
                            {
                                titleBlock.Text = $"{ws.CompanyName} - {ws.RoleTitle}";
                                compNameText.Text = ws.CompanyName;
                                roleTitleText.Text = ws.RoleTitle;
                                ((TextBlock)statusBadge.Child).Text = ws.Status.ToString().ToUpperInvariant();
                            }
                        }
                        else if (pe.PropertyName == nameof(WorkspaceDetailViewModel.Match))
                        {
                            var m = ViewModel.Match;
                            if (m != null)
                            {
                                var pct = (int)Math.Round(m.OverallMatchRatio * 100);
                                matchRatioText.Text = $"{pct}% Overall Match";
                                matchMetric.Text = $"Match: {pct}%";
                                gapsMetric.Text = $"Gaps: {m.Gaps?.Count ?? 0}";

                                strengthsList.ItemsSource = m.Strengths?.Select(s => $"{s.Skill}: {s.EvidenceQuote}").ToList();
                                gapsList.ItemsSource = m.Gaps?.Select(g => $"{g.Skill}: {g.RequirementQuote}").ToList();
                                risksList.ItemsSource = m.Risks?.ToList();
                            }
                        }
                        else if (pe.PropertyName == nameof(WorkspaceDetailViewModel.Company))
                        {
                            var c = ViewModel.Company;
                            if (c != null)
                            {
                                companyOverviewText.Text = c.Overview;
                                techStackList.ItemsSource = c.TechnologyStack?.ToList();
                                cultureList.ItemsSource = c.EngineeringCulture?.ToList();
                                citationsList.ItemsSource = c.Sources?.Select(s => $"{s.Title} ({s.Url})").ToList();
                            }
                        }
                    };
                }
            };
        }
    }
}
