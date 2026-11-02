using System;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Mensetsu.Contracts.Model;

namespace Mensetsu.Services
{
    public interface IAuthSession
    {
        string? AccessToken { get; }
        string? RefreshToken { get; }
        UserResponseDto? CurrentUser { get; }
        bool IsAuthenticated { get; }
        void SetTokens(string accessToken, string refreshToken, UserResponseDto? user = null);
        void SetAccessToken(string accessToken);
        void SetUser(UserResponseDto user);
        void Clear();
    }

    public class AuthSession : IAuthSession
    {
        private readonly ISecureTokenStore _tokenStore;
        private string? _accessToken; // Memory-only!
        private string? _refreshToken;
        private UserResponseDto? _currentUser;

        public AuthSession(ISecureTokenStore tokenStore)
        {
            _tokenStore = tokenStore;
            _refreshToken = _tokenStore.GetRefreshToken();
        }

        public string? AccessToken => _accessToken;
        public string? RefreshToken => _refreshToken;
        public UserResponseDto? CurrentUser => _currentUser;
        public bool IsAuthenticated => !string.IsNullOrEmpty(_accessToken);

        public void SetTokens(string accessToken, string refreshToken, UserResponseDto? user = null)
        {
            _accessToken = accessToken;
            _refreshToken = refreshToken;
            if (user != null)
            {
                _currentUser = user;
            }
            _tokenStore.SaveRefreshToken(refreshToken);
        }

        public void SetAccessToken(string accessToken)
        {
            _accessToken = accessToken;
        }

        public void SetUser(UserResponseDto user)
        {
            _currentUser = user;
        }

        public void Clear()
        {
            _accessToken = null;
            _refreshToken = null;
            _currentUser = null;
            _tokenStore.ClearRefreshToken();
        }
    }

    /// <summary>
    /// DelegatingHandler that injects the memory-only Bearer access token,
    /// and performs thread-safe serialized refresh + 1x retry on 401 Unauthorized responses.
    /// </summary>
    public class AuthenticatedHttpClientHandler : DelegatingHandler
    {
        private readonly IAuthSession _session;
        private readonly SemaphoreSlim _refreshLock = new SemaphoreSlim(1, 1);
        private readonly Uri _baseAddress;

        public AuthenticatedHttpClientHandler(IAuthSession session, Uri baseAddress, HttpMessageHandler? innerHandler = null)
            : base(innerHandler ?? new HttpClientHandler())
        {
            _session = session;
            _baseAddress = baseAddress;
        }

        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            var isAuthRequest = request.RequestUri != null &&
                                (request.RequestUri.AbsolutePath.Contains("/api/v1/auth/login") ||
                                 request.RequestUri.AbsolutePath.Contains("/api/v1/auth/register") ||
                                 request.RequestUri.AbsolutePath.Contains("/api/v1/auth/refresh"));

            if (!isAuthRequest && !string.IsNullOrEmpty(_session.AccessToken))
            {
                request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", _session.AccessToken);
            }

            var response = await base.SendAsync(request, cancellationToken);

            if (response.StatusCode == HttpStatusCode.Unauthorized && !isAuthRequest)
            {
                var refreshed = await TryRefreshTokenAsync(cancellationToken);
                if (refreshed)
                {
                    // Clone and retry request once with new token
                    using var clonedRequest = await CloneHttpRequestMessageAsync(request);
                    clonedRequest.Headers.Authorization = new AuthenticationHeaderValue("Bearer", _session.AccessToken);
                    return await base.SendAsync(clonedRequest, cancellationToken);
                }
            }

            return response;
        }

        private async Task<bool> TryRefreshTokenAsync(CancellationToken cancellationToken)
        {
            var currentRt = _session.RefreshToken;
            if (string.IsNullOrEmpty(currentRt))
            {
                return false;
            }

            await _refreshLock.WaitAsync(cancellationToken);
            try
            {
                // Recheck if another thread already refreshed
                if (_session.RefreshToken != currentRt && !string.IsNullOrEmpty(_session.AccessToken))
                {
                    return true;
                }

                using var refreshClient = new HttpClient { BaseAddress = _baseAddress };
                var payload = new RefreshTokenRequestDto(new Option<string?>(currentRt));
                var json = JsonSerializer.Serialize(payload, new JsonSerializerOptions
                {
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                });

                var content = new StringContent(json, Encoding.UTF8, "application/json");
                var response = await refreshClient.PostAsync("/api/v1/auth/refresh", content, cancellationToken);

                if (response.IsSuccessStatusCode)
                {
                    var responseJson = await response.Content.ReadAsStringAsync(cancellationToken);
                    using var doc = JsonDocument.Parse(responseJson);
                    var root = doc.RootElement;
                    var newAccessToken = root.GetProperty("access_token").GetString();
                    var newRefreshToken = root.GetProperty("refresh_token").GetString();

                    if (!string.IsNullOrEmpty(newAccessToken) && !string.IsNullOrEmpty(newRefreshToken))
                    {
                        _session.SetTokens(newAccessToken, newRefreshToken);
                        return true;
                    }
                }

                // If refresh failed (401 or expired token), clear session
                _session.Clear();
                return false;
            }
            catch
            {
                _session.Clear();
                return false;
            }
            finally
            {
                _refreshLock.Release();
            }
        }

        private static async Task<HttpRequestMessage> CloneHttpRequestMessageAsync(HttpRequestMessage req)
        {
            var clone = new HttpRequestMessage(req.Method, req.RequestUri)
            {
                Version = req.Version
            };

            if (req.Content != null)
            {
                var ms = new System.IO.MemoryStream();
                await req.Content.CopyToAsync(ms);
                ms.Position = 0;
                clone.Content = new StreamContent(ms);

                foreach (var header in req.Content.Headers)
                {
                    clone.Content.Headers.TryAddWithoutValidation(header.Key, header.Value);
                }
            }

            foreach (var header in req.Headers)
            {
                clone.Headers.TryAddWithoutValidation(header.Key, header.Value);
            }

            return clone;
        }
    }
}
