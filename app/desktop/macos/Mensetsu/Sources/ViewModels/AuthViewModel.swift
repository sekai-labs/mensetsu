//
// AuthViewModel.swift
// Mensetsu macOS
//

import Foundation
import Combine
import MensetsuContracts

@MainActor
public final class AuthViewModel: ObservableObject {
    @Published public var isServerConfigured: Bool = AppConfig.isConfigured
    @Published public var serverUrlInput: String = ""
    @Published public var serverUrlError: String?

    @Published public var isAuthenticated: Bool = false
    @Published public var currentUser: UserResponseDto?
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String?
    
    // Auth mode: login vs register
    @Published public var isRegisterMode: Bool = false
    
    // Fields
    @Published public var email: String = ""
    @Published public var password: String = ""
    @Published public var name: String = ""
    
    // 2FA Challenge state
    @Published public var requires2fa: Bool = false
    @Published public var challengeToken: String?
    @Published public var totpCode: String = ""
    
    // Settings Sheet presentation
    @Published public var isShowingSettings: Bool = false

    public init() {
        if AppConfig.isConfigured {
            checkExistingAuth()
        } else {
            self.isAuthenticated = false
        }
    }

    public func saveServerEndpoint() async -> Bool {
        serverUrlError = nil
        do {
            let validated = try AppConfig.normalizeAndValidateUrl(serverUrlInput)
            // Clear any lingering tokens before saving URL so no stale credentials reach the newly configured server
            try await APIClient.shared.resetSession()
            AppConfig.savedServerUrl = validated
            isServerConfigured = true
            isAuthenticated = false
            currentUser = nil
            serverUrlInput = ""
            clearForm()
            return true
        } catch {
            serverUrlError = error.localizedDescription
            return false
        }
    }

    public func changeServerEndpoint(to newUrl: String) async throws {
        let validated = try AppConfig.normalizeAndValidateUrl(newUrl)
        // Clear current server session and tokens first. If secure storage fails,
        // throws error and blocks server switch to prevent credential leakage.
        try await APIClient.shared.resetSession()
        AppConfig.savedServerUrl = validated
        isServerConfigured = true
        isAuthenticated = false
        currentUser = nil
        clearForm()
    }
    
    public func checkExistingAuth() {
        guard AppConfig.isConfigured else {
            self.isAuthenticated = false
            return
        }
        if KeychainManager.shared.getRefreshToken() != nil {
            Task {
                do {
                    _ = try await APIClient.shared.serializedRefreshToken()
                    self.isAuthenticated = true
                } catch {
                    self.isAuthenticated = false
                }
            }
        } else {
            self.isAuthenticated = false
        }
    }
    
    public func submitAuth() {
        guard AppConfig.isConfigured else {
            errorMessage = "Server endpoint is not configured."
            return
        }
        errorMessage = nil
        guard !email.trimmingCharacters(in: .whitespaces).isEmpty else {
            errorMessage = "Email cannot be empty."
            return
        }
        guard !password.trimmingCharacters(in: .whitespaces).isEmpty else {
            errorMessage = "Password cannot be empty."
            return
        }
        if isRegisterMode && name.trimmingCharacters(in: .whitespaces).isEmpty {
            errorMessage = "Name cannot be empty."
            return
        }
        
        isLoading = true
        Task {
            do {
                if isRegisterMode {
                    let req = RegisterRequestDto(
                        email: email.trimmingCharacters(in: .whitespaces),
                        name: name.trimmingCharacters(in: .whitespaces),
                        password: password
                    )
                    let auth = try await APIClient.shared.register(dto: req)
                    self.currentUser = auth.user
                    self.isAuthenticated = true
                    self.isLoading = false
                    self.clearForm()
                } else {
                    let req = LoginRequestDto(
                        email: email.trimmingCharacters(in: .whitespaces),
                        password: password
                    )
                    let loginRes = try await APIClient.shared.login(dto: req)
                    if loginRes.requires2fa == true {
                        self.requires2fa = true
                        self.challengeToken = loginRes.tempToken
                        self.isLoading = false
                    } else {
                        self.currentUser = loginRes.user
                        self.isAuthenticated = true
                        self.isLoading = false
                        self.clearForm()
                    }
                }
            } catch {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
    
    public func submit2faChallenge() {
        guard AppConfig.isConfigured else {
            errorMessage = "Server endpoint is not configured."
            return
        }
        errorMessage = nil
        guard let token = challengeToken, !token.isEmpty else {
            errorMessage = "Challenge token missing."
            return
        }
        guard !totpCode.trimmingCharacters(in: .whitespaces).isEmpty else {
            errorMessage = "Please enter 2FA verification code."
            return
        }
        
        isLoading = true
        Task {
            do {
                let req = Login2faRequestDto(
                    code: totpCode.trimmingCharacters(in: .whitespaces),
                    tempToken: token
                )
                let auth = try await APIClient.shared.login2fa(dto: req)
                self.currentUser = auth.user
                self.isAuthenticated = true
                self.requires2fa = false
                self.challengeToken = nil
                self.totpCode = ""
                self.isLoading = false
                self.clearForm()
            } catch {
                self.errorMessage = error.localizedDescription
                self.isLoading = false
            }
        }
    }
    
    public func cancel2fa() {
        requires2fa = false
        challengeToken = nil
        totpCode = ""
        errorMessage = nil
    }
    
    public func logout() {
        isLoading = true
        Task {
            do {
                try await APIClient.shared.logout()
            } catch {
                // Ignore error on logout
            }
            self.isAuthenticated = false
            self.currentUser = nil
            self.isLoading = false
            self.clearForm()
        }
    }
    
    public func clearForm() {
        email = ""
        password = ""
        name = ""
        errorMessage = nil
    }
}
