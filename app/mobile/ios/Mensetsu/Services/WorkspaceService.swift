
import Foundation
import MensetsuContracts

public protocol WorkspaceServiceProtocol {
    func listWorkspaces() async throws -> [WorkspaceResponseDto]
    func createWorkspace(companyName: String, roleTitle: String, resumeText: String, jobDescription: String) async throws -> WorkspaceResponseDto
    func createWorkspaceWithPDF(companyName: String, roleTitle: String, jobDescription: String, fileURL: URL) async throws -> WorkspaceResponseDto
    func getWorkspace(id: String) async throws -> WorkspaceResponseDto
    func getWorkspaceMatch(id: String) async throws -> MatchAnalysis?
    func getWorkspaceCompany(id: String) async throws -> CompanyResearch?
    func getWorkspaceQuestions(id: String) async throws -> [InterviewQuestion]
    func getWorkspaceLearning(id: String) async throws -> [LearningUnit]
    func toggleLearningUnit(id: String, lessonId: String) async throws
    func getLlmSettings() async throws -> [LlmConfigItemDto]
    func updateLlmSetting(provider: String, apiKey: String, isShared: Bool) async throws -> LlmConfigItemDto
    func getTokenUsage(cursor: TokenUsageCursorDto?) async throws -> TokenUsageResponseDto
}

public final class WorkspaceService: WorkspaceServiceProtocol {
    private let client: NetworkClient

    public init(client: NetworkClient = .shared) {
        self.client = client
    }

    public func listWorkspaces() async throws -> [WorkspaceResponseDto] {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesAPI.listWorkspaces { workspaces, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    continuation.resume(returning: workspaces ?? [])
                }
            }
        }
    }

    public func createWorkspace(companyName: String, roleTitle: String, resumeText: String, jobDescription: String) async throws -> WorkspaceResponseDto {
        let request = CreateWorkspaceRequestDto(
            companyName: companyName,
            jobDescription: jobDescription,
            resumeText: resumeText,
            roleTitle: roleTitle
        )
        return try await createWorkspace(request: request)
    }

    public func createWorkspaceWithPDF(
        companyName: String,
        roleTitle: String,
        jobDescription: String,
        fileURL: URL
    ) async throws -> WorkspaceResponseDto {
        let filename = fileURL.lastPathComponent
        guard filename.count > 4, filename.lowercased().hasSuffix(".pdf") else {
            throw APIError.validation("Choose a PDF résumé.")
        }
        let hasAccess = fileURL.startAccessingSecurityScopedResource()
        defer { if hasAccess { fileURL.stopAccessingSecurityScopedResource() } }
        let size = try fileURL.resourceValues(forKeys: [.fileSizeKey]).fileSize ?? 0
        guard (1...5_242_880).contains(size) else {
            throw APIError.validation("Résumé PDF must be between 1 byte and 5 MiB.")
        }

        let presign = CreateResumeUploadRequestDto(
            contentType: "application/pdf", filename: filename, sizeBytes: Int64(size)
        )
        let upload: CreateResumeUploadResponseDto = try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                ResumeUploadsAPI.createResumeUpload(createResumeUploadRequestDto: presign) { result, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let result = result {
                        continuation.resume(returning: result)
                    } else {
                        continuation.resume(throwing: APIError.invalidResponse)
                    }
                }
            }
        }

        guard let signedURL = URL(string: upload.uploadUrl) else {
            throw APIError.invalidResponse
        }
        var put = URLRequest(url: signedURL)
        put.httpMethod = "PUT"
        put.httpShouldHandleCookies = false
        put.setValue("application/pdf", forHTTPHeaderField: "Content-Type")
        let configuration = URLSessionConfiguration.ephemeral
        configuration.httpCookieStorage = nil
        configuration.timeoutIntervalForRequest = 60
        let session = URLSession(configuration: configuration, delegate: RejectStorageRedirects(), delegateQueue: nil)
        defer { session.invalidateAndCancel() }
        let (_, response) = try await session.upload(for: put, fromFile: fileURL)
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.invalidResponse
        }
        guard (200...299).contains(httpResponse.statusCode) else {
            throw APIError.serverError(statusCode: httpResponse.statusCode, message: "PDF upload to storage failed")
        }

        let completed: CompleteResumeUploadResponseDto = try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                ResumeUploadsAPI.completeResumeUpload(uploadId: upload.uploadId) { result, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let result = result {
                        continuation.resume(returning: result)
                    } else {
                        continuation.resume(throwing: APIError.invalidResponse)
                    }
                }
            }
        }
        guard completed.status == "completed" else {
            throw APIError.invalidResponse
        }
        let request = CreateWorkspaceRequestDto(
            companyName: companyName,
            jobDescription: jobDescription,
            resumeUploadId: upload.uploadId,
            roleTitle: roleTitle
        )
        return try await createWorkspace(request: request)
    }

    private func createWorkspace(request: CreateWorkspaceRequestDto) async throws -> WorkspaceResponseDto {
        try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesAPI.createWorkspace(createWorkspaceRequestDto: request) { response, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let response = response {
                        continuation.resume(returning: response)
                    } else {
                        continuation.resume(throwing: APIError.invalidResponse)
                    }
                }
            }
        }
    }

    public func getWorkspace(id: String) async throws -> WorkspaceResponseDto {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesAPI.getWorkspace(id: id) { ws, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    guard let ws = ws else {
                        continuation.resume(throwing: APIError.invalidResponse)
                        return
                    }
                    continuation.resume(returning: ws)
                }
            }
        }
    }

    public func getWorkspaceMatch(id: String) async throws -> MatchAnalysis? {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesLearningHandlersAPI.getWorkspaceMatch(id: id) { codableResp, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    guard let value = codableResp?.value, !(value is NSNull) else {
                        continuation.resume(returning: nil)
                        return
                    }
                    do {
                        let data = try JSONSerialization.data(withJSONObject: value)
                        let match = try JSONDecoder().decode(MatchAnalysis.self, from: data)
                        continuation.resume(returning: match)
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }
        }
    }

    public func getWorkspaceCompany(id: String) async throws -> CompanyResearch? {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesResearchHandlersAPI.getWorkspaceCompany(id: id) { codableResp, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    guard let value = codableResp?.value, !(value is NSNull) else {
                        continuation.resume(returning: nil)
                        return
                    }
                    do {
                        let data = try JSONSerialization.data(withJSONObject: value)
                        let comp = try JSONDecoder().decode(CompanyResearch.self, from: data)
                        continuation.resume(returning: comp)
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }
        }
    }

    public func getWorkspaceQuestions(id: String) async throws -> [InterviewQuestion] {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesLearningHandlersAPI.getWorkspaceQuestions(id: id) { questions, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    continuation.resume(returning: questions ?? [])
                }
            }
        }
    }

    public func getWorkspaceLearning(id: String) async throws -> [LearningUnit] {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesLearningHandlersAPI.getWorkspaceLearning(id: id) { lessons, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    continuation.resume(returning: lessons ?? [])
                }
            }
        }
    }

    public func toggleLearningUnit(id: String, lessonId: String) async throws {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                WorkspacesAPI.toggleLearningUnitCompleted(id: id, lessonId: lessonId) { _, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    continuation.resume(returning: ())
                }
            }
        }
    }

    public func getLlmSettings() async throws -> [LlmConfigItemDto] {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                SettingsAPI.getLlmSettings { settings, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    continuation.resume(returning: settings ?? [])
                }
            }
        }
    }

    public func updateLlmSetting(provider: String, apiKey: String, isShared: Bool) async throws -> LlmConfigItemDto {
        let req = UpsertLlmConfigRequestDto(
            apiKey: apiKey,
            isShared: isShared,
            provider: provider
        )

        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                SettingsAPI.upsertLlmSetting(upsertLlmConfigRequestDto: req) { updated, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    guard let updated = updated else {
                        continuation.resume(throwing: APIError.invalidResponse)
                        return
                    }
                    continuation.resume(returning: updated)
                }
            }
        }
    }
    public func getTokenUsage(cursor: TokenUsageCursorDto?) async throws -> TokenUsageResponseDto {
        return try await client.executeWithAuthRetry { _ in
            try await withCheckedThrowingContinuation { continuation in
                TokenUsageAPI.userTokenUsage(
                    beforeTimestamp: cursor?.beforeTimestamp,
                    beforeEventId: cursor?.beforeEventId
                ) { response, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                    } else if let response = response {
                        continuation.resume(returning: response)
                    } else {
                        continuation.resume(throwing: APIError.invalidResponse)
                    }
                }
            }
        }
    }
}

private final class RejectStorageRedirects: NSObject, URLSessionTaskDelegate {
    func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        willPerformHTTPRedirection response: HTTPURLResponse,
        newRequest request: URLRequest,
        completionHandler: @escaping (URLRequest?) -> Void
    ) {
        completionHandler(nil)
    }
}
