using System;
using Mensetsu.Services;

namespace Mensetsu.ViewModels
{
    public class ServerSetupViewModel : ViewModelBase
    {
        private readonly IEndpointContext _endpointContext;
        private string _serverUrl = string.Empty;
        private string? _errorMessage;
        private bool _canCancel;

        public event Action? EndpointConfigured;
        public event Action? CancelRequested;

        public ServerSetupViewModel(IEndpointContext endpointContext, bool canCancel = false)
        {
            _endpointContext = endpointContext;
            _canCancel = canCancel;
            _serverUrl = _endpointContext.CurrentEndpoint ?? string.Empty;
        }

        public string ServerUrl
        {
            get => _serverUrl;
            set => SetProperty(ref _serverUrl, value);
        }

        public string? ErrorMessage
        {
            get => _errorMessage;
            set => SetProperty(ref _errorMessage, value);
        }

        public bool CanCancel
        {
            get => _canCancel;
            set => SetProperty(ref _canCancel, value);
        }

        public bool SaveEndpoint()
        {
            ErrorMessage = null;
            try
            {
                // AppConfig.ValidateAndFormatUrl enforces http(s) and HTTPS outside loopback
                var formatted = AppConfig.ValidateAndFormatUrl(ServerUrl);
                _endpointContext.SetEndpoint(formatted);
                EndpointConfigured?.Invoke();
                return true;
            }
            catch (Exception ex)
            {
                ErrorMessage = ex.Message;
                return false;
            }
        }

        public void Cancel()
        {
            CancelRequested?.Invoke();
        }
    }
}
