//
// APIClient.swift
// Mensetsu macOS
//

import Foundation
import MensetsuContracts

public enum APIError: LocalizedError {
    case unauthorized
    case forbidden
    case notFound
    case serverError(Int, String)
    case networkError(Error)
    case invalidResponse
    case secureStorageUnavailable
    case challenge2faRequired(String)
    
    public var errorDescription: String? {
        switch self {
        case .unauthorized:
            return "Unauthorized. Please check your credentials or log in again."
        case .forbidden:
            return "Access denied."
        case .notFound:
            return "Resource not found."
        case .serverError(let code, let msg):
            return "Server error (\(code)): \(msg)"
        case .networkError(let err):
            return "Network connection error: \(err.localizedDescription)"
        case .invalidResponse:
            return "Invalid response received from server."
        case .secureStorageUnavailable:
            return "Cannot store the refresh token in Keychain."
        case .challenge2faRequired:
            return "Two-factor authentication code required."
        }
    }
}

public actor TokenStorage {
    public static let shared = TokenStorage()
    
    // Access token kept in memory only, tagged by current session epoch
    private var memoryAccessToken: String?
    private var currentEpoch: UInt64 = 0
    
    private init() {}
    
    public func setAccessToken(_ token: String?) {
        self.memoryAccessToken = token
    }
    
    public func getAccessToken() -> String? {
        return self.memoryAccessToken
    }
    
    public func commitTokensIfEpochMatches(epoch: UInt64, accessToken: String, refreshToken: String) throws -> Bool {
        guard self.currentEpoch == epoch else {
            return false
        }
        guard KeychainManager.shared.saveRefreshToken(refreshToken) else {
            throw APIError.secureStorageUnavailable
        }
        self.memoryAccessToken = accessToken
        return true
    }

    public func clearTokens(newEpoch: UInt64) throws {
        self.currentEpoch = newEpoch
        self.memoryAccessToken = nil
        guard KeychainManager.shared.deleteRefreshToken() else {
            throw APIError.secureStorageUnavailable
        }
    }
}

public final class APIClient {
    public static let shared = APIClient()
    
    private let session: URLSession
    private let s3Session: URLSession
    private let refreshLock = NSLock()
    private var refreshTask: Task<String, Error>?
    private var sessionEpoch: UInt64 = 0
    private var currentRefreshTaskId: UInt64 = 0
    private var nextRefreshTaskId: UInt64 = 0
    
    public var baseURL: String {
        let url = AppConfig.apiBaseUrl
        MensetsuContractsAPI.basePath = url
        return url
    }
    
    /// Resets the in-memory tokens, Keychain items, and contracts headers to isolate sessions across servers.
    /// Throws APIError.secureStorageUnavailable if Keychain deletion fails, ensuring no credential leaks.
    /// Cancels in-flight refresh tasks and increments session epoch to prevent any race condition
    /// from re-persisting old-server credentials.
    public func resetSession() async throws {
        let newEpoch: UInt64
        let inFlightTask: Task<String, Error>?

        refreshLock.lock()
        sessionEpoch &+= 1
        newEpoch = sessionEpoch
        inFlightTask = refreshTask
        refreshTask = nil
        currentRefreshTaskId = 0
        refreshLock.unlock()
        inFlightTask?.cancel()

        try await TokenStorage.shared.clearTokens(newEpoch: newEpoch)
        MensetsuContractsAPI.customHeaders.removeValue(forKey: "Authorization")
        MensetsuContractsAPI.basePath = baseURL
    }

    private final class NoRedirectDelegate: NSObject, URLSessionTaskDelegate {
        func urlSession(
            _ session: URLSession,
            task: URLSessionTask,
            willPerformHTTPRedirection response: HTTPURLResponse,
            newRequest request: URLRequest,
            completionHandler: @escaping (URLRequest?) -> Void
        ) {
            // Reject any redirection for direct S3 upload
            completionHandler(nil)
        }
    }
    
    private init() {
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 60
        self.session = URLSession(configuration: configuration)
        
        // Ephemeral URLSession strictly for direct S3 PUT with no cookie storage or credentials
        let s3Config = URLSessionConfiguration.ephemeral
        s3Config.httpCookieStorage = nil
        s3Config.httpShouldSetCookies = false
        s3Config.urlCredentialStorage = nil
        s3Config.timeoutIntervalForRequest = 60
        s3Config.timeoutIntervalForResource = 120
        self.s3Session = URLSession(configuration: s3Config, delegate: NoRedirectDelegate(), delegateQueue: nil)
        
        MensetsuContractsAPI.basePath = baseURL
    }
    
    // MARK: - Core Request Handling with serialized refresh and 1 retry
    
    public func request<T: Decodable>(
        endpoint: String,
        method: String = "GET",
        body: Data? = nil,
        headers: [String: String] = [:],
        requiresAuth: Bool = true
    ) async throws -> T {
        return try await executeWithRetry(endpoint: endpoint, method: method, body: body, headers: headers, requiresAuth: requiresAuth, isRetry: false)
    }
    
    public func requestVoid(
        endpoint: String,
        method: String = "POST",
        body: Data? = nil,
        headers: [String: String] = [:],
        requiresAuth: Bool = true
    ) async throws {
        _ = try await executeWithRetry(endpoint: endpoint, method: method, body: body, headers: headers, requiresAuth: requiresAuth, isRetry: false) as EmptyResponse
    }
    
    private struct EmptyResponse: Decodable {}
    
    private func executeWithRetry<T: Decodable>(
        endpoint: String,
        method: String,
        body: Data?,
        headers: [String: String],
        requiresAuth: Bool,
        isRetry: Bool
    ) async throws -> T {
        let cleanEndpoint = endpoint.hasPrefix("/") ? endpoint : "/\(endpoint)"
        guard let url = URL(string: "\(baseURL)\(cleanEndpoint)") else {
            throw APIError.invalidResponse
        }
        
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.httpBody = body
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        if body != nil {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        }
        
        for (k, v) in headers {
            request.setValue(v, forHTTPHeaderField: k)
        }
        
        if requiresAuth {
            let token = await TokenStorage.shared.getAccessToken()
            if let token = token, !token.isEmpty {
                request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
            }
        }
        
        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.networkError(error)
        }
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        
        // Handle 401 Unauthorized by serialized token refresh + retry once
        if httpResponse.statusCode == 401 && requiresAuth && !isRetry {
            do {
                _ = try await serializedRefreshToken()
                return try await executeWithRetry(
                    endpoint: endpoint,
                    method: method,
                    body: body,
                    headers: headers,
                    requiresAuth: requiresAuth,
                    isRetry: true
                )
            } catch {
                try await resetSession()
                throw APIError.unauthorized
            }
        }
        
        if httpResponse.statusCode == 401 {
            throw APIError.unauthorized
        }
        if httpResponse.statusCode == 403 {
            throw APIError.forbidden
        }
        if httpResponse.statusCode == 404 {
            throw APIError.notFound
        }
        if httpResponse.statusCode < 200 || httpResponse.statusCode >= 300 {
            let errorBody = String(data: data, encoding: .utf8) ?? "Unknown error"
            throw APIError.serverError(httpResponse.statusCode, errorBody)
        }
        
        if T.self == EmptyResponse.self {
            return EmptyResponse() as! T
        }
        
        do {
            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .custom { decoder in
                let container = try decoder.singleValueContainer()
                let dateStr = try container.decode(String.self)
                let isoFormatter = ISO8601DateFormatter()
                isoFormatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
                if let date = isoFormatter.date(from: dateStr) {
                    return date
                }
                isoFormatter.formatOptions = [.withInternetDateTime]
                if let date = isoFormatter.date(from: dateStr) {
                    return date
                }
                throw DecodingError.dataCorruptedError(in: container, debugDescription: "Cannot decode date: \(dateStr)")
            }
            return try decoder.decode(T.self, from: data)
        } catch {
            throw APIError.serverError(httpResponse.statusCode, "Failed to decode response: \(error.localizedDescription)")
        }
    }
    
    // MARK: - Serialized Refresh Token with Rotating Token Persistence
    
    public func serializedRefreshToken() async throws -> String {
        refreshLock.lock()
        let currentEpoch = sessionEpoch
        if let existingTask = refreshTask {
            refreshLock.unlock()
            return try await existingTask.value
        }
        
        nextRefreshTaskId &+= 1
        let taskId = nextRefreshTaskId
        currentRefreshTaskId = taskId

        let task = Task<String, Error> {
            defer {
                self.refreshLock.lock()
                // Only clear refreshTask if active task ID still matches this task
                if self.currentRefreshTaskId == taskId {
                    self.refreshTask = nil
                    self.currentRefreshTaskId = 0
                }
                self.refreshLock.unlock()
            }
            
            guard let currentRefreshToken = KeychainManager.shared.getRefreshToken(), !currentRefreshToken.isEmpty else {
                throw APIError.unauthorized
            }
            
            guard let url = URL(string: "\(self.baseURL)/api/v1/auth/refresh") else {
                throw APIError.invalidResponse
            }
            
            var request = URLRequest(url: url)
            request.httpMethod = "POST"
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.setValue("application/json", forHTTPHeaderField: "Accept")
            
            let refreshPayload = RefreshTokenRequestDto(refreshToken: currentRefreshToken)
            let encoder = JSONEncoder()
            request.httpBody = try encoder.encode(refreshPayload)
            
            let (data, response) = try await self.session.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse, (200..<300).contains(httpResponse.statusCode) else {
                throw APIError.unauthorized
            }
            let tokenResponse = try JSONDecoder().decode(TokenResponseDto.self, from: data)
            
            if Task.isCancelled {
                throw APIError.unauthorized
            }

            // Atomically check epoch inside TokenStorage actor:
            // If resetSession occurred at any point prior to commit, commit is rejected
            // and old tokens are never written to Keychain or memory.
            let committed = try await TokenStorage.shared.commitTokensIfEpochMatches(
                epoch: currentEpoch,
                accessToken: tokenResponse.accessToken,
                refreshToken: tokenResponse.refreshToken
            )

            guard committed, !Task.isCancelled else {
                throw APIError.unauthorized
            }

            // Final verify under refreshLock: if epoch bumped during actor write, purge immediately
            self.refreshLock.lock()
            let isStale = (self.sessionEpoch != currentEpoch)
            self.refreshLock.unlock()

            if isStale {
                _ = KeychainManager.shared.deleteRefreshToken()
                await TokenStorage.shared.setAccessToken(nil)
                MensetsuContractsAPI.customHeaders.removeValue(forKey: "Authorization")
                throw APIError.unauthorized
            }

            MensetsuContractsAPI.customHeaders["Authorization"] = "Bearer \(tokenResponse.accessToken)"
            return tokenResponse.accessToken
        }
        self.refreshTask = task
        refreshLock.unlock()
        return try await task.value
    }
    // MARK: - Auth API Methods
    
    public func register(dto: RegisterRequestDto) async throws -> AuthResponseDto {
        let encoder = JSONEncoder()
        let body = try encoder.encode(dto)
        let response: AuthResponseDto = try await request(
            endpoint: "/api/v1/auth/register",
            method: "POST",
            body: body,
            requiresAuth: false
        )
        
        guard KeychainManager.shared.saveRefreshToken(response.refreshToken) else {
            throw APIError.secureStorageUnavailable
        }
        await TokenStorage.shared.setAccessToken(response.accessToken)
        MensetsuContractsAPI.customHeaders["Authorization"] = "Bearer \(response.accessToken)"
        return response
    }
    
    public func login(dto: LoginRequestDto) async throws -> LoginResponseDto {
        let encoder = JSONEncoder()
        let body = try encoder.encode(dto)
        let response: LoginResponseDto = try await request(
            endpoint: "/api/v1/auth/login",
            method: "POST",
            body: body,
            requiresAuth: false
        )
        
        if response.requires2fa == true {
            return response
        }
        
        if let access = response.accessToken, let refresh = response.refreshToken {
            guard KeychainManager.shared.saveRefreshToken(refresh) else {
                throw APIError.secureStorageUnavailable
            }
            await TokenStorage.shared.setAccessToken(access)
            MensetsuContractsAPI.customHeaders["Authorization"] = "Bearer \(access)"
        }
        return response
    }
    
    public func login2fa(dto: Login2faRequestDto) async throws -> AuthResponseDto {
        let encoder = JSONEncoder()
        let body = try encoder.encode(dto)
        let response: AuthResponseDto = try await request(
            endpoint: "/api/v1/auth/login-2fa",
            method: "POST",
            body: body,
            requiresAuth: false
        )
        
        guard KeychainManager.shared.saveRefreshToken(response.refreshToken) else {
            throw APIError.secureStorageUnavailable
        }
        await TokenStorage.shared.setAccessToken(response.accessToken)
        MensetsuContractsAPI.customHeaders["Authorization"] = "Bearer \(response.accessToken)"
        return response
    }
    public func logout() async throws {
        var serverError: Error?
        do {
            try await requestVoid(
                endpoint: "/api/v1/auth/logout",
                method: "POST",
                requiresAuth: true
            )
        } catch {
            serverError = error
        }
        try await resetSession()
        if let serverError { throw serverError }
    }
    // MARK: - Workspaces API Methods
    
    public func listWorkspaces() async throws -> [WorkspaceResponseDto] {
        return try await request(endpoint: "/api/v1/workspaces", method: "GET", requiresAuth: true)
    }
    
    public func getWorkspace(id: String) async throws -> WorkspaceResponseDto {
        let cleanId = id.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(endpoint: "/api/v1/workspaces/\(cleanId)", method: "GET", requiresAuth: true)
    }
    
    public func createWorkspace(dto: CreateWorkspaceRequestDto) async throws -> WorkspaceResponseDto {
        let encoder = JSONEncoder()
        let body = try encoder.encode(dto)
        return try await request(
            endpoint: "/api/v1/workspaces",
            method: "POST",
            body: body,
            requiresAuth: true
        )
    }
    // MARK: - Resume Upload Methods
    
    public func createResumeUpload(filename: String, sizeBytes: Int64, contentType: String = "application/pdf") async throws -> CreateResumeUploadResponseDto {
        let req = CreateResumeUploadRequestDto(contentType: contentType, filename: filename, sizeBytes: sizeBytes)
        let encoder = JSONEncoder()
        let body = try encoder.encode(req)
        return try await request(
            endpoint: "/api/v1/resume-uploads",
            method: "POST",
            body: body,
            requiresAuth: true
        )
    }
    
    public func uploadPdfToPresignedUrl(url: URL, fileUrl: URL) async throws {
        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("application/pdf", forHTTPHeaderField: "Content-Type")
        request.httpShouldHandleCookies = false
        // Strictly NO Authorization header or session cookies for direct S3 upload
        
        let (_, response): (Data, URLResponse)
        do {
            (_, response) = try await s3Session.upload(for: request, fromFile: fileUrl)
        } catch {
            throw APIError.networkError(error)
        }
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        
        guard (200...299).contains(httpResponse.statusCode) else {
            throw APIError.serverError(httpResponse.statusCode, "S3 upload failed with status \(httpResponse.statusCode)")
        }
    }
    
    public func uploadPdfToPresignedUrl(url: URL, data: Data) async throws {
        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("application/pdf", forHTTPHeaderField: "Content-Type")
        request.httpShouldHandleCookies = false
        // Strictly NO Authorization header or session cookies for direct S3 upload
        
        let (_, response): (Data, URLResponse)
        do {
            (_, response) = try await s3Session.upload(for: request, from: data)
        } catch {
            throw APIError.networkError(error)
        }
        
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        
        guard (200...299).contains(httpResponse.statusCode) else {
            throw APIError.serverError(httpResponse.statusCode, "S3 upload failed with status \(httpResponse.statusCode)")
        }
    }
    
    public func completeResumeUpload(uploadId: String) async throws -> CompleteResumeUploadResponseDto {
        let cleanId = uploadId.replacingOccurrences(of: "resume_upload:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(
            endpoint: "/api/v1/resume-uploads/\(cleanId)/complete",
            method: "POST",
            requiresAuth: true
        )
    }
    
    
    public func getWorkspaceMatch(id: String) async throws -> MatchAnalysis {
        let cleanId = id.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(endpoint: "/api/v1/workspaces/\(cleanId)/match", method: "GET", requiresAuth: true)
    }
    
    public func getWorkspaceCompany(id: String) async throws -> CompanyResearch {
        let cleanId = id.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(endpoint: "/api/v1/workspaces/\(cleanId)/company", method: "GET", requiresAuth: true)
    }
    
    public func getWorkspaceLearning(id: String) async throws -> [LearningUnit] {
        let cleanId = id.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(endpoint: "/api/v1/workspaces/\(cleanId)/learning", method: "GET", requiresAuth: true)
    }
    
    public func toggleLearningUnitCompleted(workspaceId: String, lessonId: String) async throws {
        let cleanWsId = workspaceId.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        let cleanLessonId = lessonId.replacingOccurrences(of: "lesson:", with: "").replacingOccurrences(of: "`", with: "")
        try await requestVoid(
            endpoint: "/api/v1/workspaces/\(cleanWsId)/learning/\(cleanLessonId)/toggle",
            method: "PUT",
            requiresAuth: true
        )
    }
    
    public func getWorkspaceQuestions(id: String) async throws -> [InterviewQuestion] {
        let cleanId = id.replacingOccurrences(of: "workspace:", with: "").replacingOccurrences(of: "`", with: "")
        return try await request(endpoint: "/api/v1/workspaces/\(cleanId)/questions", method: "GET", requiresAuth: true)
    }
    
    // MARK: - LLM Settings API Methods
    
    public func getLlmSettings() async throws -> [LlmConfigItemDto] {
        return try await request(endpoint: "/api/v1/settings/llm", method: "GET", requiresAuth: true)
    }
    
    public func upsertLlmSetting(dto: UpsertLlmConfigRequestDto) async throws -> LlmConfigItemDto {
        let encoder = JSONEncoder()
        let body = try encoder.encode(dto)
        return try await request(
            endpoint: "/api/v1/settings/llm",
            method: "PUT",
            body: body,
            requiresAuth: true
        )
    }
    
    public func deleteLlmSetting(provider: String) async throws {
        try await requestVoid(
            endpoint: "/api/v1/settings/llm/\(provider)",
            method: "DELETE",
            requiresAuth: true
        )
    }
}
