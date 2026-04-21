use std::env;

/// Application configuration loaded from environment variables
pub struct AppConfig {
    pub server_addr: String,
    pub broker_url: String,
    pub db_path: String,
    pub log_level: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        AppConfig {
            server_addr: env::var("ADMIN_API_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".to_string()),
            broker_url: env::var("BROKER_URL")
                .unwrap_or_else(|_| "http://localhost:6000".to_string()),
            db_path: env::var("ADMIN_DB_PATH")
                .unwrap_or_else(|_| "admin.db".to_string()),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| "info".to_string()),
        }
    }
}
