//
// AppConfig.swift
// Mensetsu macOS
//

import Foundation

public struct AppConfig {
    public static var apiBaseUrl: String {
        if let envUrl = ProcessInfo.processInfo.environment["MENSETSU_API_URL"], !envUrl.isEmpty {
            return envUrl
        }
        if let argIndex = ProcessInfo.processInfo.arguments.firstIndex(of: "--api-url"),
           argIndex + 1 < ProcessInfo.processInfo.arguments.count {
            return ProcessInfo.processInfo.arguments[argIndex + 1]
        }
        #if DEBUG
        return "http://127.0.0.1:3000"
        #else
        return "https://api.mensetsu.ai"
        #endif
    }
}
