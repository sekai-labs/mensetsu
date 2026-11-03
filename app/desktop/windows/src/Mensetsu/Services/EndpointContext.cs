using System;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace Mensetsu.Services
{
    public interface IEndpointContext : INotifyPropertyChanged
    {
        string? CurrentEndpoint { get; }
        bool IsConfigured { get; }
        void SetEndpoint(string endpoint);
    }

    public class EndpointContext : IEndpointContext
    {
        private readonly IServerEndpointStore _endpointStore;
        private readonly IAuthSession _session;
        private string? _currentEndpoint;

        public event PropertyChangedEventHandler? PropertyChanged;

        public EndpointContext(IServerEndpointStore endpointStore, IAuthSession session)
        {
            _endpointStore = endpointStore;
            _session = session;

            var saved = _endpointStore.GetServerEndpoint();
            if (!string.IsNullOrEmpty(saved))
            {
                try
                {
                    _currentEndpoint = AppConfig.ValidateAndFormatUrl(saved);
                }
                catch
                {
                    _currentEndpoint = null;
                }
            }
        }

        public string? CurrentEndpoint => _currentEndpoint;

        public bool IsConfigured => !string.IsNullOrEmpty(_currentEndpoint);

        public void SetEndpoint(string endpoint)
        {
            var validated = AppConfig.ValidateAndFormatUrl(endpoint);

            // If already saved on disk and memory matches, no-op
            var savedDisk = _endpointStore.GetServerEndpoint();
            var alreadyPersisted = !string.IsNullOrEmpty(savedDisk) &&
                                   string.Equals(savedDisk, validated, StringComparison.OrdinalIgnoreCase);

            if (string.Equals(_currentEndpoint, validated, StringComparison.OrdinalIgnoreCase) && alreadyPersisted)
            {
                return;
            }

            // Switching servers must reset session and credentials to prevent token leakage
            _session.Clear();

            // Persist to disk BEFORE updating in-memory _currentEndpoint so IO failures do not leave memory inconsistent
            _endpointStore.SaveServerEndpoint(validated);

            _currentEndpoint = validated;

            OnPropertyChanged(nameof(CurrentEndpoint));
            OnPropertyChanged(nameof(IsConfigured));
        }

        protected virtual void OnPropertyChanged([CallerMemberName] string? propertyName = null)
        {
            PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
        }
    }
}
