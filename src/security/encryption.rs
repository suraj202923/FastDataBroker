// Phase 5: Message Encryption with AES-GCM
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::fmt;

/// Encryption configuration
#[derive(Debug, Clone)]
pub struct EncryptionConfig {
    /// Enable encryption for messages
    pub enabled: bool,
    /// Encryption key (32 bytes for AES-256)
    pub key: Vec<u8>,
    /// Algorithm: "AES-256-GCM" (future: ChaCha20Poly1305)
    pub algorithm: String,
}

impl EncryptionConfig {
    /// Create new encryption config with random key
    pub fn new_random() -> Result<Self> {
        let mut rng = rand::thread_rng();
        let mut key = vec![0u8; 32]; // 32 bytes = 256 bits for AES-256
        rng.fill(&mut key[..]);
        
        Ok(Self {
            enabled: true,
            key,
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Create new encryption config with provided key
    pub fn with_key(key: Vec<u8>) -> Result<Self> {
        if key.len() != 32 {
            return Err(anyhow::anyhow!(
                "Invalid key length: expected 32 bytes, got {}",
                key.len()
            ));
        }

        Ok(Self {
            enabled: true,
            key,
            algorithm: "AES-256-GCM".to_string(),
        })
    }

    /// Disable encryption
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            key: vec![],
            algorithm: "NONE".to_string(),
        }
    }
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Encrypted message payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Initialization vector (12 bytes for AES-GCM)
    pub nonce: Vec<u8>,
    /// Encrypted content
    pub ciphertext: Vec<u8>,
    /// Authentication tag
    pub tag_or_combined: Vec<u8>,
    /// Encryption algorithm used
    pub algorithm: String,
}

/// Message encryptor/decryptor
pub struct MessageEncryptor {
    config: EncryptionConfig,
    cipher: Option<Aes256Gcm>,
}

impl MessageEncryptor {
    /// Create new message encryptor
    pub fn new(config: EncryptionConfig) -> Result<Self> {
        let cipher = if config.enabled {
            Some(Aes256Gcm::new_from_slice(&config.key)
                .map_err(|e| anyhow::anyhow!("Failed to create cipher: {}", e))?
                .into())
        } else {
            None
        };

        Ok(Self { config, cipher })
    }

    /// Encrypt message
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedMessage> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        let cipher = self.cipher.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Cipher not initialized"))?;

        // Generate random nonce (12 bytes for AES-GCM)
        let mut rng = rand::thread_rng();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, Payload::from(plaintext))
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedMessage {
            nonce: nonce_bytes.to_vec(),
            ciphertext,
            tag_or_combined: vec![], // AES-GCM combines tag with ciphertext
            algorithm: self.config.algorithm.clone(),
        })
    }

    /// Decrypt message
    pub fn decrypt(&self, encrypted: &EncryptedMessage) -> Result<Vec<u8>> {
        if !self.config.enabled {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        if encrypted.algorithm != self.config.algorithm {
            return Err(anyhow::anyhow!(
                "Algorithm mismatch: expected {}, got {}",
                self.config.algorithm,
                encrypted.algorithm
            ));
        }

        let cipher = self.cipher.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Cipher not initialized"))?;

        if encrypted.nonce.len() != 12 {
            return Err(anyhow::anyhow!(
                "Invalid nonce length: expected 12, got {}",
                encrypted.nonce.len()
            ));
        }

        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, Payload::from(encrypted.ciphertext.as_slice()))
            .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Encrypt string message
    pub fn encrypt_string(&self, message: &str) -> Result<EncryptedMessage> {
        self.encrypt(message.as_bytes())
    }

    /// Decrypt to string
    pub fn decrypt_string(&self, encrypted: &EncryptedMessage) -> Result<String> {
        let plaintext = self.decrypt(encrypted)?;
        String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Failed to decode UTF-8: {}", e))
    }

    /// Get current encryption state
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl Clone for MessageEncryptor {
    fn clone(&self) -> Self {
        Self::new(self.config.clone())
            .expect("Failed to clone encryptor")
    }
}

impl fmt::Debug for MessageEncryptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MessageEncryptor")
            .field("enabled", &self.config.enabled)
            .field("algorithm", &self.config.algorithm)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_config_new_random() {
        let config = EncryptionConfig::new_random();
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.enabled);
        assert_eq!(config.key.len(), 32);
        assert_eq!(config.algorithm, "AES-256-GCM");
    }

    #[test]
    fn test_encryption_config_with_key() {
        let key = vec![0u8; 32];
        let config = EncryptionConfig::with_key(key);
        assert!(config.is_ok());
    }

    #[test]
    fn test_encryption_config_invalid_key_length() {
        let key = vec![0u8; 16]; // Wrong size
        let config = EncryptionConfig::with_key(key);
        assert!(config.is_err());
    }

    #[test]
    fn test_encryption_config_disabled() {
        let config = EncryptionConfig::disabled();
        assert!(!config.enabled);
        assert_eq!(config.algorithm, "NONE");
    }

    #[test]
    fn test_message_encryptor_encrypt_decrypt() {
        let config = EncryptionConfig::new_random().unwrap();
        let encryptor = MessageEncryptor::new(config).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = encryptor.encrypt(plaintext).unwrap();
        assert_eq!(encrypted.algorithm, "AES-256-GCM");
        assert_eq!(encrypted.nonce.len(), 12);
        assert!(!encrypted.ciphertext.is_empty());

        let decrypted = encryptor.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_message_encryptor_encrypt_string() {
        let config = EncryptionConfig::new_random().unwrap();
        let encryptor = MessageEncryptor::new(config).unwrap();

        let message = "Secret message";
        let encrypted = encryptor.encrypt_string(message).unwrap();
        let decrypted = encryptor.decrypt_string(&encrypted).unwrap();
        assert_eq!(decrypted, message);
    }

    #[test]
    fn test_message_encryptor_disabled() {
        let config = EncryptionConfig::disabled();
        let encryptor = MessageEncryptor::new(config).unwrap();
        assert!(!encryptor.is_enabled());

        let result = encryptor.encrypt(b"test");
        assert!(result.is_err());
    }

    #[test]
    fn test_message_encryptor_different_nonces() {
        let config = EncryptionConfig::new_random().unwrap();
        let encryptor = MessageEncryptor::new(config).unwrap();

        let plaintext = b"Same message";
        let encrypted1 = encryptor.encrypt(plaintext).unwrap();
        let encrypted2 = encryptor.encrypt(plaintext).unwrap();

        // Nonces should be different
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        // But both should decrypt to same plaintext
        assert_eq!(encryptor.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&encrypted2).unwrap(), plaintext);
    }
}
