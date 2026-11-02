using System;
using System.Threading.Tasks;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class MainViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly IAuthSession _session;

        private object? _currentView;

        public MainViewModel(IMensetsuApiService apiService, IAuthSession session)
        {
            _apiService = apiService;
            _session = session;
        }

        public object? CurrentView
        {
            get => _currentView;
            set => SetProperty(ref _currentView, value);
        }

        public async Task InitializeAsync()
        {
            // If we have a refresh token, try refreshing to restore session
            if (!string.IsNullOrEmpty(_session.RefreshToken))
            {
                var refreshed = await _apiService.RefreshAsync();
                if (refreshed)
                {
                    ShowWorkspaceList();
                    return;
                }
            }

            ShowAuth();
        }

        public void ShowAuth()
        {
            var authVm = new AuthViewModel(_apiService, _session);
            authVm.Authenticated += () => ShowWorkspaceList();
            CurrentView = authVm;
        }

        public void ShowWorkspaceList()
        {
            var listVm = new WorkspaceListViewModel(_apiService, _session);
            listVm.OpenWorkspaceRequested += id => ShowWorkspaceDetail(id);
            listVm.OpenSettingsRequested += () => ShowSettings();
            listVm.LoggedOut += () => ShowAuth();
            _ = listVm.LoadWorkspacesAsync();
            CurrentView = listVm;
        }

        public void ShowWorkspaceDetail(string workspaceId)
        {
            var detailVm = new WorkspaceDetailViewModel(_apiService, workspaceId);
            detailVm.BackRequested += () => ShowWorkspaceList();
            _ = detailVm.LoadDetailsAsync();
            CurrentView = detailVm;
        }

        public void ShowSettings()
        {
            var settingsVm = new SettingsViewModel(_apiService);
            settingsVm.BackRequested += () => ShowWorkspaceList();
            _ = settingsVm.LoadSettingsAsync();
            CurrentView = settingsVm;
        }
    }
}
