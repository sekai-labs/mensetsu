use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;

use crate::shared::error::{AppError, Result};

#[derive(Clone)]
pub struct CryptoService {
    key: [u8; 32],
}

impl CryptoService {
    pub fn new(raw_key_hex_or_secret: &str) -> Self {
        let mut key = [0u8; 32];
        let bytes = raw_key_hex_or_secret.as_bytes();
        if raw_key_hex_or_secret.len() == 64 && raw_key_hex_or_secret.chars().all(|c| c.is_ascii_hexdigit()) {
            if let Ok(decoded) = hex::decode(raw_key_hex_or_secret) {
                if decoded.len() == 32 {
                    key.copy_from_slice(&decoded);
                    return Self { key };
                }
            }
        }

        
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        let result = hasher.finalize();
        key.copy_from_slice(&result);

        Self { key }
    }

    
    
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| AppError::Internal(format!("Crypto key init error: {e}")))?;

        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::Internal(format!("Encryption failure: {e}")))?;

        let mut combined = Vec::with_capacity(12 + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(combined))
    }

    
    pub fn decrypt(&self, payload_b64: &str) -> Result<String> {
        let combined = BASE64
            .decode(payload_b64)
            .map_err(|e| AppError::Internal(format!("Base64 decode failure: {e}")))?;

        if combined.len() < 12 {
            return Err(AppError::Internal("Invalid encrypted payload: too short".into()));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .map_err(|e| AppError::Internal(format!("Crypto key init error: {e}")))?;

        let decrypted_bytes = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Internal(format!("Decryption failure or auth tag mismatch: {e}")))?;

        String::from_utf8(decrypted_bytes)
            .map_err(|e| AppError::Internal(format!("Invalid UTF-8 plaintext: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let crypto = CryptoService::new("my-super-secret-development-key-32chars!");
        let secret = "sk-ant-api03-test-token-123456789";

        let encrypted = crypto.encrypt(secret).expect("encryption succeeds");
        assert_ne!(secret, encrypted);

        let decrypted = crypto.decrypt(&encrypted).expect("decryption succeeds");
        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_corrupted_ciphertext_fails() {
        let crypto = CryptoService::new("key123");
        let secret = "openai-api-key-abc";
        let mut encrypted = crypto.encrypt(secret).unwrap();
        
        encrypted.push('A');
        assert!(crypto.decrypt(&encrypted).is_err());
    }
}
