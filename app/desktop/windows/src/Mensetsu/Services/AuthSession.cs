using System;
using System.Net;
using System.Net.Http;
using System.Net.Http.Headers;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Mensetsu.Contracts.Client;
using Mensetsu.Contracts.Model;

namespace Mensetsu.Services
{
    public interface IAuthSession
    {
        string? AccessToken { get; }
        string? RefreshToken { get; }
        UserResponseDto? CurrentUser { get; }
        bool IsAuthenticated { get; }
        long Epoch { get; }
        void SetTokens(string accessToken, string refreshToken, UserResponseDto? user = null);
        bool TrySetTokensForEpoch(long expectedEpoch, string accessToken, string refreshToken, UserResponseDto? user = null);
        void SetAccessToken(string accessToken);
        void SetUser(UserResponseDto user);
        void Clear();
    }

    public class AuthSession : IAuthSession
    {
        private readonly ISecureTokenStore _tokenStore;
        private readonly object _sessionLock = new object();
        private string? _accessToken; // Memory-only!
        private string? _refreshToken;
        private UserResponseDto? _currentUser;
        private long _epoch;

        public AuthSession(ISecureTokenStore tokenStore)
        {
            _tokenStore = tokenStore;
            _refreshToken = _tokenStore.GetRefreshToken();
            _epoch = 1;
        }

        public string? AccessToken
        {
            get { lock (_sessionLock) return _accessToken; }
        }

        public string? RefreshToken
        {
            get { lock (_sessionLock) return _refreshToken; }
        }

        public UserResponseDto? CurrentUser
        {
            get { lock (_sessionLock) return _currentUser; }
        }

        public bool IsAuthenticated
        {
            get { lock (_sessionLock) return !string.IsNullOrEmpty(_accessToken); }
        }

        public long Epoch
        {
            get { lock (_sessionLock) return _epoch; }
        }

        public void SetTokens(string accessToken, string refreshToken, UserResponseDto? user = null)
        {
            lock (_sessionLock)
            {
                _accessToken = accessToken;
                _refreshToken = refreshToken;
                if (user != null)
                {
                    _currentUser = user;
                }
                _tokenStore.SaveRefreshToken(refreshToken);
            }
        }

        public bool TrySetTokensForEpoch(long expectedEpoch, string accessToken, string refreshToken, UserResponseDto? user = null)
        {
            lock (_sessionLock)
            {
                if (_epoch != expectedEpoch)
                {
                    // Stale epoch (e.g. server switch or clear occurred while refresh was in flight) -> discard!
                    return false;
                }

                _accessToken = accessToken;
                _refreshToken = refreshToken;
                if (user != null)
                {
                    _currentUser = user;
                }
                _tokenStore.SaveRefreshToken(refreshToken);
                return true;
            }
        }

        public void SetAccessToken(string accessToken)
        {
            lock (_sessionLock)
            {
                _accessToken = accessToken;
            }
        }

        public void SetUser(UserResponseDto user)
        {
            lock (_sessionLock)
            {
                _currentUser = user;
            }
        }

        public void Clear()
        {
            lock (_sessionLock)
            {
                // Increment epoch so any in-flight refresh from prior server or session is discarded
                _epoch++;
                _accessToken = null;
                _refreshToken = null;
                _currentUser = null;
                _tokenStore.ClearRefreshToken();
            }
        }
    }

    /// <summary>
    /// DelegatingHandler that injects the memory-only Bearer access token,
    /// and performs thread-safe serialized refresh + 1x retry on 401 Unauthorized responses.
    /// </summary>
    public class AuthenticatedHttpClientHandler : DelegatingHandler
    {
        private readonly IAuthSession _session;
        private readonly IEndpointContext _endpointContext;
        private readonly SemaphoreSlim _refreshLock = new SemaphoreSlim(1, 1);

        public AuthenticatedHttpClientHandler(IAuthSession session, IEndpointContext endpointContext, HttpMessageHandler? innerHandler = null)
            : base(innerHandler ?? new HttpClientHandler())
        {
            _session = session;
            _endpointContext = endpointContext;
        }

        protected override async Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken)
        {
            var isAuthRequest = request.RequestUri != null &&
                                (request.RequestUri.AbsolutePath.Contains("/api/v1/auth/login") ||
                                 request.RequestUri.AbsolutePath.Contains("/api/v1/auth/register") ||
                                 request.RequestUri.AbsolutePath.Contains("/api/v1/auth/refresh"));

            // Capture session epoch and endpoint BEFORE sending original request
            var requestEpoch = _session.Epoch;
            var requestEndpoint = _endpointContext.CurrentEndpoint;

            if (!isAuthRequest && !string.IsNullOrEmpty(_session.AccessToken))
            {
                request.Headers.Authorization = new AuthenticationHeaderValue("Bearer", _session.AccessToken);
            }

            var response = await base.SendAsync(request, cancellationToken);

            if (response.StatusCode == HttpStatusCode.Unauthorized && !isAuthRequest)
            {
                // Do not attempt refresh or retry if the server or session changed during original request
                if (_session.Epoch != requestEpoch || !string.Equals(_endpointContext.CurrentEndpoint, requestEndpoint, StringComparison.OrdinalIgnoreCase))
                {
                    return response;
                }

                var refreshed = await TryRefreshTokenAsync(cancellationToken);
                if (refreshed)
                {
                    // Verify session epoch and endpoint STILL match requestEpoch before retrying original request
                    if (_session.Epoch != requestEpoch || !string.Equals(_endpointContext.CurrentEndpoint, requestEndpoint, StringComparison.OrdinalIgnoreCase))
                    {
                        return response;
                    }

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

            var currentBase = _endpointContext.CurrentEndpoint;
            if (string.IsNullOrEmpty(currentBase))
            {
                return false;
            }

            // Capture session epoch before starting refresh
            var capturedEpoch = _session.Epoch;

            await _refreshLock.WaitAsync(cancellationToken);
            try
            {
                // Check if session epoch or endpoint changed while waiting for lock
                if (_session.Epoch != capturedEpoch || !string.Equals(_endpointContext.CurrentEndpoint, currentBase, StringComparison.OrdinalIgnoreCase))
                {
                    return false;
                }

                // Recheck if another thread already refreshed under same epoch
                if (_session.RefreshToken != currentRt && !string.IsNullOrEmpty(_session.AccessToken))
                {
                    return true;
                }

                using var refreshClient = new HttpClient { BaseAddress = new Uri(currentBase) };
                var payload = new RefreshTokenRequestDto(new Option<string?>(currentRt));
                var json = JsonSerializer.Serialize(payload, new JsonSerializerOptions
                {
                    PropertyNamingPolicy = JsonNamingPolicy.CamelCase
                });

                var content = new StringContent(json, Encoding.UTF8, "application/json");
                var response = await refreshClient.PostAsync("/api/v1/auth/refresh", content, cancellationToken);

                // Check epoch again immediately after network call returns
                if (_session.Epoch != capturedEpoch || !string.Equals(_endpointContext.CurrentEndpoint, currentBase, StringComparison.OrdinalIgnoreCase))
                {
                    // Server switch or session clear happened while refresh was in-flight -> discard!
                    return false;
                }

                if (response.IsSuccessStatusCode)
                {
                    var responseJson = await response.Content.ReadAsStringAsync(cancellationToken);
                    using var doc = JsonDocument.Parse(responseJson);
                    var root = doc.RootElement;
                    var newAccessToken = root.GetProperty("access_token").GetString();
                    var newRefreshToken = root.GetProperty("refresh_token").GetString();

                    if (!string.IsNullOrEmpty(newAccessToken) && !string.IsNullOrEmpty(newRefreshToken))
                    {
                        return _session.TrySetTokensForEpoch(capturedEpoch, newAccessToken, newRefreshToken);
                    }
                }

                // If refresh failed (401 or expired token), clear session ONLY if epoch has not advanced
                if (_session.Epoch == capturedEpoch)
                {
                    _session.Clear();
                }
                return false;
            }
            catch
            {
                if (_session.Epoch == capturedEpoch)
                {
                    _session.Clear();
                }
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
