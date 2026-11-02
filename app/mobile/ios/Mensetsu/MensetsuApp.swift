//
// MensetsuApp.swift
// Mensetsu
//

import SwiftUI
import MensetsuContracts

@main
struct MensetsuApp: App {
    @StateObject private var authViewModel = AuthViewModel()

    init() {
        // Configure openapi-generator base path
        MensetsuContractsAPI.basePath = AppConfig.apiBaseUrl
    }

    var body: some Scene {
        WindowGroup {
            ContentView(authViewModel: authViewModel)
                .task {
                    await authViewModel.checkExistingSession()
                }
        }
    }
}

public struct ContentView: View {
    @ObservedObject var authViewModel: AuthViewModel

    public init(authViewModel: AuthViewModel) {
        self.authViewModel = authViewModel
    }

    public var body: some View {
        Group {
            switch authViewModel.authState {
            case .authenticated:
                WorkspaceListView(authViewModel: authViewModel)
            case .unauthenticated, .requires2FA:
                AuthView(viewModel: authViewModel)
            }
        }
    }
}
