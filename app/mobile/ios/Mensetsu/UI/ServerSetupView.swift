
import SwiftUI
import MensetsuContracts

public struct ServerSetupView: View {
    @ObservedObject var authViewModel: AuthViewModel
    var onConfigured: (String) -> Void

    @State private var serverUrlInput: String = ""
    @State private var errorMessage: String? = nil

    public init(authViewModel: AuthViewModel, onConfigured: @escaping (String) -> Void) {
        self.authViewModel = authViewModel
        self.onConfigured = onConfigured
    }

    public var body: some View {
        NavigationView {
            ZStack {
                Color(UIColor.systemGroupedBackground).ignoresSafeArea()

                VStack(spacing: 24) {
                    VStack(spacing: 8) {
                        Image(systemName: "server.rack")
                            .font(.system(size: 48, weight: .semibold))
                            .foregroundColor(AppColors.primary)

                        Text("Connect Server")
                            .font(.system(size: 26, weight: .bold))
                            .foregroundColor(AppColors.slate900)

                        Text("Enter the base URL of your self-hosted Rust API to get started.")
                            .font(.system(size: 14))
                            .foregroundColor(AppColors.slate500)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, 24)
                    }
                    .padding(.top, 40)

                    VStack(alignment: .leading, spacing: 12) {
                        Text("Server URL")
                            .font(.system(size: 14, weight: .semibold))
                            .foregroundColor(AppColors.slate700)

                        TextField("https://api.example.com", text: $serverUrlInput)
                            .textFieldStyle(.roundedBorder)
                            .autocapitalization(.none)
                            .disableAutocorrection(true)
                            .keyboardType(.URL)
                            .accessibilityIdentifier(AccessibilityID.inputServerUrl)

                        Text("Use HTTPS for remote deployments. Loopback (http://localhost or http://127.0.0.1) is permitted for local development.")
                            .font(.system(size: 12))
                            .foregroundColor(AppColors.slate500)

                        if let error = errorMessage {
                            HStack(spacing: 6) {
                                Image(systemName: "exclamationmark.circle.fill")
                                    .foregroundColor(AppColors.roseDark)
                                Text(error)
                                    .font(.system(size: 13))
                                    .foregroundColor(AppColors.roseDark)
                            }
                            .padding(.vertical, 4)
                            .accessibilityIdentifier(AccessibilityID.serverUrlErrorMessage)
                        }

                        Button(action: saveServer) {
                            HStack {
                                Spacer()
                                Text("Connect")
                                    .font(.system(size: 16, weight: .semibold))
                                    .foregroundColor(.white)
                                Spacer()
                            }
                            .padding(.vertical, 14)
                            .background(AppColors.primary)
                            .cornerRadius(8)
                        }
                        .disabled(serverUrlInput.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        .opacity(serverUrlInput.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ? 0.6 : 1.0)
                        .accessibilityIdentifier(AccessibilityID.buttonSaveServerUrl)
                    }
                    .padding(20)
                    .background(Color(UIColor.systemBackground))
                    .cornerRadius(12)
                    .shadow(color: Color.black.opacity(0.04), radius: 8, x: 0, y: 2)

                    Spacer()
                }
                .padding(.horizontal, 20)
            }
            .navigationBarHidden(true)
        }
        .accessibilityIdentifier(AccessibilityID.serverSetupScreen)
    }

    private func saveServer() {
        errorMessage = nil
        do {
            let validated = try AppConfig.normalizeAndValidateUrl(serverUrlInput)
            Task {
                do {
                    try await authViewModel.resetSessionForServerChange(newServerUrl: validated)
                    try AppConfig.saveServerUrl(validated)
                    onConfigured(validated)
                } catch {
                    errorMessage = "Failed to clear existing credentials: \(error.localizedDescription)"
                }
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
