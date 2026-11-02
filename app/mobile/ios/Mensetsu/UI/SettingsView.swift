//
// SettingsView.swift
// Mensetsu
//

import SwiftUI
import MensetsuContracts

public struct SettingsView: View {
    @Environment(\.presentationMode) var presentationMode
    @StateObject private var viewModel = SettingsViewModel()

    public init() {}

    public var body: some View {
        NavigationView {
            Group {
                if viewModel.isLoading && viewModel.configs.isEmpty {
                    ProgressView("Loading settings...")
                        .accessibilityIdentifier(AccessibilityID.settingsLoadingIndicator)
                } else if let error = viewModel.errorMessage, viewModel.configs.isEmpty {
                    VStack(spacing: 12) {
                        Image(systemName: "exclamationmark.triangle")
                            .font(.system(size: 40))
                            .foregroundColor(AppColors.rose)
                        Text("Error Loading Settings")
                            .font(.headline)
                        Text(error)
                            .font(.subheadline)
                            .foregroundColor(AppColors.slate500)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, 32)
                        Button("Retry") {
                            Task { await viewModel.loadSettings() }
                        }
                        .buttonStyle(.borderedProminent)
                    }
                    .accessibilityIdentifier(AccessibilityID.settingsErrorBanner)
                } else {
                    List {
                        Section(header: Text("LLM Provider Keys"), footer: Text("Configure API keys for external intelligence models. Keys are securely stored and masked.")) {
                            ForEach($viewModel.configs, id: \.provider) { $config in
                                ProviderConfigRow(config: $config, onSave: { key in
                                    Task { await viewModel.saveKey(provider: config.provider, key: key) }
                                })
                                .accessibilityIdentifier(AccessibilityID.providerRow(provider: config.provider))
                            }
                        }
                    }
                    .listStyle(InsetGroupedListStyle())
                    .accessibilityIdentifier(AccessibilityID.llmProvidersList)
                }
            }
            .navigationTitle("LLM Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        presentationMode.wrappedValue.dismiss()
                    }
                    .accessibilityIdentifier(AccessibilityID.settingsDoneButton)
                }
            }
            .task {
                await viewModel.loadSettings()
            }
        }
        .accessibilityIdentifier(AccessibilityID.settingsScreen)
    }
}

public struct ProviderConfigRow: View {
    @Binding var config: LlmConfigItemDto
    var onSave: (String) -> Void

    @State private var inputKey: String = ""
    @State private var isEditing: Bool = false

    public var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(config.provider.uppercased())
                    .font(.system(size: 15, weight: .semibold))
                Spacer()
                if config.hasKey {
                    Text("CONFIGURED")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(AppColors.emeraldDark)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(AppColors.emeraldLight)
                        .cornerRadius(4)
                } else {
                    Text("NO KEY")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(AppColors.slate500)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(AppColors.slate100)
                        .cornerRadius(4)
                }
            }

            if let masked = config.maskedKey, !isEditing {
                Text(masked)
                    .font(.system(size: 12, design: .monospaced))
                    .foregroundColor(AppColors.slate500)
            }

            if isEditing {
                HStack {
                    SecureField("Enter API Key (sk-...)", text: $inputKey)
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .accessibilityIdentifier(AccessibilityID.providerKeyField(provider: config.provider))

                    Button("Save") {
                        let trimmed = inputKey.trimmingCharacters(in: .whitespaces)
                        if !trimmed.isEmpty {
                            onSave(trimmed)
                            isEditing = false
                            inputKey = ""
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier(AccessibilityID.providerSaveButton(provider: config.provider))

                    Button("Cancel") {
                        isEditing = false
                        inputKey = ""
                    }
                    .buttonStyle(.borderless)
                }
            } else {
                Button(action: { isEditing = true }) {
                    Text(config.hasKey ? "Update Key" : "Add Key")
                        .font(.system(size: 13, weight: .medium))
                        .foregroundColor(AppColors.primary)
                }
            }
        }
        .padding(.vertical, 4)
    }
}

@MainActor
public final class SettingsViewModel: ObservableObject {
    @Published public var configs: [LlmConfigItemDto] = []
    @Published public var isLoading: Bool = false
    @Published public var errorMessage: String? = nil

    private let service: WorkspaceServiceProtocol

    public init(service: WorkspaceServiceProtocol = WorkspaceService()) {
        self.service = service
    }

    public func loadSettings() async {
        isLoading = true
        errorMessage = nil
        do {
            configs = try await service.getLlmSettings()
        } catch {
            errorMessage = error.localizedDescription
        }
        isLoading = false
    }

    public func saveKey(provider: String, key: String) async {
        do {
            let updated = try await service.updateLlmSetting(provider: provider, apiKey: key, isShared: false)
            configs = configs.map { item in
                if item.provider == provider {
                    return updated
                }
                return item
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
