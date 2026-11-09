import SwiftUI
import MensetsuContracts

public enum SettingsTab: String, CaseIterable, Identifiable {
    case providers = "Providers"
    case tokenUsage = "Token Usage"

    public var id: String { rawValue }
}

public struct SettingsView: View {
    @Environment(\.presentationMode) var presentationMode
    @ObservedObject var authViewModel: AuthViewModel
    @StateObject private var viewModel = SettingsViewModel()
    @State private var selectedTab: SettingsTab = .providers
    @State private var serverUrlInput: String = AppConfig.savedServerUrl ?? AppConfig.apiBaseUrl
    @State private var serverUrlError: String? = nil
    @State private var serverSaveSuccess: Bool = false

    public init(authViewModel: AuthViewModel) {
        self.authViewModel = authViewModel
    }

    public var body: some View {
        NavigationView {
            VStack(spacing: 0) {
                Picker("Settings Section", selection: $selectedTab) {
                    ForEach(SettingsTab.allCases) { tab in
                        Text(tab.rawValue).tag(tab)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, 16)
                .padding(.vertical, 8)

                if selectedTab == .providers {
                    providersView
                } else {
                    tokenUsageView
                }
            }
            .navigationTitle("Settings")
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
                if case .authenticated = authViewModel.authState {
                    await viewModel.loadSettings()
                    await viewModel.loadTokenUsage()
                }
            }
        }
        .accessibilityIdentifier(AccessibilityID.settingsScreen)
    }

    private var providersView: some View {
        List {
            Section(
                header: Text("API Server Endpoint"),
                footer: Text("Base URL of your self-hosted Rust API. Changing the server signs out the active session.")
            ) {
                VStack(alignment: .leading, spacing: 8) {
                    TextField("https://api.example.com", text: $serverUrlInput)
                        .textFieldStyle(.roundedBorder)
                        .autocapitalization(.none)
                        .disableAutocorrection(true)
                        .keyboardType(.URL)
                        .accessibilityIdentifier(AccessibilityID.settingsServerUrlInput)

                    if let error = serverUrlError {
                        Text(error)
                            .font(.caption)
                            .foregroundColor(AppColors.rose)
                            .accessibilityIdentifier(AccessibilityID.serverUrlErrorMessage)
                    } else if serverSaveSuccess {
                        Text("Server updated. Session reset.")
                            .font(.caption)
                            .foregroundColor(AppColors.emerald)
                    }

                    Button("Save Server URL") {
                        saveServerEndpoint()
                    }
                    .buttonStyle(.borderedProminent)
                    .accessibilityIdentifier(AccessibilityID.settingsSaveServerUrlButton)
                }
                .padding(.vertical, 4)
            }

            if case .authenticated = authViewModel.authState {
                Section(
                    header: Text("LLM Provider Keys"),
                    footer: Text("Configure API keys for external intelligence models. Keys are securely stored and masked.")
                ) {
                    if viewModel.isLoading && viewModel.configs.isEmpty {
                        HStack {
                            Spacer()
                            ProgressView("Loading settings...")
                                .accessibilityIdentifier(AccessibilityID.settingsLoadingIndicator)
                            Spacer()
                        }
                    } else if let error = viewModel.errorMessage, viewModel.configs.isEmpty {
                        VStack(spacing: 8) {
                            Text(error)
                                .font(.subheadline)
                                .foregroundColor(AppColors.rose)
                            Button("Retry") {
                                Task { await viewModel.loadSettings() }
                            }
                            .buttonStyle(.bordered)
                        }
                        .accessibilityIdentifier(AccessibilityID.settingsErrorBanner)
                    } else {
                        if let error = viewModel.errorMessage {
                            VStack(alignment: .leading, spacing: 6) {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(AppColors.rose)
                                Button("Retry") {
                                    Task { await viewModel.loadSettings() }
                                }
                                .font(.caption)
                            }
                            .accessibilityIdentifier(AccessibilityID.settingsErrorBanner)
                        }

                        if viewModel.configs.isEmpty {
                            Text("No provider settings available.")
                                .font(.subheadline)
                                .foregroundColor(AppColors.slate500)
                        } else {
                            ForEach($viewModel.configs, id: \.provider) { $config in
                                ProviderConfigRow(config: $config, onSave: { key in
                                    Task { await viewModel.saveKey(provider: config.provider, key: key) }
                                })
                                .accessibilityIdentifier(AccessibilityID.providerRow(provider: config.provider))
                            }
                        }
                    }
                }
                .accessibilityIdentifier(AccessibilityID.llmProvidersList)
            }
        }
        .listStyle(InsetGroupedListStyle())
    }

    private var tokenUsageView: some View {
        List {
            if case .authenticated = authViewModel.authState {
                Section {
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text("Token Usage Analytics")
                                    .font(.system(size: 14, weight: .semibold))
                                    .foregroundColor(AppColors.slate900)
                                Text("Per-model-request totals from recorded provider responses.")
                                    .font(.system(size: 11))
                                    .foregroundColor(AppColors.slate500)
                            }
                            Spacer()
                            Button(action: {
                                Task { await viewModel.loadTokenUsage() }
                            }) {
                                HStack(spacing: 4) {
                                    if viewModel.isTokenUsageLoading {
                                        ProgressView()
                                            .scaleEffect(0.7)
                                    } else {
                                        Image(systemName: "arrow.clockwise")
                                            .font(.system(size: 11))
                                    }
                                    Text("Refresh")
                                        .font(.system(size: 12, weight: .medium))
                                }
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(AppColors.slate100)
                                .cornerRadius(6)
                            }
                            .buttonStyle(.plain)
                            .disabled(viewModel.isTokenUsageLoading || viewModel.isLoadingOlder)
                        }

                        let totals = viewModel.tokenUsage?.totals
                        let columns = [
                            GridItem(.flexible(), spacing: 8),
                            GridItem(.flexible(), spacing: 8)
                        ]
                        LazyVGrid(columns: columns, spacing: 8) {
                            kpiMetricCard(
                                title: "REQUESTS",
                                value: formatNumber(totals?.requestCount ?? 0),
                                isAccent: false
                            )
                            kpiMetricCard(
                                title: "INPUT TOKENS",
                                value: formatNumber(totals?.inputTokens ?? 0),
                                isAccent: false
                            )
                            kpiMetricCard(
                                title: "OUTPUT TOKENS",
                                value: formatNumber(totals?.outputTokens ?? 0),
                                isAccent: false
                            )
                            kpiMetricCard(
                                title: "ALL TOKENS",
                                value: formatNumber(totals?.totalTokens ?? 0),
                                isAccent: true
                            )
                        }
                    }
                    .padding(.vertical, 4)
                }

                Section(header: Text("Request History")) {
                    if viewModel.isTokenUsageLoading && viewModel.tokenUsage == nil {
                        HStack {
                            Spacer()
                            ProgressView("Loading token usage...")
                            Spacer()
                        }
                        .padding(.vertical, 16)
                    } else if let error = viewModel.tokenUsageError, viewModel.tokenUsage == nil {
                        VStack(spacing: 8) {
                            Text(error)
                                .font(.subheadline)
                                .foregroundColor(AppColors.rose)
                            Button("Retry") {
                                Task { await viewModel.loadTokenUsage() }
                            }
                            .buttonStyle(.bordered)
                        }
                        .padding(.vertical, 8)
                    } else {
                        if let error = viewModel.tokenUsageError {
                            VStack(alignment: .leading, spacing: 6) {
                                Text(error)
                                    .font(.caption)
                                    .foregroundColor(AppColors.rose)
                                Button("Retry") {
                                    Task { await viewModel.loadTokenUsage() }
                                }
                                .font(.caption)
                            }
                        }

                        let rows = (viewModel.tokenUsage?.items ?? []) + viewModel.olderRows
                        if rows.isEmpty {
                            HStack {
                                Spacer()
                                Text("No model request usage has been recorded yet.")
                                    .font(.subheadline)
                                    .foregroundColor(AppColors.slate500)
                                    .padding(.vertical, 16)
                                Spacer()
                            }
                        } else {
                            ForEach(rows, id: \.id) { item in
                                TokenUsageRowView(item: item)
                            }

                            VStack(spacing: 8) {
                                HStack {
                                    Text("\(rows.count) requests loaded; aggregate covers \(formatNumber(viewModel.tokenUsage?.totals.requestCount ?? 0)).")
                                        .font(.system(size: 11))
                                        .foregroundColor(AppColors.slate500)
                                    Spacer()
                                }

                                if let olderError = viewModel.olderError {
                                    Text(olderError)
                                        .font(.caption)
                                        .foregroundColor(AppColors.rose)
                                }

                                if viewModel.tokenUsage?.nextCursor != nil {
                                    Button(action: {
                                        Task { await viewModel.loadOlderTokenUsage() }
                                    }) {
                                        HStack {
                                            if viewModel.isLoadingOlder {
                                                ProgressView()
                                                    .scaleEffect(0.8)
                                            }
                                            Text(viewModel.isLoadingOlder ? "Loading older requests..." : "Load older requests")
                                                .font(.system(size: 13, weight: .medium))
                                        }
                                        .frame(maxWidth: .infinity)
                                        .padding(.vertical, 6)
                                    }
                                    .buttonStyle(.bordered)
                                    .disabled(viewModel.isLoadingOlder || viewModel.isTokenUsageLoading)
                                }
                            }
                            .padding(.vertical, 6)
                        }
                    }
                }
            } else {
                Section {
                    Text("Sign in to view token usage analytics.")
                        .font(.subheadline)
                        .foregroundColor(AppColors.slate500)
                        .padding(.vertical, 12)
                }
            }
        }
        .listStyle(InsetGroupedListStyle())
    }

    private func kpiMetricCard(title: String, value: String, isAccent: Bool) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.system(size: 10, weight: .semibold))
                .foregroundColor(AppColors.slate500)
            Text(value)
                .font(.system(size: 16, weight: .semibold, design: .monospaced))
                .foregroundColor(isAccent ? AppColors.primary : AppColors.slate900)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(10)
        .background(AppColors.slate50)
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .stroke(AppColors.slate200, lineWidth: 1)
        )
        .cornerRadius(6)
    }

    private func saveServerEndpoint() {
        serverUrlError = nil
        serverSaveSuccess = false
        do {
            let validated = try AppConfig.normalizeAndValidateUrl(serverUrlInput)
            Task {
                do {
                    try await authViewModel.resetSessionForServerChange(newServerUrl: validated)
                    try AppConfig.saveServerUrl(validated)
                    serverUrlInput = validated
                    serverSaveSuccess = true
                    presentationMode.wrappedValue.dismiss()
                } catch {
                    serverUrlError = "Failed to reset session: \(error.localizedDescription)"
                }
            }
        } catch {
            serverUrlError = error.localizedDescription
        }
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

public struct TokenUsageRowView: View {
    let item: TokenUsageRowDto

    public var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(alignment: .center) {
                Text(item.modelName)
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(AppColors.slate900)
                    .lineLimit(1)
                Text(item.modelProvider)
                    .font(.system(size: 10, weight: .medium))
                    .foregroundColor(AppColors.slate500)
                    .padding(.horizontal, 4)
                    .padding(.vertical, 1)
                    .background(AppColors.slate100)
                    .cornerRadius(3)
                Spacer()
                Text("\(formatNumber(item.totalTokens)) tok")
                    .font(.system(size: 12, weight: .bold, design: .monospaced))
                    .foregroundColor(AppColors.primary)
            }

            HStack {
                Text(item.agentId)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundColor(AppColors.slate600)
                    .lineLimit(1)
                Spacer()
                Text("In: \(formatNumber(item.inputTokens)) · Out: \(formatNumber(item.outputTokens))")
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundColor(AppColors.slate500)
            }

            HStack {
                Text(formatTimestamp(item.timestamp))
                    .font(.system(size: 10, design: .monospaced))
                    .foregroundColor(AppColors.slate400)
                Spacer()
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

    @Published public var tokenUsage: TokenUsageResponseDto? = nil
    @Published public var olderRows: [TokenUsageRowDto] = []
    @Published public var isTokenUsageLoading: Bool = false
    @Published public var tokenUsageError: String? = nil
    @Published public var isLoadingOlder: Bool = false
    @Published public var olderError: String? = nil

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

    public func loadTokenUsage() async {
        guard !isTokenUsageLoading, !isLoadingOlder else { return }
        isTokenUsageLoading = true
        tokenUsageError = nil
        olderRows = []
        olderError = nil
        do {
            let res: TokenUsageResponseDto = try await NetworkClient.shared.executeWithAuthRetry { _ in
                try await withCheckedThrowingContinuation { continuation in
                    TokenUsageAPI.userTokenUsage { data, error in
                        if let error = error {
                            continuation.resume(throwing: error)
                        } else if let data = data {
                            continuation.resume(returning: data)
                        } else {
                            continuation.resume(throwing: APIError.invalidResponse)
                        }
                    }
                }
            }
            tokenUsage = res
        } catch {
            tokenUsageError = error.localizedDescription
        }
        isTokenUsageLoading = false
    }

    public func loadOlderTokenUsage() async {
        guard !isTokenUsageLoading, let cursor = tokenUsage?.nextCursor, !isLoadingOlder else { return }
        isLoadingOlder = true
        olderError = nil
        do {
            let page: TokenUsageResponseDto = try await NetworkClient.shared.executeWithAuthRetry { _ in
                try await withCheckedThrowingContinuation { continuation in
                    TokenUsageAPI.userTokenUsage(
                        beforeTimestamp: cursor.beforeTimestamp,
                        beforeEventId: cursor.beforeEventId
                    ) { data, error in
                        if let error = error {
                            continuation.resume(throwing: error)
                        } else if let data = data {
                            continuation.resume(returning: data)
                        } else {
                            continuation.resume(throwing: APIError.invalidResponse)
                        }
                    }
                }
            }
            olderRows.append(contentsOf: page.items)
            if let current = tokenUsage {
                tokenUsage = TokenUsageResponseDto(
                    items: current.items,
                    nextCursor: page.nextCursor,
                    totals: page.totals
                )
            }
        } catch {
            olderError = "Older requests could not be loaded."
        }
        isLoadingOlder = false
    }
}

private func formatNumber(_ value: Int64) -> String {
    let formatter = NumberFormatter()
    formatter.numberStyle = .decimal
    return formatter.string(from: NSNumber(value: value)) ?? "\(value)"
}

private func formatTimestamp(_ isoString: String) -> String {
    let iso = ISO8601DateFormatter()
    iso.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    var date = iso.date(from: isoString)
    if date == nil {
        iso.formatOptions = [.withInternetDateTime]
        date = iso.date(from: isoString)
    }
    if let date = date {
        let df = DateFormatter()
        df.dateStyle = .short
        df.timeStyle = .short
        return df.string(from: date)
    }
    if isoString.count >= 16 {
        let start = isoString.index(isoString.startIndex, offsetBy: 5)
        let end = isoString.index(isoString.startIndex, offsetBy: 16)
        return String(isoString[start..<end]).replacingOccurrences(of: "T", with: " ")
    }
    return isoString
}
