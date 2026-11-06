
import SwiftUI
import MensetsuContracts

@main
struct MensetsuApp: App {
    @StateObject private var authViewModel = AuthViewModel()
    @State private var configuredEndpoint: String = AppConfig.apiBaseUrl

    init() {
        let initialUrl = AppConfig.apiBaseUrl
        if !initialUrl.isEmpty {
            MensetsuContractsAPI.basePath = initialUrl
        }
    }

    var body: some Scene {
        WindowGroup {
            ContentView(authViewModel: authViewModel, configuredEndpoint: $configuredEndpoint)
                .task {
                    if !configuredEndpoint.isEmpty {
                        await authViewModel.checkExistingSession()
                    }
                }
        }
    }
}

public struct ContentView: View {
    @ObservedObject var authViewModel: AuthViewModel
    @Binding var configuredEndpoint: String

    public init(authViewModel: AuthViewModel, configuredEndpoint: Binding<String>? = nil) {
        self.authViewModel = authViewModel
        self._configuredEndpoint = configuredEndpoint ?? .constant(AppConfig.apiBaseUrl)
    }

    public var body: some View {
        Group {
            if configuredEndpoint.isEmpty {
                ServerSetupView(authViewModel: authViewModel, onConfigured: { url in
                    configuredEndpoint = url
                })
            } else {
                switch authViewModel.authState {
                case .authenticated:
                    WorkspaceListView(authViewModel: authViewModel)
                case .unauthenticated, .requires2FA:
                    AuthView(viewModel: authViewModel)
                }
            }
        }
    }
}
