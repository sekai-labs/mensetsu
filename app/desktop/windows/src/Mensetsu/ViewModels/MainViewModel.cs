using System;
using System.Threading.Tasks;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class MainViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly IAuthSession _session;
        private readonly IEndpointContext _endpointContext;

        private object? _currentView;

        public MainViewModel(IMensetsuApiService apiService, IAuthSession session, IEndpointContext endpointContext)
        {
            _apiService = apiService;
            _session = session;
            _endpointContext = endpointContext;
        }

        public object? CurrentView
        {
            get => _currentView;
            set => SetProperty(ref _currentView, value);
        }

        public async Task InitializeAsync()
        {
            // If endpoint is not configured (first launch), show server setup prompt first.
            // No auth/API calls allowed before an endpoint is selected.
            if (!_endpointContext.IsConfigured)
            {
                ShowServerSetup(canCancel: false);
                return;
            }

            // If we have a refresh token, try refreshing to restore session
            if (!string.IsNullOrEmpty(_session.RefreshToken))
            {
                try
                {
                    var refreshed = await _apiService.RefreshAsync();
                    if (refreshed)
                    {
                        ShowWorkspaceList();
                        return;
                    }
                }
                catch
                {
                    // Server unreachable or token expired, proceed to auth screen
                }
            }

            ShowAuth();
        }

        public void ShowServerSetup(bool canCancel)
        {
            var setupVm = new ServerSetupViewModel(_endpointContext, canCancel: canCancel);
            setupVm.EndpointConfigured += () =>
            {
                // Transition to Auth after endpoint is saved
                ShowAuth();
            };
            setupVm.CancelRequested += () =>
            {
                ShowAuth();
            };
            CurrentView = setupVm;
        }

        public void ShowAuth()
        {
            var authVm = new AuthViewModel(_apiService, _session);
            authVm.Authenticated += () => ShowWorkspaceList();
            authVm.ChangeServerRequested += () => ShowServerSetup(canCancel: true);
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
            var settingsVm = new SettingsViewModel(_apiService, _endpointContext);
            settingsVm.BackRequested += () => ShowWorkspaceList();
            settingsVm.ServerEndpointChanged += () =>
            {
                // When the server endpoint is changed in settings, user is logged out and returned to auth
                ShowAuth();
            };
            _ = settingsVm.LoadSettingsAsync();
            CurrentView = settingsVm;
        }
    }
}
