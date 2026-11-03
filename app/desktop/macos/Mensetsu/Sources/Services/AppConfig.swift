//
// AppConfig.swift
// Mensetsu macOS
//

import Foundation

public enum EndpointValidationError: LocalizedError {
    case emptyUrl
    case invalidUrl
    case unsupportedScheme
    case containsUserInfo
    case containsQueryOrFragment
    case containsPath
    case insecureRemoteConnection

    public var errorDescription: String? {
        switch self {
        case .emptyUrl:
            return "Server URL cannot be empty."
        case .invalidUrl:
            return "Please enter a valid, absolute URL (e.g. https://api.example.com)."
        case .unsupportedScheme:
            return "Only http:// and https:// URLs are supported."
        case .containsUserInfo:
            return "Server URL must not contain user credentials (username/password)."
        case .containsQueryOrFragment:
            return "Server URL must not contain query parameters or fragments."
        case .containsPath:
            return "Server URL must not contain a path. Enter the base server origin (e.g. https://api.example.com)."
        case .insecureRemoteConnection:
            return "Insecure HTTP is only permitted for loopback development (localhost, 127.0.0.1, ::1). Remote servers must use HTTPS."
        }
    }
}

public struct AppConfig {
    public static let serverEndpointKey = "mensetsu_server_endpoint"

    /// Persisted endpoint saved by user in UserDefaults.
    public static var savedServerUrl: String? {
        get {
            guard let raw = UserDefaults.standard.string(forKey: serverEndpointKey),
                  !raw.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
                return nil
            }
            return try? normalizeAndValidateUrl(raw)
        }
        set {
            if let newValue = newValue {
                if let validated = try? normalizeAndValidateUrl(newValue) {
                    UserDefaults.standard.set(validated, forKey: serverEndpointKey)
                }
            } else {
                UserDefaults.standard.removeObject(forKey: serverEndpointKey)
            }
        }
    }

    /// True if an endpoint is configured in saved user endpoint.
    public static var isConfigured: Bool {
        return savedServerUrl != nil
    }

    /// The active base URL for API requests. Returns saved URL if present.
    public static var apiBaseUrl: String {
        if let saved = savedServerUrl {
            return saved
        }
        return ""
    }

    /// Normalizes (strips trailing slash, trims) and validates the URL.
    /// Rejects userinfo, query, fragment, path, and non-loopback HTTP.
    public static func normalizeAndValidateUrl(_ rawUrl: String) throws -> String {
        let trimmed = rawUrl.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            throw EndpointValidationError.emptyUrl
        }

        guard let components = URLComponents(string: trimmed) else {
            throw EndpointValidationError.invalidUrl
        }

        guard let scheme = components.scheme?.lowercased(),
              let host = components.host?.lowercased(),
              !host.isEmpty else {
            throw EndpointValidationError.invalidUrl
        }

        guard scheme == "http" || scheme == "https" else {
            throw EndpointValidationError.unsupportedScheme
        }

        // Disallow credentials in userinfo
        if components.user != nil || components.password != nil {
            throw EndpointValidationError.containsUserInfo
        }

        // Disallow query or fragment
        if components.query != nil || components.fragment != nil {
            throw EndpointValidationError.containsQueryOrFragment
        }

        // Disallow path beyond empty or root slash
        let path = components.path
        if !path.isEmpty && path != "/" {
            throw EndpointValidationError.containsPath
        }

        // Only 127.0.0.1, ::1, and localhost (or *.localhost) are valid loopback addresses
        let isLoopback = host == "localhost"
            || host == "127.0.0.1"
            || host == "::1"
            || host.hasSuffix(".localhost")

        if scheme == "http" && !isLoopback {
            throw EndpointValidationError.insecureRemoteConnection
        }

        // Rebuild clean base origin without trailing slash
        var cleanComponents = URLComponents()
        cleanComponents.scheme = scheme
        cleanComponents.host = host
        cleanComponents.port = components.port

        guard let cleanUrl = cleanComponents.string else {
            throw EndpointValidationError.invalidUrl
        }

        return cleanUrl
    }

    /// Saves a newly validated server URL. Returns the validated normalized URL.
    @discardableResult
    public static func saveServerUrl(_ rawUrl: String) throws -> String {
        let validated = try normalizeAndValidateUrl(rawUrl)
        savedServerUrl = validated
        return validated
    }

    /// Clears the saved server URL.
    public static func clearSavedServerUrl() {
        savedServerUrl = nil
    }
}
