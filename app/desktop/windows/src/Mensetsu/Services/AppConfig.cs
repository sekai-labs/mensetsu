using System;

namespace Mensetsu.Services
{
    public class AppConfig
    {

        public static string ValidateAndFormatUrl(string url)
        {
            if (string.IsNullOrWhiteSpace(url))
            {
                throw new ArgumentException("Server URL cannot be empty.", nameof(url));
            }

            var raw = url.Trim();
            // Reject any query or fragment characters even if empty (e.g. "?", "#")
            if (raw.Contains('?') || raw.Contains('#'))
            {
                throw new ArgumentException("Server URL must not contain query parameters or fragments.");
            }

            var trimmed = raw.TrimEnd('/');
            if (!Uri.TryCreate(trimmed, UriKind.Absolute, out var uri))
            {
                throw new ArgumentException($"Invalid base API URL: {url}");
            }

            if (!uri.Scheme.Equals("http", StringComparison.OrdinalIgnoreCase) &&
                !uri.Scheme.Equals("https", StringComparison.OrdinalIgnoreCase))
            {
                throw new ArgumentException($"Invalid scheme '{uri.Scheme}'. URL must start with http:// or https://");
            }

            // Reject credentials/userinfo embedded in URL
            if (!string.IsNullOrEmpty(uri.UserInfo))
            {
                throw new ArgumentException("Server URL must not contain embedded user credentials/userinfo.");
            }

            // Enter the API origin only (not an /api/v1 endpoint or subpath)
            // uri.AbsolutePath will be "/" for an origin without path
            if (!string.IsNullOrEmpty(uri.AbsolutePath) && uri.AbsolutePath != "/")
            {
                throw new ArgumentException($"Server URL must be the origin only without a path (got '{uri.AbsolutePath}'). Enter the base API URL (e.g. https://api.example.org).");
            }

            // Require HTTPS outside of localhost / 127.0.0.1 development
            // Note: 0.0.0.0 is a bind address, not loopback; reject 0.0.0.0 as loopback HTTP
            var host = uri.Host;
            var isLoopback = host.Equals("localhost", StringComparison.OrdinalIgnoreCase)
                             || host.Equals("127.0.0.1", StringComparison.OrdinalIgnoreCase)
                             || host.Equals("::1", StringComparison.OrdinalIgnoreCase);

            if (!isLoopback && !uri.Scheme.Equals("https", StringComparison.OrdinalIgnoreCase))
            {
                throw new InvalidOperationException($"Insecure HTTP URL '{url}' is only allowed for loopback development (localhost, 127.0.0.1, ::1).");
            }

            return trimmed;
        }
    }
}
