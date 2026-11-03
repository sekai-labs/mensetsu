//
// AuthView.swift
// Mensetsu
//

import SwiftUI
import MensetsuContracts

public struct AuthView: View {
    @ObservedObject var viewModel: AuthViewModel

    @State private var isRegisterMode = false
    @State private var name = ""
    @State private var email = ""
    @State private var password = ""
    @State private var twoFaCode = ""
    @State private var showingServerSettings = false
    public init(viewModel: AuthViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        NavigationView {
            ZStack {
                Color(UIColor.systemGroupedBackground).ignoresSafeArea()

                VStack(spacing: 24) {
                    // App Logo / Title
                    VStack(spacing: 8) {
                        Image(systemName: "sparkles")
                            .font(.system(size: 44, weight: .semibold))
                            .foregroundColor(AppColors.primary)

                        Text("Mensetsu")
                            .font(.system(size: 28, weight: .bold))
                            .foregroundColor(AppColors.slate900)

                        Text("Evidence-backed interview preparation platform")
                            .font(.system(size: 14))
                            .foregroundColor(AppColors.slate500)
                            .multilineTextAlignment(.center)
                    }
                    .padding(.top, 32)

                    if case .requires2FA(let tempToken) = viewModel.authState {
                        // 2FA Challenge Form
                        twoFaChallengeView(tempToken: tempToken)
                    } else {
                        // Standard Login / Register Card
                        authFormCard
                    }

                    // Pre-auth server setting trigger
                    Button(action: {
                        showingServerSettings = true
                    }) {
                        HStack(spacing: 6) {
                            Image(systemName: "server.rack")
                                .font(.system(size: 13))
                            Text("Server: \(AppConfig.apiBaseUrl.isEmpty ? "Not configured" : AppConfig.apiBaseUrl)")
                                .font(.system(size: 13, weight: .medium))
                                .lineLimit(1)
                                .truncationMode(.middle)
                        }
                        .foregroundColor(AppColors.slate600)
                        .padding(.vertical, 8)
                        .padding(.horizontal, 12)
                        .background(Color(UIColor.systemBackground))
                        .cornerRadius(8)
                    }
                    .accessibilityIdentifier(AccessibilityID.openServerSettingsButton)

                    Spacer()
                }
                .padding(.horizontal, 20)

                if viewModel.isLoading {
                    Color.black.opacity(0.15).ignoresSafeArea()
                    ProgressView()
                        .progressViewStyle(CircularProgressViewStyle())
                        .scaleEffect(1.5)
                        .accessibilityIdentifier(AccessibilityID.authLoadingIndicator)
                }
            }
            .sheet(isPresented: $showingServerSettings) {
                SettingsView(authViewModel: viewModel)
            }
            .navigationBarHidden(true)
        }
        .accessibilityIdentifier(AccessibilityID.authScreen)
    }

    private var authFormCard: some View {
        VStack(spacing: 16) {
            // Mode Picker
            Picker("Mode", selection: $isRegisterMode) {
                Text("Login").tag(false)
                    .accessibilityIdentifier(AccessibilityID.authLoginTab)
                Text("Register").tag(true)
                    .accessibilityIdentifier(AccessibilityID.authRegisterTab)
            }
            .pickerStyle(SegmentedPickerStyle())
            .accessibilityIdentifier(AccessibilityID.authModeSegment)

            if let error = viewModel.errorMessage {
                HStack(spacing: 8) {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundColor(AppColors.roseDark)
                    Text(error)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(AppColors.roseDark)
                    Spacer()
                }
                .padding(10)
                .background(AppColors.roseLight)
                .cornerRadius(6)
                .accessibilityIdentifier(AccessibilityID.authErrorBanner)
            }

            VStack(spacing: 12) {
                if isRegisterMode {
                    TextField("Full Name", text: $name)
                        .textContentType(.name)
                        .autocapitalization(.words)
                        .padding(12)
                        .background(Color(UIColor.secondarySystemGroupedBackground))
                        .cornerRadius(8)
                        .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color(UIColor.separator), lineWidth: 0.5))
                        .accessibilityIdentifier(AccessibilityID.nameField)
                }

                TextField("Email Address", text: $email)
                    .keyboardType(.emailAddress)
                    .autocapitalization(.none)
                    .disableAutocorrection(true)
                    .padding(12)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(8)
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color(UIColor.separator), lineWidth: 0.5))
                    .accessibilityIdentifier(AccessibilityID.emailField)

                SecureField("Password", text: $password)
                    .textContentType(isRegisterMode ? .newPassword : .password)
                    .padding(12)
                    .background(Color(UIColor.secondarySystemGroupedBackground))
                    .cornerRadius(8)
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color(UIColor.separator), lineWidth: 0.5))
                    .accessibilityIdentifier(AccessibilityID.passwordField)
            }

            Button(action: {
                Task {
                    if isRegisterMode {
                        await viewModel.register(name: name, email: email, password: password)
                    } else {
                        await viewModel.login(email: email, password: password)
                    }
                }
            }) {
                HStack {
                    Spacer()
                    Text(isRegisterMode ? "Create Account" : "Sign In")
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundColor(.white)
                    Spacer()
                }
                .padding(.vertical, 14)
                .background(AppColors.primary)
                .cornerRadius(8)
            }
            .disabled(viewModel.isLoading || email.isEmpty || password.isEmpty || (isRegisterMode && name.isEmpty))
            .opacity((email.isEmpty || password.isEmpty || (isRegisterMode && name.isEmpty)) ? 0.6 : 1.0)
            .accessibilityIdentifier(isRegisterMode ? AccessibilityID.registerButton : AccessibilityID.loginButton)
        }
        .padding(20)
        .background(Color(UIColor.systemBackground))
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.04), radius: 8, x: 0, y: 2)
    }

    private func twoFaChallengeView(tempToken: String) -> some View {
        VStack(spacing: 16) {
            Text("Two-Factor Authentication")
                .font(.system(size: 18, weight: .bold))
                .foregroundColor(AppColors.slate900)

            Text("Enter the 6-digit TOTP code from your authenticator app.")
                .font(.system(size: 14))
                .foregroundColor(AppColors.slate600)
                .multilineTextAlignment(.center)

            if let error = viewModel.errorMessage {
                Text(error)
                    .font(.system(size: 13, weight: .medium))
                    .foregroundColor(AppColors.roseDark)
                    .padding(10)
                    .background(AppColors.roseLight)
                    .cornerRadius(6)
                    .accessibilityIdentifier(AccessibilityID.twoFaErrorBanner)
            }

            TextField("6-digit code", text: $twoFaCode)
                .keyboardType(.numberPad)
                .font(.system(size: 20, weight: .semibold, design: .monospaced))
                .multilineTextAlignment(.center)
                .padding(12)
                .background(Color(UIColor.secondarySystemGroupedBackground))
                .cornerRadius(8)
                .accessibilityIdentifier(AccessibilityID.twoFaCodeField)

            Button(action: {
                Task {
                    await viewModel.verify2FA(code: twoFaCode, tempToken: tempToken)
                }
            }) {
                HStack {
                    Spacer()
                    Text("Verify & Continue")
                        .font(.system(size: 16, weight: .semibold))
                        .foregroundColor(.white)
                    Spacer()
                }
                .padding(.vertical, 14)
                .background(AppColors.primary)
                .cornerRadius(8)
            }
            .disabled(twoFaCode.count < 6 || viewModel.isLoading)
            .accessibilityIdentifier(AccessibilityID.twoFaVerifyButton)

            Button("Back to Login") {
                viewModel.cancel2FA()
            }
            .font(.system(size: 14))
            .foregroundColor(AppColors.slate500)
        }
        .padding(20)
        .background(Color(UIColor.systemBackground))
        .cornerRadius(12)
        .shadow(color: Color.black.opacity(0.04), radius: 8, x: 0, y: 2)
        .accessibilityIdentifier(AccessibilityID.twoFaScreen)
    }
}
