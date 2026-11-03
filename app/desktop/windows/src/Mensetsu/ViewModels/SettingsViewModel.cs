using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using Mensetsu.Contracts.Model;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class SettingsViewModel : ViewModelBase
    {
        private readonly IMensetsuApiService _apiService;
        private readonly IEndpointContext _endpointContext;

        private bool _isLoading;
        private string? _errorMessage;
        private string? _statusMessage;

        private string _serverUrl = string.Empty;
        private string _inputProvider = "anthropic";
        private string _inputApiKey = string.Empty;
        private bool _isSaving;

        public ObservableCollection<LlmConfigItemDto> Providers { get; } = new();

        public event Action? BackRequested;
        public event Action? ServerEndpointChanged;

        public SettingsViewModel(IMensetsuApiService apiService, IEndpointContext endpointContext)
        {
            _apiService = apiService;
            _endpointContext = endpointContext;
            _serverUrl = _endpointContext.CurrentEndpoint ?? string.Empty;
        }

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

        public string? StatusMessage
        {
            get => _statusMessage;
            set => SetProperty(ref _statusMessage, value);
        }

        public string ServerUrl
        {
            get => _serverUrl;
            set => SetProperty(ref _serverUrl, value);
        }

        public string InputProvider
        {
            get => _inputProvider;
            set => SetProperty(ref _inputProvider, value);
        }

        public string InputApiKey
        {
            get => _inputApiKey;
            set => SetProperty(ref _inputApiKey, value);
        }

        public bool IsSaving
        {
            get => _isSaving;
            set => SetProperty(ref _isSaving, value);
        }

        public void SaveServerUrl()
        {
            ErrorMessage = null;
            StatusMessage = null;
            try
            {
                var formatted = AppConfig.ValidateAndFormatUrl(ServerUrl);
                var previous = _endpointContext.CurrentEndpoint;
                _endpointContext.SetEndpoint(formatted);
                ServerUrl = formatted;
                StatusMessage = "Server URL updated. Active session cleared.";

                if (!string.Equals(previous, formatted, StringComparison.OrdinalIgnoreCase))
                {
                    ServerEndpointChanged?.Invoke();
                }
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
        }

        public async Task LoadSettingsAsync()
        {
            IsLoading = true;
            ErrorMessage = null;
            ServerUrl = _endpointContext.CurrentEndpoint ?? string.Empty;

            try
            {
                var settings = await _apiService.GetLlmSettingsAsync();
                Providers.Clear();
                foreach (var s in settings)
                {
                    Providers.Add(s);
                }
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

        public async Task SaveProviderKeyAsync(string? providerOverride = null, string? keyOverride = null)
        {
            var prov = providerOverride ?? InputProvider;
            var key = keyOverride ?? InputApiKey;

            if (string.IsNullOrWhiteSpace(prov) || string.IsNullOrWhiteSpace(key))
            {
                ErrorMessage = "Provider and API Key cannot be empty.";
                return;
            }

            IsSaving = true;
            ErrorMessage = null;
            StatusMessage = null;

            try
            {
                await _apiService.UpsertLlmSettingAsync(prov.Trim(), key.Trim());
                InputApiKey = string.Empty;
                await LoadSettingsAsync();
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
            }
            finally
            {
                IsSaving = false;
            }
        }

        public void GoBack()
        {
            BackRequested?.Invoke();
        }
    }
}
