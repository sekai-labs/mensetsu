package org.mensetsu.app.core.network

import java.net.URI

object EndpointValidator {

    sealed class ValidationResult {
        data class Valid(val normalizedUrl: String) : ValidationResult()
        data class Invalid(val reason: String) : ValidationResult()
    }

    fun validate(rawUrl: String?): ValidationResult {
        if (rawUrl.isNullOrBlank()) {
            return ValidationResult.Invalid("API endpoint URL cannot be empty.")
        }

        val trimmed = rawUrl.trim().trimEnd('/')
        if (trimmed.isEmpty()) {
            return ValidationResult.Invalid("API endpoint URL cannot be empty.")
        }

        val uri = try {
            val parsed = URI.create(trimmed)
            if (!parsed.isAbsolute) {
                return ValidationResult.Invalid("URL must be absolute (e.g. https://api.example.com).")
            }
            parsed
        } catch (e: Exception) {
            return ValidationResult.Invalid("Invalid URL format: ${e.message ?: "malformed URI"}")
        }

        if (uri.rawUserInfo != null) {
            return ValidationResult.Invalid("URL must not contain credentials or userinfo.")
        }

        if (uri.rawQuery != null) {
            return ValidationResult.Invalid("URL must not contain a query string.")
        }

        if (uri.rawFragment != null) {
            return ValidationResult.Invalid("URL must not contain a fragment identifier.")
        }

        if (uri.rawPath != "" && uri.rawPath != "/") {
            return ValidationResult.Invalid("Enter the API origin only, without a path.")
        }

        val scheme = uri.scheme?.lowercase()
        if (scheme != "http" && scheme != "https") {
            return ValidationResult.Invalid("URL scheme must be http or https.")
        }

        val host = uri.host?.lowercase()
        if (host.isNullOrBlank()) {
            return ValidationResult.Invalid("URL must include a valid host.")
        }

        if (host == "0.0.0.0") {
            return ValidationResult.Invalid("0.0.0.0 is a bind address and not a valid client destination.")
        }

        val isLoopback = isLocalHost(host)

        if (scheme == "http" && !isLoopback) {
            return ValidationResult.Invalid("HTTPS is required for remote servers. Plain HTTP is only permitted for local/loopback development.")
        }

        return ValidationResult.Valid(trimmed)
    }
    private fun isLocalHost(host: String): Boolean {
        if (host == "localhost" || host == "127.0.0.1") {
            return true
        }
        if (host == "10.0.2.2") {
            return true
        }
        if (host == "10.0.3.2") {
            return true
        }
        return false
    }
}
