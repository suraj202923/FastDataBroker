// ============================================================================
// FastDataBroker QUIC PSK (Pre-Shared Key) Authentication
// TLS 1.3 Pre-Shared Key authentication for client validation
// ============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use sha2::{Sha256, Digest};

/// PSK (Pre-Shared Key) for client authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreSharedKey {
    /// Unique identifier for this PSK
    pub psk_id: String,
    /// Hexadecimal pre-shared key (client knows this)
    pub psk_secret: String,
    /// Client ID associated with this key
    pub client_id: String,
    /// Tenant ID that owns this key
    pub tenant_id: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp (optional)
    pub expires_at: Option<DateTime<Utc>>,
    /// Is this key active
    pub is_active: bool,
    /// Key identity string sent during TLS handshake
    pub identity: String,
    /// Obfuscated ticket key (internal use)
    pub obfuscated_ticket_key: Option<String>,
    /// Last used timestamp
    pub last_used: Option<DateTime<Utc>>,
    /// Usage count
    pub usage_count: u64,
}

impl PreSharedKey {
    /// Create a new PSK
    pub fn new(
        client_id: &str,
        tenant_id: &str,
        psk_secret: String,
    ) -> Self {
        let psk_id = format!(
            "psk_{}_{}_{}",
            tenant_id,
            client_id,
            uuid::Uuid::new_v4().to_string().replace("-", "")
        );

        let identity = format!("{}:{}", tenant_id, client_id);

        PreSharedKey {
            psk_id,
            psk_secret,
            client_id: client_id.to_string(),
            tenant_id: tenant_id.to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            is_active: true,
            identity,
            obfuscated_ticket_key: None,
            last_used: None,
            usage_count: 0,
        }
    }

    /// Check if PSK is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Validate PSK secret (returns hash for comparison)
    pub fn hash_secret(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.psk_secret.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify a client PSK against stored PSK
    pub fn verify_secret(&self, provided_secret: &str) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(provided_secret.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        self.hash_secret() == hash
    }
}

/// PSK Authentication Manager
pub struct PskAuthManager {
    // Map of PSK ID -> PSK
    psks: Arc<RwLock<HashMap<String, PreSharedKey>>>,
    // Map of client identity -> PSK ID (for quick lookup)
    identity_index: Arc<RwLock<HashMap<String, String>>>,
    // Metrics
    pub psk_validations: Arc<std::sync::atomic::AtomicU32>,
    pub psk_failures: Arc<std::sync::atomic::AtomicU32>,
}

impl PskAuthManager {
    /// Create new PSK manager
    pub fn new() -> Self {
        PskAuthManager {
            psks: Arc::new(RwLock::new(HashMap::new())),
            identity_index: Arc::new(RwLock::new(HashMap::new())),
            psk_validations: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            psk_failures: Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }

    /// Register a PSK
    pub async fn register_psk(&self, psk: PreSharedKey) -> Result<String, String> {
        if psk.psk_id.is_empty() {
            return Err("PSK ID cannot be empty".to_string());
        }

        let psk_id = psk.psk_id.clone();
        let identity = psk.identity.clone();

        let mut psks = self.psks.write().await;
        let mut identity_index = self.identity_index.write().await;

        psks.insert(psk_id.clone(), psk);
        identity_index.insert(identity, psk_id.clone());

        info!("PSK registered: {}", psk_id);
        Ok(psk_id)
    }

    /// Validate PSK during handshake
    pub async fn validate_psk(
        &self,
        identity: &str,
        provided_secret: &str,
    ) -> Result<PreSharedKey, String> {
        // Lookup PSK by identity
        let identity_index = self.identity_index.read().await;
        let psk_id = identity_index
            .get(identity)
            .ok_or(format!("PSK not found for identity: {}", identity))?;

        let psks = self.psks.read().await;
        let psk = psks
            .get(psk_id)
            .ok_or(format!("PSK metadata not found: {}", psk_id))?;

        // Check if active
        if !psk.is_active {
            self.psk_failures.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Err("PSK is not active".to_string());
        }

        // Check if expired
        if psk.is_expired() {
            self.psk_failures.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Err("PSK has expired".to_string());
        }

        // Verify secret
        if !psk.verify_secret(provided_secret) {
            self.psk_failures.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            return Err("Invalid PSK secret".to_string());
        }

        self.psk_validations.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        debug!("PSK validated successfully: {} ({})", identity, psk_id);

        Ok(psk.clone())
    }

    /// Revoke a PSK
    pub async fn revoke_psk(&self, psk_id: &str) -> Result<(), String> {
        let mut psks = self.psks.write().await;
        
        match psks.get_mut(psk_id) {
            Some(psk) => {
                psk.is_active = false;
                info!("PSK revoked: {}", psk_id);
                Ok(())
            }
            None => Err(format!("PSK not found: {}", psk_id)),
        }
    }

    /// Generate a random PSK secret (hexadecimal)
    pub fn generate_psk_secret() -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        hex::encode(&bytes)
    }

    /// Get PSK by ID
    pub async fn get_psk(&self, psk_id: &str) -> Option<PreSharedKey> {
        let psks = self.psks.read().await;
        psks.get(psk_id).cloned()
    }

    /// List all PSKs for a tenant
    pub async fn list_tenant_psks(&self, tenant_id: &str) -> Vec<PreSharedKey> {
        let psks = self.psks.read().await;
        psks.values()
            .filter(|psk| psk.tenant_id == tenant_id)
            .cloned()
            .collect()
    }

    /// Get validation stats
    pub fn get_stats(&self) -> (u32, u32) {
        (
            self.psk_validations.load(std::sync::atomic::Ordering::SeqCst),
            self.psk_failures.load(std::sync::atomic::Ordering::SeqCst),
        )
    }
}

impl Default for PskAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psk_creation() {
        let psk = PreSharedKey::new("client1", "tenant1", "secret123".to_string());
        assert_eq!(psk.client_id, "client1");
        assert_eq!(psk.tenant_id, "tenant1");
        assert_eq!(psk.identity, "tenant1:client1");
        assert!(!psk.is_expired());
    }

    #[test]
    fn test_psk_verification() {
        let psk = PreSharedKey::new("client1", "tenant1", "secret123".to_string());
        assert!(psk.verify_secret("secret123"));
        assert!(!psk.verify_secret("wrong_secret"));
    }

    #[tokio::test]
    async fn test_psk_manager() {
        let manager = PskAuthManager::new();
        let psk = PreSharedKey::new("client1", "tenant1", "secret123".to_string());

        let psk_id = manager.register_psk(psk.clone()).await.unwrap();
        assert_eq!(psk_id, psk.psk_id);

        let result = manager
            .validate_psk("tenant1:client1", "secret123")
            .await;
        assert!(result.is_ok());

        let result = manager.validate_psk("tenant1:client1", "wrong_secret").await;
        assert!(result.is_err());
    }
}
