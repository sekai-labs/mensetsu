
import Foundation
import MensetsuContracts

private actor HeaderGate {
    private var busy = false
    private var waiters: [CheckedContinuation<Void, Never>] = []

    func acquire() async {
        if busy {
            await withCheckedContinuation { waiters.append($0) }
        } else {
            busy = true
        }
    }

    func release() {
        if waiters.isEmpty {
            busy = false
        } else {
            waiters.removeFirst().resume()
        }
    }
}

public final class NetworkClient {
    public static let shared = NetworkClient()

    public let tokenManager: TokenManager
    public let keychainStore: SecureTokenStoring
    private let headerGate = HeaderGate()

    public init(
        keychainStore: SecureTokenStoring = KeychainStore(),
        baseUrl: String = AppConfig.apiBaseUrl
    ) {
        self.keychainStore = keychainStore
        self.tokenManager = TokenManager(keychainStore: keychainStore)
        MensetsuContractsAPI.basePath = baseUrl
    }

    public func setBaseUrl(_ url: String) {
        MensetsuContractsAPI.basePath = url
    }

    public func clearSession() async throws {
        try await tokenManager.clear()
        MensetsuContractsAPI.customHeaders.removeValue(forKey: "Authorization")
    }
    public func executeWithAuthRetry<T>(
        request: @escaping (_ accessToken: String) async throws -> T
    ) async throws -> T {
        var token = await tokenManager.getAccessToken()
        if token == nil {
            token = try await tokenManager.refreshToken()
        }

        guard let validToken = token else {
            throw APIError.unauthorized("Missing access token")
        }

        do {
            return try await executeWithToken(token: validToken, request: request)
        } catch {
            if isUnauthorized(error: error) {
                let newToken = try await tokenManager.refreshToken()
                return try await executeWithToken(token: newToken, request: request)
            } else {
                throw error
            }
        }
    }

    private func executeWithToken<T>(
        token: String,
        request: @escaping (_ accessToken: String) async throws -> T
    ) async throws -> T {
        await headerGate.acquire()
        MensetsuContractsAPI.customHeaders["Authorization"] = "Bearer \(token)"
        do {
            let result = try await request(token)
            await headerGate.release()
            return result
        } catch {
            await headerGate.release()
            throw error
        }
    }

    private func isUnauthorized(error: Error) -> Bool {
        if let errorResp = error as? ErrorResponse {
            switch errorResp {
            case let .error(statusCode, _, _, _):
                return statusCode == 401
            }
        }
        let nsError = error as NSError
        return nsError.code == 401
    }
}
