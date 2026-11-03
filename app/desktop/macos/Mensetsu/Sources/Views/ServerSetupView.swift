//
// ServerSetupView.swift
// Mensetsu macOS
//

import SwiftUI

public struct ServerSetupView: View {
    @ObservedObject var viewModel: AuthViewModel
    @State private var serverUrl: String = ""
    @State private var errorMessage: String?

    public init(viewModel: AuthViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: 24) {
            VStack(spacing: 8) {
                Image(systemName: "server.rack")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 48, height: 48)
                    .foregroundColor(.accentColor)

                Text("Connect to Mensetsu")
                    .font(.system(size: 26, weight: .bold))

                Text("Enter the base URL of your self-hosted Rust API server.")
                    .font(.subheadline)
                    .foregroundColor(.secondary)
                    .multilineTextAlignment(.center)
            }
            .padding(.top, 24)

            VStack(alignment: .leading, spacing: 14) {
                VStack(alignment: .leading, spacing: 6) {
                    Text("Server Base URL")
                        .font(.caption)
                        .fontWeight(.semibold)
                        .foregroundColor(.secondary)

                    TextField("https://api.mensetsu.example.com", text: $serverUrl)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityIdentifiers.inputServerUrl)

                    Text("e.g. http://127.0.0.1:3000 for local development or https://api.example.com for remote.")
                        .font(.caption2)
                        .foregroundColor(.secondary)
                }

                if let error = errorMessage ?? viewModel.serverUrlError {
                    Text(error)
                        .font(.caption)
                        .foregroundColor(.red)
                        .fixedSize(horizontal: false, vertical: true)
                        .accessibilityIdentifier(AccessibilityIdentifiers.serverUrlErrorMessage)
                }

                Button(action: {
                    handleConnect()
                }) {
                    Text("Connect to Server")
                        .fontWeight(.semibold)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 8)
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.large)
                .accessibilityIdentifier(AccessibilityIdentifiers.buttonSaveServerUrl)
            }
            .frame(maxWidth: 360)

            Spacer()
        }
        .padding(32)
        .frame(minWidth: 460, minHeight: 480)
        .accessibilityIdentifier(AccessibilityIdentifiers.serverSetupScreen)
        .onAppear {
            if let saved = AppConfig.savedServerUrl {
                serverUrl = saved
            }
        }
    }

    private func handleConnect() {
        errorMessage = nil
        viewModel.serverUrlInput = serverUrl
        Task {
            let success = await viewModel.saveServerEndpoint()
            if !success {
                errorMessage = viewModel.serverUrlError
            }
        }
    }
}
