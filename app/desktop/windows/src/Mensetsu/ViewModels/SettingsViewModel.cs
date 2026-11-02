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

        private bool _isLoading;
        private string? _errorMessage;

        private string _inputProvider = "anthropic";
        private string _inputApiKey = string.Empty;
        private bool _isSaving;

        public ObservableCollection<LlmConfigItemDto> Providers { get; } = new();

        public event Action? BackRequested;

        public SettingsViewModel(IMensetsuApiService apiService)
        {
            _apiService = apiService;
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

        public async Task LoadSettingsAsync()
        {
            IsLoading = true;
            ErrorMessage = null;

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
