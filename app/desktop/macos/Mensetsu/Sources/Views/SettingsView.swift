//
// SettingsView.swift
// Mensetsu macOS
//

import SwiftUI
import MensetsuContracts

public struct SettingsView: View {
    @ObservedObject var authViewModel: AuthViewModel
    @State private var serverUrl: String = ""
    @State private var serverUrlError: String?
    @State private var serverUrlSavedMessage: String?

    // LLM Provider Settings
    @State private var llmConfigs: [LlmConfigItemDto] = []
    @State private var isLoadingLlm: Bool = false
    @State private var llmError: String?
    @State private var editingProvider: String?
    @State private var editingApiKey: String = ""
    @State private var isSavingLlmKey: Bool = false
    @State private var llmKeyErrorMessage: String?

    @Environment(\.dismiss) private var dismiss

    public init(authViewModel: AuthViewModel) {
        self.authViewModel = authViewModel
    }

    public var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                Text("Settings")
                    .font(.title2)
                    .fontWeight(.bold)
                Spacer()
                Button("Done") {
                    dismiss()
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.regular)
                .accessibilityIdentifier(AccessibilityIdentifiers.settingsButtonDone)
            }
            .padding(20)

            Divider()

            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    // Server Endpoint Section
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Server Endpoint")
                            .font(.headline)

                        Text("Rust API base URL used for all workspace, intelligence, and auth operations.")
                            .font(.caption)
                            .foregroundColor(.secondary)

                        VStack(alignment: .leading, spacing: 6) {
                            TextField("https://api.example.com", text: $serverUrl)
                                .textFieldStyle(RoundedBorderTextFieldStyle())
                                .accessibilityIdentifier(AccessibilityIdentifiers.settingsInputServerUrl)

                            if let error = serverUrlError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                                    .accessibilityIdentifier(AccessibilityIdentifiers.settingsServerUrlErrorMessage)
                            } else if let savedMsg = serverUrlSavedMessage {
                                Text(savedMsg)
                                    .font(.caption)
                                    .foregroundColor(.green)
                            }
                        }

                        Button("Update Server Endpoint") {
                            saveServerUrl()
                        }
                        .buttonStyle(.bordered)
                        .accessibilityIdentifier(AccessibilityIdentifiers.settingsButtonSaveServerUrl)
                    }
                    .padding(16)
                    .background(Color(NSColor.controlBackgroundColor))
                    .cornerRadius(8)

                    // LLM Providers Section (if authenticated)
                    if authViewModel.isAuthenticated {
                        VStack(alignment: .leading, spacing: 12) {
                            Text("LLM Provider Configuration")
                                .font(.headline)

                            Text("Custom API keys for LLM reasoning providers.")
                                .font(.caption)
                                .foregroundColor(.secondary)

                            if isLoadingLlm && llmConfigs.isEmpty {
                                ProgressView()
                            } else if let error = llmError {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(.red)
                                Button("Retry") {
                                    loadLlmConfigs()
                                }
                                .controlSize(.small)
                            } else {
                                VStack(spacing: 8) {
                                    ForEach(llmConfigs, id: \.provider) { config in
                                        HStack {
                                            VStack(alignment: .leading, spacing: 2) {
                                                Text(config.provider.uppercased())
                                                    .font(.subheadline)
                                                    .fontWeight(.medium)
                                                Text(config.hasKey ? "Key configured (\(config.isShared ? "Shared" : "Personal"))" : "No key provided")
                                                    .font(.caption2)
                                                    .foregroundColor(config.hasKey ? .green : .secondary)
                                            }
                                            Spacer()
                                            Button("Configure") {
                                                editingProvider = config.provider
                                                editingApiKey = ""
                                                llmKeyErrorMessage = nil
                                            }
                                            .buttonStyle(.bordered)
                                            .controlSize(.small)
                                            .accessibilityIdentifier("\(AccessibilityIdentifiers.llmConfigureKeyPrefix)\(config.provider)")
                                        }
                                        .padding(10)
                                        .background(Color(NSColor.windowBackgroundColor))
                                        .cornerRadius(6)
                                        .accessibilityIdentifier("\(AccessibilityIdentifiers.llmProviderRowPrefix)\(config.provider)")
                                    }
                                }
                                .accessibilityIdentifier(AccessibilityIdentifiers.llmProvidersList)
                            }
                        }
                        .padding(16)
                        .background(Color(NSColor.controlBackgroundColor))
                        .cornerRadius(8)
                    }
                }
                .padding(20)
            }
        }
        .frame(minWidth: 500, minHeight: 480)
        .accessibilityIdentifier(AccessibilityIdentifiers.settingsScreen)
        .onAppear {
            serverUrl = AppConfig.savedServerUrl ?? ""
            if authViewModel.isAuthenticated {
                loadLlmConfigs()
            }
        }
        .sheet(isPresented: Binding(
            get: { editingProvider != nil },
            set: { if !$0 { editingProvider = nil } }
        )) {
            VStack(spacing: 16) {
                Text("Configure \(editingProvider?.uppercased() ?? "") Key")
                    .font(.headline)

                SecureField("Enter API Key (sk-...)", text: $editingApiKey)
                    .textFieldStyle(RoundedBorderTextFieldStyle())
                    .accessibilityIdentifier(AccessibilityIdentifiers.inputLlmApiKey)

                if let error = llmKeyErrorMessage {
                    Text(error)
                        .font(.caption)
                        .foregroundColor(.red)
                }

                HStack(spacing: 12) {
                    Button("Cancel") {
                        editingProvider = nil
                    }
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonCancelLlmKey)

                    Button("Save") {
                        if let prov = editingProvider {
                            saveLlmKey(provider: prov, apiKey: editingApiKey)
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(editingApiKey.isEmpty || isSavingLlmKey)
                    .accessibilityIdentifier(AccessibilityIdentifiers.buttonSaveLlmKey)
                }
            }
            .padding(24)
            .frame(width: 380)
            .accessibilityIdentifier(AccessibilityIdentifiers.llmKeySheet)
        }
    }

    private func saveServerUrl() {
        serverUrlError = nil
        serverUrlSavedMessage = nil
        do {
            let previous = AppConfig.apiBaseUrl
            let validated = try AppConfig.normalizeAndValidateUrl(serverUrl)

            Task {
                do {
                    // If changing endpoint, reset tokens and isolate session
                    if previous != validated {
                        try await authViewModel.changeServerEndpoint(to: validated)
                        serverUrlSavedMessage = "Server endpoint updated. Active session reset."
                    } else {
                        AppConfig.savedServerUrl = validated
                        serverUrlSavedMessage = "Server endpoint saved."
                    }
                    serverUrl = validated
                } catch {
                    serverUrlError = error.localizedDescription
                }
            }
        } catch {
            serverUrlError = error.localizedDescription
        }
    }

    private func loadLlmConfigs() {
        isLoadingLlm = true
        llmError = nil
        Task {
            do {
                let configs = try await APIClient.shared.getLlmSettings()
                await MainActor.run {
                    self.llmConfigs = configs
                    self.isLoadingLlm = false
                }
            } catch {
                await MainActor.run {
                    self.llmError = error.localizedDescription
                    self.isLoadingLlm = false
                }
            }
        }
    }

    private func saveLlmKey(provider: String, apiKey: String) {
        isSavingLlmKey = true
        llmKeyErrorMessage = nil
        Task {
            do {
                let req = UpsertLlmConfigRequestDto(apiKey: apiKey, provider: provider)
                _ = try await APIClient.shared.upsertLlmSetting(dto: req)
                await MainActor.run {
                    self.isSavingLlmKey = false
                    self.editingProvider = nil
                    self.loadLlmConfigs()
                }
            } catch {
                await MainActor.run {
                    self.llmKeyErrorMessage = error.localizedDescription
                    self.isSavingLlmKey = false
                }
            }
        }
    }
}
