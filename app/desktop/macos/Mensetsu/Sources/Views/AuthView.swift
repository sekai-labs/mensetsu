//
// AuthView.swift
// Mensetsu macOS
//

import SwiftUI

public struct AuthView: View {
    @ObservedObject var viewModel: AuthViewModel
    
    public init(viewModel: AuthViewModel) {
        self.viewModel = viewModel
    }
    
    public var body: some View {
        VStack(spacing: 24) {
            // Header
            VStack(spacing: 8) {
                Image(systemName: "sparkles.rectangle.stack.fill")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 48, height: 48)
                    .foregroundColor(.accentColor)
                
                Text("Mensetsu")
                    .font(.system(size: 28, weight: .bold))
                
                Text("Native Intelligence for Career Preparation")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
            }
            .padding(.top, 24)
            
            // Picker Mode
            Picker("Mode", selection: $viewModel.isRegisterMode) {
                Text("Sign In")
                    .tag(false)
                    .accessibilityIdentifier(AccessibilityIdentifiers.tabLogin)
                Text("Register")
                    .tag(true)
                    .accessibilityIdentifier(AccessibilityIdentifiers.tabRegister)
            }
            .pickerStyle(SegmentedPickerStyle())
            .frame(maxWidth: 320)
            
            // Input Fields
            VStack(alignment: .leading, spacing: 14) {
                if viewModel.isRegisterMode {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Full Name")
                            .font(.caption)
                            .foregroundColor(.secondary)
                        TextField("e.g. Satoshi Nakamoto", text: $viewModel.name)
                            .textFieldStyle(RoundedBorderTextFieldStyle())
                            .accessibilityIdentifier(AccessibilityIdentifiers.inputName)
                    }
                }
                
                VStack(alignment: .leading, spacing: 4) {
                    Text("Email")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    TextField("you@example.com", text: $viewModel.email)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputEmail)
                }
                
                VStack(alignment: .leading, spacing: 4) {
                    Text("Password")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    SecureField("••••••••", text: $viewModel.password)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputPassword)
                }
                
                if let error = viewModel.errorMessage {
                    Text(error)
                        .font(.caption)
                        .foregroundColor(.red)
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityIdentifier(AccessibilityIdentifiers.authErrorMessage)
                }
                
                Button(action: {
                    viewModel.submitAuth()
                }) {
                    HStack {
                        if viewModel.isLoading {
                            ProgressView()
                                .controlSize(.small)
                        }
                        Text(viewModel.isRegisterMode ? "Create Account" : "Sign In")
                            .fontWeight(.semibold)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 8)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .disabled(viewModel.isLoading)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonSubmit)
            }
            .frame(maxWidth: 320)
            
            // Server Endpoint Configuration button (accessible even if login fails)
            HStack {
                Spacer()
                Button(action: {
                    viewModel.isShowingSettings = true
                }) {
                    HStack(spacing: 4) {
                        Image(systemName: "server.rack")
                        Text("Server: \(AppConfig.apiBaseUrl.isEmpty ? "Not configured" : AppConfig.apiBaseUrl)")
                            .lineLimit(1)
                            .truncationMode(.middle)
                    }
                    .font(.caption)
                    .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonSettings)
                Spacer()
            }
            .frame(maxWidth: 320)

            Spacer()
        }
        .padding(32)
        .frame(minWidth: 460, minHeight: 480)
        .accessibilityIdentifier(AccessibilityIdentifiers.authScreen)
        .sheet(isPresented: $viewModel.requires2fa) {
            TwoFactorChallengeSheet(viewModel: viewModel)
        }
        .sheet(isPresented: $viewModel.isShowingSettings) {
            SettingsView(authViewModel: viewModel)
        }
    }
}

public struct TwoFactorChallengeSheet: View {
    @ObservedObject var viewModel: AuthViewModel
    
    public var body: some View {
        VStack(spacing: 20) {
            Image(systemName: "shield.lefthalf.filled")
                .resizable()
                .scaledToFit()
                .frame(width: 44, height: 44)
                .foregroundColor(.accentColor)
            
            Text("Two-Factor Authentication")
                .font(.title2)
                .fontWeight(.bold)
            
            Text("Enter the 6-digit TOTP code from your authenticator app.")
                .font(.subheadline)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
            
            TextField("000000", text: $viewModel.totpCode)
                .textFieldStyle(RoundedBorderTextFieldStyle())
                .font(.system(.title3, design: .monospaced))
                .multilineTextAlignment(.center)
                .frame(width: 180)
                .accessibilityIdentifier(AccessibilityIdentifiers.input2faCode)
            
            if let error = viewModel.errorMessage {
                Text(error)
                    .font(.caption)
                    .foregroundColor(.red)
                    .accessibilityIdentifier(AccessibilityIdentifiers.authErrorMessage)
            }
            
            HStack(spacing: 12) {
                Button("Cancel") {
                    viewModel.cancel2fa()
                }
                .keyboardShortcut(.cancelAction)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonCancel2fa)
                
                Button("Verify") {
                    viewModel.submit2faChallenge()
                }
                .buttonStyle(.borderedProminent)
                .disabled(viewModel.isLoading || viewModel.totpCode.count < 6)
                .keyboardShortcut(.defaultAction)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonSubmit2fa)
            }
        }
        .padding(30)
        .frame(width: 360, height: 300)
        .accessibilityIdentifier(AccessibilityIdentifiers.challenge2faSheet)
    }
}
