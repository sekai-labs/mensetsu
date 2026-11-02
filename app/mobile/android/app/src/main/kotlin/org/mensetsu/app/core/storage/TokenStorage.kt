package org.mensetsu.app.core.storage

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey

/**
 * Android Keystore-backed persistence for the rotating refresh token.
 * Access tokens are NEVER written to disk and remain memory-only.
 */
class TokenStorage(context: Context) {
    private val prefs: SharedPreferences = EncryptedSharedPreferences.create(
        context,
        "mensetsu_secure_prefs",
        MasterKey.Builder(context).setKeyScheme(MasterKey.KeyScheme.AES256_GCM).build(),
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
    )

    private val basePrefs: SharedPreferences =
        context.getSharedPreferences("mensetsu_config_prefs", Context.MODE_PRIVATE)

    companion object {
        private const val KEY_REFRESH_TOKEN = "refresh_token"
        private const val KEY_CUSTOM_BASE_URL = "custom_base_url"
    }

    fun getRefreshToken(): String? {
        return prefs.getString(KEY_REFRESH_TOKEN, null)
    }

    fun saveRefreshToken(token: String?) {
        val editor = prefs.edit()
        if (token.isNullOrBlank()) {
            editor.remove(KEY_REFRESH_TOKEN)
        } else {
            editor.putString(KEY_REFRESH_TOKEN, token)
        }
        check(editor.commit()) { "Failed to securely persist refresh token" }
    }

    fun clearRefreshToken() {
        check(prefs.edit().remove(KEY_REFRESH_TOKEN).commit()) { "Failed to clear refresh token" }
    }

    fun getCustomBaseUrl(): String? {
        return basePrefs.getString(KEY_CUSTOM_BASE_URL, null)
    }

    fun setCustomBaseUrl(url: String?) {
        if (url.isNullOrBlank()) {
            basePrefs.edit().remove(KEY_CUSTOM_BASE_URL).apply()
        } else {
            basePrefs.edit().putString(KEY_CUSTOM_BASE_URL, url.trim().trimEnd('/')).apply()
        }
    }
}
