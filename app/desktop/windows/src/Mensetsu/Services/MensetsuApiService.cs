using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using Mensetsu.Contracts.Client;
using Mensetsu.Contracts.Model;

namespace Mensetsu.Services
{
    public interface IMensetsuApiService
    {
        // Auth
        Task<LoginResponseDto> LoginAsync(string email, string password, CancellationToken cancellationToken = default);
        Task<AuthResponseDto> RegisterAsync(string name, string email, string password, CancellationToken cancellationToken = default);
        Task<AuthResponseDto> Login2faAsync(string code, string tempToken, CancellationToken cancellationToken = default);
        Task<bool> RefreshAsync(CancellationToken cancellationToken = default);
        Task LogoutAsync(CancellationToken cancellationToken = default);
        Task<UserResponseDto?> GetMeAsync(CancellationToken cancellationToken = default);

        // Workspaces
        Task<List<WorkspaceResponseDto>> ListWorkspacesAsync(CancellationToken cancellationToken = default);
        Task<WorkspaceResponseDto> GetWorkspaceAsync(string id, CancellationToken cancellationToken = default);
        Task<WorkspaceResponseDto> CreateWorkspaceAsync(string companyName, string roleTitle, string? resumeText, string? resumeUploadId, string jobDescription, CancellationToken cancellationToken = default);

        // Resume uploads
        Task<CreateResumeUploadResponseDto> CreateResumeUploadAsync(string filename, long sizeBytes, string contentType = "application/pdf", CancellationToken cancellationToken = default);
        Task UploadResumePdfBytesAsync(string uploadUrl, byte[] pdfBytes, CancellationToken cancellationToken = default);
        Task<CompleteResumeUploadResponseDto> CompleteResumeUploadAsync(string uploadId, CancellationToken cancellationToken = default);

        // Details
        Task<MatchAnalysis?> GetWorkspaceMatchAsync(string id, CancellationToken cancellationToken = default);
        Task<CompanyResearch?> GetWorkspaceCompanyAsync(string id, CancellationToken cancellationToken = default);
        Task<List<LearningUnit>> GetWorkspaceLearningAsync(string id, CancellationToken cancellationToken = default);
        Task<bool> ToggleLearningUnitAsync(string workspaceId, string lessonId, CancellationToken cancellationToken = default);
        Task<List<InterviewQuestion>> GetWorkspaceQuestionsAsync(string id, CancellationToken cancellationToken = default);

        // Settings
        Task<List<LlmConfigItemDto>> GetLlmSettingsAsync(CancellationToken cancellationToken = default);
        Task<LlmConfigItemDto> UpsertLlmSettingAsync(string provider, string apiKey, CancellationToken cancellationToken = default);
    }

    public class MensetsuApiService : IMensetsuApiService
    {
        private readonly HttpClient _httpClient;
        private readonly IAuthSession _session;
        private readonly JsonSerializerOptions _jsonOptions;

        public MensetsuApiService(HttpClient httpClient, IAuthSession session)
        {
            _httpClient = httpClient;
            _session = session;
            _jsonOptions = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true,
                PropertyNamingPolicy = JsonNamingPolicy.CamelCase
            };
        }

        private static string CleanId(string id) => id.Replace("workspace:", "").Replace("`", "");

        public async Task<LoginResponseDto> LoginAsync(string email, string password, CancellationToken cancellationToken = default)
        {
            var req = new LoginRequestDto(email, password);
            var content = new StringContent(JsonSerializer.Serialize(req, _jsonOptions), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/auth/login", content, cancellationToken);
            var responseBody = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var errorMsg = ExtractErrorMessage(responseBody) ?? $"Login failed ({response.StatusCode})";
                throw new InvalidOperationException(errorMsg);
            }

            var result = JsonSerializer.Deserialize<LoginResponseDto>(responseBody, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse login response.");

            if (result.Requires2fa != true && !string.IsNullOrEmpty(result.AccessToken) && !string.IsNullOrEmpty(result.RefreshToken))
            {
                _session.SetTokens(result.AccessToken, result.RefreshToken, result.User);
            }

            return result;
        }

        public async Task<AuthResponseDto> RegisterAsync(string name, string email, string password, CancellationToken cancellationToken = default)
        {
            var req = new RegisterRequestDto(email, name, password);
            var content = new StringContent(JsonSerializer.Serialize(req, _jsonOptions), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/auth/register", content, cancellationToken);
            var responseBody = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var errorMsg = ExtractErrorMessage(responseBody) ?? $"Registration failed ({response.StatusCode})";
                throw new InvalidOperationException(errorMsg);
            }

            var result = JsonSerializer.Deserialize<AuthResponseDto>(responseBody, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse registration response.");

            if (!string.IsNullOrEmpty(result.AccessToken) && !string.IsNullOrEmpty(result.RefreshToken))
            {
                _session.SetTokens(result.AccessToken, result.RefreshToken, result.User);
            }

            return result;
        }

        public async Task<AuthResponseDto> Login2faAsync(string code, string tempToken, CancellationToken cancellationToken = default)
        {
            var req = new Login2faRequestDto(code, tempToken);
            var content = new StringContent(JsonSerializer.Serialize(req, _jsonOptions), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/auth/login-2fa", content, cancellationToken);
            var responseBody = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var errorMsg = ExtractErrorMessage(responseBody) ?? $"2FA verification failed ({response.StatusCode})";
                throw new InvalidOperationException(errorMsg);
            }

            var result = JsonSerializer.Deserialize<AuthResponseDto>(responseBody, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse 2FA response.");

            if (!string.IsNullOrEmpty(result.AccessToken) && !string.IsNullOrEmpty(result.RefreshToken))
            {
                _session.SetTokens(result.AccessToken, result.RefreshToken, result.User);
            }

            return result;
        }

        public async Task<bool> RefreshAsync(CancellationToken cancellationToken = default)
        {
            var rt = _session.RefreshToken;
            if (string.IsNullOrEmpty(rt)) return false;

            var req = new RefreshTokenRequestDto(new Option<string?>(rt));
            var content = new StringContent(JsonSerializer.Serialize(req, _jsonOptions), Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/auth/refresh", content, cancellationToken);

            if (response.IsSuccessStatusCode)
            {
                var responseBody = await response.Content.ReadAsStringAsync(cancellationToken);
                var tokenDto = JsonSerializer.Deserialize<TokenResponseDto>(responseBody, _jsonOptions);
                if (tokenDto != null && !string.IsNullOrEmpty(tokenDto.AccessToken) && !string.IsNullOrEmpty(tokenDto.RefreshToken))
                {
                    _session.SetTokens(tokenDto.AccessToken, tokenDto.RefreshToken);
                    return true;
                }
            }

            return false;
        }

        public async Task LogoutAsync(CancellationToken cancellationToken = default)
        {
            try
            {
                await _httpClient.PostAsync("/api/v1/auth/logout", null, cancellationToken);
            }
            catch
            {
                // Continue logout locally regardless of network outcome
            }
            finally
            {
                _session.Clear();
            }
        }

        public async Task<UserResponseDto?> GetMeAsync(CancellationToken cancellationToken = default)
        {
            var response = await _httpClient.GetAsync("/api/v1/auth/me", cancellationToken);
            if (!response.IsSuccessStatusCode) return null;

            var body = await response.Content.ReadAsStringAsync(cancellationToken);
            return JsonSerializer.Deserialize<UserResponseDto>(body, _jsonOptions);
        }

        public async Task<List<WorkspaceResponseDto>> ListWorkspacesAsync(CancellationToken cancellationToken = default)
        {
            var response = await _httpClient.GetAsync("/api/v1/workspaces", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load workspaces ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<List<WorkspaceResponseDto>>(body, _jsonOptions) ?? new List<WorkspaceResponseDto>();
        }

        public async Task<WorkspaceResponseDto> GetWorkspaceAsync(string id, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(id);
            var response = await _httpClient.GetAsync($"/api/v1/workspaces/{clean}", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Workspace not found ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<WorkspaceResponseDto>(body, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse workspace.");
        }

        public async Task<CreateResumeUploadResponseDto> CreateResumeUploadAsync(string filename, long sizeBytes, string contentType = "application/pdf", CancellationToken cancellationToken = default)
        {
            var req = new CreateResumeUploadRequestDto(
                contentType: contentType,
                filename: filename,
                sizeBytes: sizeBytes
            );

            var json = JsonSerializer.Serialize(req, _jsonOptions);
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/resume-uploads", content, cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to initialize resume upload ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<CreateResumeUploadResponseDto>(body, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse resume upload response.");
        }

        public async Task UploadResumePdfBytesAsync(string uploadUrl, byte[] pdfBytes, CancellationToken cancellationToken = default)
        {
            // Direct signed URL HTTP PUT:
            // - Use uploadUrl verbatim (Host signature binds, do NOT rewrite host)
            // - Content-Type exactly application/pdf
            // - Binary byte body
            // - NO Rust API Authorization header or cookies
            using var s3Handler = new HttpClientHandler { UseCookies = false, AllowAutoRedirect = false };
            using var s3Client = new HttpClient(s3Handler) { Timeout = TimeSpan.FromMinutes(2) };

            using var content = new ByteArrayContent(pdfBytes);
            content.Headers.ContentType = new System.Net.Http.Headers.MediaTypeHeaderValue("application/pdf");

            using var request = new HttpRequestMessage(HttpMethod.Put, uploadUrl)
            {
                Content = content
            };

            var response = await s3Client.SendAsync(request, cancellationToken);
            if (!response.IsSuccessStatusCode)
            {
                var responseBody = await response.Content.ReadAsStringAsync(cancellationToken);
                var err = ExtractErrorMessage(responseBody) ?? $"Failed to upload PDF bytes to storage ({response.StatusCode})";
                throw new InvalidOperationException(err);
            }
        }

        public async Task<CompleteResumeUploadResponseDto> CompleteResumeUploadAsync(string uploadId, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(uploadId).Replace("resume_upload:", "");
            var emptyContent = new StringContent("{}", Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync($"/api/v1/resume-uploads/{clean}/complete", emptyContent, cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to complete resume upload ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<CompleteResumeUploadResponseDto>(body, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse upload completion response.");
        }

        public async Task<WorkspaceResponseDto> CreateWorkspaceAsync(string companyName, string roleTitle, string? resumeText, string? resumeUploadId, string jobDescription, CancellationToken cancellationToken = default)
        {
            var req = new CreateWorkspaceRequestDto(
                companyName: new Option<string?>(companyName),
                roleTitle: new Option<string?>(roleTitle),
                resumeText: string.IsNullOrWhiteSpace(resumeText) ? default : new Option<string?>(resumeText),
                resumeUploadId: string.IsNullOrWhiteSpace(resumeUploadId) ? default : new Option<string?>(resumeUploadId),
                jobDescription: new Option<string?>(jobDescription)
            );

            var json = JsonSerializer.Serialize(req, _jsonOptions);
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PostAsync("/api/v1/workspaces", content, cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to create workspace ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<WorkspaceResponseDto>(body, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse created workspace.");
        }

        public async Task<MatchAnalysis?> GetWorkspaceMatchAsync(string id, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(id);
            var response = await _httpClient.GetAsync($"/api/v1/workspaces/{clean}/match", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load match analysis ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            try
            {
                return JsonSerializer.Deserialize<MatchAnalysis>(body, _jsonOptions);
            }
            catch
            {
                // Return null if empty or unpopulated
                return null;
            }
        }

        public async Task<CompanyResearch?> GetWorkspaceCompanyAsync(string id, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(id);
            var response = await _httpClient.GetAsync($"/api/v1/workspaces/{clean}/company", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load company research ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            try
            {
                return JsonSerializer.Deserialize<CompanyResearch>(body, _jsonOptions);
            }
            catch
            {
                return null;
            }
        }

        public async Task<List<LearningUnit>> GetWorkspaceLearningAsync(string id, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(id);
            var response = await _httpClient.GetAsync($"/api/v1/workspaces/{clean}/learning", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load learning units ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<List<LearningUnit>>(body, _jsonOptions) ?? new List<LearningUnit>();
        }

        public async Task<bool> ToggleLearningUnitAsync(string workspaceId, string lessonId, CancellationToken cancellationToken = default)
        {
            var cleanWs = CleanId(workspaceId);
            var cleanLesson = CleanId(lessonId);
            var response = await _httpClient.PutAsync($"/api/v1/workspaces/{cleanWs}/learning/{cleanLesson}/toggle", null, cancellationToken);
            return response.IsSuccessStatusCode;
        }

        public async Task<List<InterviewQuestion>> GetWorkspaceQuestionsAsync(string id, CancellationToken cancellationToken = default)
        {
            var clean = CleanId(id);
            var response = await _httpClient.GetAsync($"/api/v1/workspaces/{clean}/questions", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load interview questions ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<List<InterviewQuestion>>(body, _jsonOptions) ?? new List<InterviewQuestion>();
        }

        public async Task<List<LlmConfigItemDto>> GetLlmSettingsAsync(CancellationToken cancellationToken = default)
        {
            var response = await _httpClient.GetAsync("/api/v1/settings/llm", cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to load LLM settings ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<List<LlmConfigItemDto>>(body, _jsonOptions) ?? new List<LlmConfigItemDto>();
        }

        public async Task<LlmConfigItemDto> UpsertLlmSettingAsync(string provider, string apiKey, CancellationToken cancellationToken = default)
        {
            var req = new UpsertLlmConfigRequestDto(
                provider: provider,
                apiKey: new Option<string?>(apiKey)
            );

            var json = JsonSerializer.Serialize(req, _jsonOptions);
            var content = new StringContent(json, Encoding.UTF8, "application/json");
            var response = await _httpClient.PutAsync("/api/v1/settings/llm", content, cancellationToken);
            var body = await response.Content.ReadAsStringAsync(cancellationToken);

            if (!response.IsSuccessStatusCode)
            {
                var error = ExtractErrorMessage(body) ?? $"Failed to update LLM setting ({response.StatusCode})";
                throw new InvalidOperationException(error);
            }

            return JsonSerializer.Deserialize<LlmConfigItemDto>(body, _jsonOptions)
                ?? throw new InvalidOperationException("Failed to parse updated LLM config.");
        }

        private static string? ExtractErrorMessage(string json)
        {
            if (string.IsNullOrWhiteSpace(json)) return null;
            try
            {
                using var doc = JsonDocument.Parse(json);
                if (doc.RootElement.TryGetProperty("message", out var msg)) return msg.GetString();
                if (doc.RootElement.TryGetProperty("error", out var err)) return err.GetString();
                if (doc.RootElement.TryGetProperty("details", out var det)) return det.GetString();
            }
            catch
            {
                // Not a json message
            }
            return null;
        }
    }
}
