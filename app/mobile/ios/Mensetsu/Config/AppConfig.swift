//
// AppConfig.swift
// Mensetsu
//

import Foundation

public enum EndpointValidationError: LocalizedError {
    case emptyUrl
    case invalidUrl
    case unsupportedScheme
    case insecureRemoteConnection
    case disallowedComponents

    public var errorDescription: String? {
        switch self {
        case .emptyUrl:
            return "Server URL cannot be empty."
        case .invalidUrl:
            return "Please enter a valid, absolute URL (e.g. https://api.example.com)."
        case .unsupportedScheme:
            return "Only http:// and https:// URLs are supported."
        case .insecureRemoteConnection:
            return "Insecure HTTP is only permitted for loopback development (localhost, 127.0.0.1, ::1). Remote servers must use HTTPS."
        case .disallowedComponents:
            return "Server URL must not contain user credentials, query parameters, or fragments."
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
    /// Falls back to empty string if not configured (never silent default in production).
    public static var apiBaseUrl: String {
        if let saved = savedServerUrl {
            return saved
        }
        return ""
    }

    /// Normalizes (strips trailing slash, trims whitespace) and validates the URL.
    /// Requires absolute http/https URL and HTTPS for non-loopback hosts.
    public static func normalizeAndValidateUrl(_ rawUrl: String) throws -> String {
        let trimmed = rawUrl.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            throw EndpointValidationError.emptyUrl
        }

        // Strip trailing slashes
        var cleaned = trimmed
        while cleaned.hasSuffix("/") {
            cleaned.removeLast()
        }

        guard let components = URLComponents(string: cleaned),
              let scheme = components.scheme?.lowercased(),
              let host = components.host?.lowercased(),
              !host.isEmpty else {
            throw EndpointValidationError.invalidUrl
        }

        // Disallow user credentials, query strings, and fragments
        if components.user != nil || components.password != nil || components.query != nil || components.fragment != nil {
            throw EndpointValidationError.disallowedComponents
        }

        // Disallow path beyond empty or "/"
        if !components.path.isEmpty && components.path != "/" {
            throw EndpointValidationError.disallowedComponents
        }

        guard scheme == "http" || scheme == "https" else {
            throw EndpointValidationError.unsupportedScheme
        }

        let isLoopback = host == "localhost"
            || host == "127.0.0.1"
            || host == "::1"
            || host.hasSuffix(".localhost")

        if scheme == "http" && !isLoopback {
            throw EndpointValidationError.insecureRemoteConnection
        }

        return cleaned
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
