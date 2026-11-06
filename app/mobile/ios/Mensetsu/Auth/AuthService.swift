
import Foundation
import MensetsuContracts

public protocol AuthServiceProtocol {
    func login(email: String, password: String) async throws -> LoginResult
    func register(name: String, email: String, password: String) async throws -> UserResponseDto
    func login2FA(code: String, tempToken: String) async throws -> UserResponseDto
    func logout() async throws
    func clearSession() async throws
    func checkSession() async throws -> UserResponseDto?
}

public enum LoginResult {
    case success(UserResponseDto)
    case requires2FA(tempToken: String)
}

public final class AuthService: AuthServiceProtocol {
    private let client: NetworkClient

    public init(client: NetworkClient = .shared) {
        self.client = client
    }

    public func login(email: String, password: String) async throws -> LoginResult {
        let req = LoginRequestDto(email: email, password: password)

        let loginResponse: LoginResponseDto = try await withCheckedThrowingContinuation { continuation in
            AuthAPI.login(loginRequestDto: req) { resp, error in
                if let error = error {
                    continuation.resume(throwing: error)
                    return
                }
                guard let resp = resp else {
                    continuation.resume(throwing: APIError.invalidResponse)
                    return
                }
                continuation.resume(returning: resp)
            }
        }

        if loginResponse.requires2fa == true, let tempToken = loginResponse.tempToken {
            return .requires2FA(tempToken: tempToken)
        }

        guard let accessToken = loginResponse.accessToken,
              let user = loginResponse.user else {
            throw APIError.invalidResponse
        }

        await client.tokenManager.setAccessToken(accessToken)
        if let refreshToken = loginResponse.refreshToken {
            try await client.tokenManager.setRefreshToken(refreshToken)
        }

        return .success(user)
    }

    public func register(name: String, email: String, password: String) async throws -> UserResponseDto {
        let req = RegisterRequestDto(email: email, name: name, password: password)

        let authResponse: AuthResponseDto = try await withCheckedThrowingContinuation { continuation in
            AuthAPI.register(registerRequestDto: req) { resp, error in
                if let error = error {
                    continuation.resume(throwing: error)
                    return
                }
                guard let resp = resp else {
                    continuation.resume(throwing: APIError.invalidResponse)
                    return
                }
                continuation.resume(returning: resp)
            }
        }

        await client.tokenManager.setAccessToken(authResponse.accessToken)
        try await client.tokenManager.setRefreshToken(authResponse.refreshToken)

        return authResponse.user
    }

    public func login2FA(code: String, tempToken: String) async throws -> UserResponseDto {
        let req = Login2faRequestDto(code: code, tempToken: tempToken)

        let authResponse: AuthResponseDto = try await withCheckedThrowingContinuation { continuation in
            AuthAPI.login2fa(login2faRequestDto: req) { resp, error in
                if let error = error {
                    continuation.resume(throwing: error)
                    return
                }
                guard let resp = resp else {
                    continuation.resume(throwing: APIError.invalidResponse)
                    return
                }
                continuation.resume(returning: resp)
            }
        }

        await client.tokenManager.setAccessToken(authResponse.accessToken)
        try await client.tokenManager.setRefreshToken(authResponse.refreshToken)

        return authResponse.user
    }

    public func logout() async throws {
        var serverError: Error?
        do {
            try await client.executeWithAuthRetry { _ in
                try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
                    AuthAPI.logout { _, error in
                        if let error {
                            continuation.resume(throwing: error)
                        } else {
                            continuation.resume(returning: ())
                        }
                    }
                }
            }
        } catch {
            serverError = error
        }
        try await client.tokenManager.clear()
        MensetsuContractsAPI.customHeaders.removeValue(forKey: "Authorization")
        if let serverError { throw serverError }
    }

    public func clearSession() async throws {
        try await client.clearSession()
    }

    public func checkSession() async throws -> UserResponseDto? {
        do {
            return try await client.executeWithAuthRetry { _ in
                try await withCheckedThrowingContinuation { continuation in
                    AuthAPI.me { user, error in
                        if let error = error {
                            continuation.resume(throwing: error)
                            return
                        }
                        continuation.resume(returning: user)
                    }
                }
            }
        } catch {
            try? await client.tokenManager.clear()
            return nil
        }
    }
}
