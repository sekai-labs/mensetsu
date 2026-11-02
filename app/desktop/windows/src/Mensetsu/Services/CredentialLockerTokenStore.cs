#nullable enable
using System;
using System.Runtime.InteropServices;
using Windows.Security.Credentials;

namespace Mensetsu.Services
{
    public interface ISecureTokenStore
    {
        void SaveRefreshToken(string refreshToken);
        string? GetRefreshToken();
        void ClearRefreshToken();
    }

    /// <summary>
    /// Implements secure storage for the rotating refresh_token using Windows Credential Locker
    /// (Windows.Security.Credentials.PasswordVault).
    /// Fails closed if unavailable or if vault operations fail.
    /// </summary>
    public class CredentialLockerTokenStore : ISecureTokenStore
    {
        private const string ResourceName = "MensetsuApp";
        private const string UserName = "RefreshToken";

        public void SaveRefreshToken(string refreshToken)
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException("Windows Credential Locker is only supported on Windows.");
            }

            if (string.IsNullOrEmpty(refreshToken))
            {
                ClearRefreshToken();
                return;
            }

            var vault = new PasswordVault();
            // Remove any existing credential under the same resource/user first to prevent duplicate key errors
            try
            {
                var existing = vault.Retrieve(ResourceName, UserName);
                if (existing != null)
                {
                    vault.Remove(existing);
                }
            }
            catch (Exception ex) when (IsCredentialNotFound(ex))
            {
                // Normal when no existing credential is in the vault
            }

            var cred = new PasswordCredential(ResourceName, UserName, refreshToken);
            // vault.Add throws on failure; we propagate it directly so auth fails closed
            vault.Add(cred);
        }

        public string? GetRefreshToken()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException("Windows Credential Locker is only supported on Windows.");
            }

            var vault = new PasswordVault();
            try
            {
                var cred = vault.Retrieve(ResourceName, UserName);
                cred.RetrievePassword();
                return cred.Password;
            }
            catch (Exception ex) when (IsCredentialNotFound(ex))
            {
                return null;
            }
        }

        public void ClearRefreshToken()
        {
            if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            {
                throw new PlatformNotSupportedException("Windows Credential Locker is only supported on Windows.");
            }

            var vault = new PasswordVault();
            try
            {
                var cred = vault.Retrieve(ResourceName, UserName);
                if (cred != null)
                {
                    vault.Remove(cred);
                }
            }
            catch (Exception ex) when (IsCredentialNotFound(ex))
            {
                // Normal when already empty
            }
        }

        private static bool IsCredentialNotFound(Exception ex)
        {
            // Windows error ERROR_NOT_FOUND (0x80070490) or Win32/COM message
            return ex.HResult == unchecked((int)0x80070490) ||
                   ex.Message.Contains("Element not found", StringComparison.OrdinalIgnoreCase) ||
                   ex.Message.Contains("not found", StringComparison.OrdinalIgnoreCase);
        }
    }
}
