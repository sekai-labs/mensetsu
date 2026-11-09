
import Foundation
import MensetsuContracts

public enum AuthState: Equatable {
    case unauthenticated
    case requires2FA(tempToken: String)
    case authenticated(UserResponseDto)
}

public actor TokenManager {
    private var inMemoryAccessToken: String?
    private let keychainStore: SecureTokenStoring
    private var refreshTask: Task<String, Error>?
    private var epoch: UInt64 = 0
    public init(keychainStore: SecureTokenStoring) {
        self.keychainStore = keychainStore
    }

    public func setAccessToken(_ token: String?) {
        self.inMemoryAccessToken = token
    }

    public func getAccessToken() -> String? {
        return inMemoryAccessToken
    }

    public func setRefreshToken(_ token: String?) throws {
        if let token = token, !token.isEmpty {
            try keychainStore.saveRefreshToken(token)
        } else {
            try keychainStore.clearRefreshToken()
        }
    }

    public func getRefreshToken() throws -> String? {
        return try keychainStore.getRefreshToken()
    }

    public func clear() throws {
        epoch &+= 1
        inMemoryAccessToken = nil
        refreshTask?.cancel()
        refreshTask = nil
        try keychainStore.clearRefreshToken()
    }

    private func clearRefreshTask() {
        refreshTask = nil
    }

    public func refreshToken() async throws -> String {
        if let existingTask = refreshTask {
            return try await existingTask.value
        }

        guard let storedRefresh = try self.getRefreshToken(), !storedRefresh.isEmpty else {
            throw APIError.unauthorized("No refresh token stored")
        }

        let startEpoch = self.epoch

        let task = Task<String, Error> {
            try Task.checkCancellation()
            let requestDto = RefreshTokenRequestDto(refreshToken: storedRefresh)

            let response: TokenResponseDto = try await withCheckedThrowingContinuation { continuation in
                AuthAPI.refresh(refreshTokenRequestDto: requestDto) { tokenResponse, error in
                    if let error = error {
                        continuation.resume(throwing: error)
                        return
                    }
                    guard let tokenResponse = tokenResponse else {
                        continuation.resume(throwing: APIError.invalidResponse)
                        return
                    }
                    continuation.resume(returning: tokenResponse)
                }
            }

            try Task.checkCancellation()
            try await self.persistRefreshToken(response.refreshToken, expectedEpoch: startEpoch)
            return response.accessToken
        }

        refreshTask = task

        do {
            let newAccessToken = try await task.value
            guard self.isValidEpoch(startEpoch) else {
                throw APIError.unauthorized("Session cleared during token refresh")
            }
            self.inMemoryAccessToken = newAccessToken
            self.clearRefreshTask()
            return newAccessToken
        } catch {
            self.clearRefreshTask()
            if self.isValidEpoch(startEpoch) {
                try? self.clear()
            }
            throw error
        }
    }

    private func isValidEpoch(_ expectedEpoch: UInt64) -> Bool {
        return self.epoch == expectedEpoch
    }

    private func persistRefreshToken(_ token: String?, expectedEpoch: UInt64) throws {
        guard self.epoch == expectedEpoch else {
            throw APIError.unauthorized("Session cleared during token refresh")
        }
        if let token = token, !token.isEmpty {
            try keychainStore.saveRefreshToken(token)
        } else {
            try keychainStore.clearRefreshToken()
        }
    }
}

public enum APIError: Error, LocalizedError {
    case unauthorized(String)
    case invalidResponse
    case validation(String)
    case serverError(statusCode: Int, message: String)

    public var errorDescription: String? {
        switch self {
        case let .unauthorized(msg):
            return "Authentication required: \(msg)"
        case .invalidResponse:
            return "Invalid server response"
        case let .validation(message):
            return message
        case let .serverError(statusCode, message):
            return "Server returned error \(statusCode): \(message)"
        }
    }
}
