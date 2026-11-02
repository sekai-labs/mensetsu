//
// MensetsuApp.swift
// Mensetsu macOS
//

import SwiftUI

@main
struct MensetsuApp: App {
    @StateObject private var authViewModel = AuthViewModel()
    
    var body: some Scene {
        WindowGroup {
            Group {
                if authViewModel.isAuthenticated {
                    MainSplitView(authViewModel: authViewModel)
                } else {
                    AuthView(viewModel: authViewModel)
                }
            }
            .frame(minWidth: 800, minHeight: 550)
        }
        .windowStyle(.titleBar)
        .windowToolbarStyle(.unified)
    }
}
