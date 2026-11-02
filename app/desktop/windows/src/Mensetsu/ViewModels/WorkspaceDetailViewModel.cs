using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Mensetsu.Contracts.Model;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class WorkspaceDetailViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly string _workspaceId;

        private bool _isLoading;
        private string? _errorMessage;
        private int _selectedTabIndex; // 0=Overview, 1=Match, 2=Company, 3=Learning, 4=Questions

        private WorkspaceResponseDto? _workspace;
        private MatchAnalysis? _match;
        private CompanyResearch? _company;
        private List<LearningUnit> _learningUnits = new();
        private List<InterviewQuestion> _allQuestions = new();
        private string _selectedDifficulty = "all";

        public ObservableCollection<LearningUnit> FilteredLearningUnits { get; } = new();
        public ObservableCollection<InterviewQuestion> FilteredQuestions { get; } = new();

        public event Action? BackRequested;

        public WorkspaceDetailViewModel(IMensetsuApiService apiService, string workspaceId)
        {
            _apiService = apiService;
            _workspaceId = workspaceId;
        }

        public string WorkspaceId => _workspaceId;

        public bool IsLoading
        {
            get => _isLoading;
            set => SetProperty(ref _isLoading, value);
        }

        public string? ErrorMessage
        {
            get => _errorMessage;
            set => SetProperty(ref _errorMessage, value);
        }

        public int SelectedTabIndex
        {
            get => _selectedTabIndex;
            set => SetProperty(ref _selectedTabIndex, value);
        }

        public WorkspaceResponseDto? Workspace
        {
            get => _workspace;
            private set => SetProperty(ref _workspace, value);
        }

        public MatchAnalysis? Match
        {
            get => _match;
            private set => SetProperty(ref _match, value);
        }

        public CompanyResearch? Company
        {
            get => _company;
            private set => SetProperty(ref _company, value);
        }

        public string SelectedDifficulty
        {
            get => _selectedDifficulty;
            set
            {
                if (SetProperty(ref _selectedDifficulty, value))
                {
                    ApplyQuestionsFilter();
                }
            }
        }

        public async Task LoadDetailsAsync()
        {
            IsLoading = true;
            ErrorMessage = null;

            try
            {
                // Load workspace core info
                Workspace = await _apiService.GetWorkspaceAsync(_workspaceId);

                // Load all tab data concurrently
                var matchTask = _apiService.GetWorkspaceMatchAsync(_workspaceId);
                var companyTask = _apiService.GetWorkspaceCompanyAsync(_workspaceId);
                var learningTask = _apiService.GetWorkspaceLearningAsync(_workspaceId);
                var questionsTask = _apiService.GetWorkspaceQuestionsAsync(_workspaceId);

                await Task.WhenAll(matchTask, companyTask, learningTask, questionsTask);

                Match = await matchTask;
                Company = await companyTask;
                _learningUnits = await learningTask;
                _allQuestions = await questionsTask;

                UpdateLearningUnitsList();
                ApplyQuestionsFilter();
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsLoading = false;
            }
        }

        private void UpdateLearningUnitsList()
        {
            FilteredLearningUnits.Clear();
            foreach (var lu in _learningUnits)
            {
                FilteredLearningUnits.Add(lu);
            }
        }

        private void ApplyQuestionsFilter()
        {
            FilteredQuestions.Clear();
            var filtered = _allQuestions.Where(q =>
            {
                if (_selectedDifficulty.Equals("all", StringComparison.OrdinalIgnoreCase)) return true;
                return q.Difficulty.ToString().Equals(_selectedDifficulty, StringComparison.OrdinalIgnoreCase);
            });

            foreach (var q in filtered)
            {
                FilteredQuestions.Add(q);
            }
        }

        public async Task ToggleLearningUnitCompletedAsync(string lessonId)
        {
            try
            {
                var success = await _apiService.ToggleLearningUnitAsync(_workspaceId, lessonId);
                if (success)
                {
                    var unit = _learningUnits.FirstOrDefault(u => u.Id == lessonId);
                    if (unit != null)
                    {
                        var current = unit.Completed == true;
                        unit.Completed = !current;
                    }
                    UpdateLearningUnitsList();
                    OnPropertyChanged(nameof(FilteredLearningUnits));
                }
            }
            catch (Exception ex)
            {
                ErrorMessage = $"Failed to toggle learning unit: {ex.Message}";
            }
        }

        public void GoBack()
        {
            BackRequested?.Invoke();
        }
    }
}
