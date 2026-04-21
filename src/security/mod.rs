// Phase 5: Security Module
pub mod encryption;
pub mod quic_auth;
pub mod psk_auth;

pub use encryption::{MessageEncryptor, EncryptionConfig, EncryptedMessage};
pub use quic_auth::QuicAuthValidator;
pub use psk_auth::{PreSharedKey, PskAuthManager};
