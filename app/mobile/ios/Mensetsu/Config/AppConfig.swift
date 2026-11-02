//
// AppConfig.swift
// Mensetsu
//

import Foundation

public struct AppConfig {
    public static var apiBaseUrl: String {
        if let envUrl = ProcessInfo.processInfo.environment["MENSETSU_API_BASE_URL"], !envUrl.isEmpty {
            return envUrl
        }
        if let plistUrl = Bundle.main.infoDictionary?["API_BASE_URL"] as? String, !plistUrl.isEmpty {
            return plistUrl
        }
        #if DEBUG
        return "http://localhost:8080"
        #else
        return "https://api.mensetsu.app"
        #endif
    }
}
