//
// AuthSession.swift
// Mensetsu
//

import Foundation
import Combine
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
        inMemoryAccessToken = nil
        refreshTask?.cancel()
        refreshTask = nil
        try keychainStore.clearRefreshToken()
    }

    private func clearRefreshTask() {
        refreshTask = nil
    }

    /// Serialized refresh operation: if multiple calls arrive concurrently,
    /// all await the same in-flight refresh Task.
    public func refreshToken() async throws -> String {
        if let existingTask = refreshTask {
            return try await existingTask.value
        }

        guard let storedRefresh = try self.getRefreshToken(), !storedRefresh.isEmpty else {
            throw APIError.unauthorized("No refresh token stored")
        }

        let task = Task<String, Error> {
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

            // Securely persist rotated refresh token
            try await self.setRefreshToken(response.refreshToken)
            return response.accessToken
        }

        refreshTask = task

        do {
            let newAccessToken = try await task.value
            self.inMemoryAccessToken = newAccessToken
            self.clearRefreshTask()
            return newAccessToken
        } catch {
            self.clearRefreshTask()
            try? self.clear()
            throw error
        }
    }
}

public enum APIError: Error, LocalizedError {
    case unauthorized(String)
    case invalidResponse
    case validation(String)
    case networkError(String)
    case serverError(statusCode: Int, message: String)

    public var errorDescription: String? {
        switch self {
        case let .unauthorized(msg):
            return "Authentication required: \(msg)"
        case .invalidResponse:
            return "Invalid server response"
        case let .validation(message):
            return message
        case let .networkError(msg):
            return "Network connection error: \(msg)"
        case let .serverError(statusCode, message):
            return "Server returned error \(statusCode): \(message)"
        }
    }
}
