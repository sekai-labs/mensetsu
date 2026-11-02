//
// AuthViewModel.swift
// Mensetsu
//

import Foundation
import Combine
import MensetsuContracts

@MainActor
public final class AuthViewModel: ObservableObject {
    @Published public var authState: AuthState = .unauthenticated
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String? = nil

    private let authService: AuthServiceProtocol

    public init(authService: AuthServiceProtocol = AuthService()) {
        self.authService = authService
    }

    public func checkExistingSession() async {
        isLoading = true
        errorMessage = nil
        do {
            if let user = try await authService.checkSession() {
                authState = .authenticated(user)
            } else {
                authState = .unauthenticated
            }
        } catch {
            authState = .unauthenticated
        }
        isLoading = false
    }

    public func login(email: String, password: String) async {
        isLoading = true
        errorMessage = nil
        do {
            let result = try await authService.login(email: email, password: password)
            switch result {
            case let .success(user):
                authState = .authenticated(user)
            case let .requires2FA(tempToken):
                authState = .requires2FA(tempToken: tempToken)
            }
        } catch {
            errorMessage = error.localizedDescription
            authState = .unauthenticated
        }
        isLoading = false
    }

    public func register(name: String, email: String, password: String) async {
        isLoading = true
        errorMessage = nil
        do {
            let user = try await authService.register(name: name, email: email, password: password)
            authState = .authenticated(user)
        } catch {
            errorMessage = error.localizedDescription
            authState = .unauthenticated
        }
        isLoading = false
    }

    public func verify2FA(code: String, tempToken: String) async {
        isLoading = true
        errorMessage = nil
        do {
            let user = try await authService.login2FA(code: code, tempToken: tempToken)
            authState = .authenticated(user)
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    public func cancel2FA() {
        authState = .unauthenticated
        errorMessage = nil
    }

    public func logout() async {
        isLoading = true
        do {
            try await authService.logout()
        } catch {
            // Still clear local auth state
        }
        authState = .unauthenticated
        errorMessage = nil
        isLoading = false
    }
}
