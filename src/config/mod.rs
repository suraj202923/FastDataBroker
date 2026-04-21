// ============================================================================
// Configuration Module - Multi-Tenant App Settings Management (v3.1)
// ============================================================================
// Supports both legacy configuration and new v3.1 JSON per-tenant architecture

pub mod tenant;
pub mod app_settings;

// v3.1 JSON Per-Tenant Architecture modules
pub mod startup_config;
pub mod tenant_json;
pub mod in_memory_cache;
pub mod background_workers;
pub mod config_manager;

pub use app_settings::{AppSettings, AppConfig, ServerConfig, LoggingConfig, FeatureFlags};
pub use tenant::{TenantConfig, TenantFeatures};

// v3.1 exports
pub use startup_config::StartupConfig;
pub use tenant_json::{TenantJsonConfig, TenantSettings, Credential, MetricsSnapshot};
pub use in_memory_cache::{CacheSystem, TenantConfigCache, MetricsCounters, PskVerificationCache};
pub use background_workers::{MetricsFlushWorker, ConfigReloadWorker, BackupWorker};
pub use config_manager::ConfigManager;

// Re-export commonly used types
pub type Config = AppSettings;
pub type Tenant = TenantConfig;
