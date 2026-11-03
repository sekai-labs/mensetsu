using System;
using System.IO;

namespace Mensetsu.Services
{
    public interface IServerEndpointStore
    {
        string? GetServerEndpoint();
        void SaveServerEndpoint(string endpoint);
        void ClearServerEndpoint();
    }

    /// <summary>
    /// Persists the validated server endpoint in local AppData across restarts.
    /// Works reliably in both unpackaged WinUI 3 desktop apps and packaged apps.
    /// </summary>
    public class FileServerEndpointStore : IServerEndpointStore
    {
        private readonly string _filePath;

        public FileServerEndpointStore(string? customFilePath = null)
        {
            if (!string.IsNullOrEmpty(customFilePath))
            {
                _filePath = customFilePath;
            }
            else
            {
                var appData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
                var dir = Path.Combine(appData, "Mensetsu");
                _filePath = Path.Combine(dir, "server_endpoint.txt");
            }
        }

        public string? GetServerEndpoint()
        {
            try
            {
                if (File.Exists(_filePath))
                {
                    var text = File.ReadAllText(_filePath).Trim();
                    if (!string.IsNullOrWhiteSpace(text))
                    {
                        return text;
                    }
                }
            }
            catch
            {
                // Ignore IO errors reading cached endpoint
            }
            return null;
        }

        public void SaveServerEndpoint(string endpoint)
        {
            if (string.IsNullOrWhiteSpace(endpoint))
            {
                ClearServerEndpoint();
                return;
            }

            var dir = Path.GetDirectoryName(_filePath);
            if (!string.IsNullOrEmpty(dir) && !Directory.Exists(dir))
            {
                Directory.CreateDirectory(dir);
            }

            File.WriteAllText(_filePath, endpoint.Trim());
        }

        public void ClearServerEndpoint()
        {
            try
            {
                if (File.Exists(_filePath))
                {
                    File.Delete(_filePath);
                }
            }
            catch
            {
                // Ignore IO errors deleting cached endpoint
            }
        }
    }
}
