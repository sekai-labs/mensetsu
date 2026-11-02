using System;

namespace Mensetsu.Services
{
    public class AppConfig
    {
        private const string DefaultLocalApiUrl = "http://127.0.0.1:3000";

        public static string BaseApiUrl
        {
            get
            {
                var envUrl = Environment.GetEnvironmentVariable("MENSETSU_API_URL");
                if (!string.IsNullOrWhiteSpace(envUrl))
                {
                    return ValidateAndFormatUrl(envUrl);
                }

                return DefaultLocalApiUrl;
            }
        }

        public static string ValidateAndFormatUrl(string url)
        {
            var trimmed = url.Trim().TrimEnd('/');
            if (!Uri.TryCreate(trimmed, UriKind.Absolute, out var uri))
            {
                throw new ArgumentException($"Invalid base API URL: {url}");
            }

            // Require HTTPS outside of localhost / 127.0.0.1 development
            var isLocalhost = uri.Host.Equals("localhost", StringComparison.OrdinalIgnoreCase)
                              || uri.Host.Equals("127.0.0.1", StringComparison.OrdinalIgnoreCase)
                              || uri.Host.Equals("::1", StringComparison.OrdinalIgnoreCase);

            if (!isLocalhost && !uri.Scheme.Equals("https", StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException($"Insecure HTTP URL '{url}' is only allowed for localhost / 127.0.0.1.");
            }

            return trimmed;
        }
    }
}
