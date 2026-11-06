
import Foundation
import Security

public protocol SecureTokenStoring {
    func saveRefreshToken(_ token: String) throws
    func getRefreshToken() throws -> String?
    func clearRefreshToken() throws
}

public final class KeychainStore: SecureTokenStoring {
    private let service: String
    private let account: String

    public init(service: String = "app.mensetsu.ios", account: String = "mensetsu_refresh_token") {
        self.service = service
        self.account = account
    }

    public func saveRefreshToken(_ token: String) throws {
        guard let data = token.data(using: .utf8) else { return }

        try? clearRefreshToken()

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]

        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw KeychainError.status(status)
        }
    }

    public func getRefreshToken() throws -> String? {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecReturnData as String: true,
            kSecMatchLimit as String: kSecMatchLimitOne,
        ]

        var item: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &item)

        if status == errSecItemNotFound {
            return nil
        }
        guard status == errSecSuccess, let data = item as? Data else {
            throw KeychainError.status(status)
        }

        return String(data: data, encoding: .utf8)
    }

    public func clearRefreshToken() throws {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
        ]

        let status = SecItemDelete(query as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw KeychainError.status(status)
        }
    }

    public enum KeychainError: Error, LocalizedError {
        case status(OSStatus)

        public var errorDescription: String? {
            switch self {
            case let .status(st):
                return SecCopyErrorMessageString(st, nil) as String? ?? "Keychain error code: \(st)"
            }
        }
    }
}
