use rusqlite::Connection;
use crate::error::{AdminApiError, AdminResult};
use chrono::Utc;

/// Initialize database schema
pub fn init_db(conn: &Connection) -> AdminResult<()> {
    // System configuration table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS system_config (
            id TEXT PRIMARY KEY,
            broker_url TEXT NOT NULL,
            max_brokers INTEGER DEFAULT 3,
            replication_factor INTEGER DEFAULT 3,
            log_level TEXT DEFAULT 'info',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Cluster environments table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS cluster_environments (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            region TEXT NOT NULL,
            broker_addresses TEXT NOT NULL,
            replication_factor INTEGER DEFAULT 3,
            status TEXT DEFAULT 'active',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Tenants table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tenants (
            tenant_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            api_key TEXT NOT NULL UNIQUE,
            status TEXT DEFAULT 'active',
            max_message_size INTEGER DEFAULT 10485760,
            rate_limit_rps INTEGER DEFAULT 1000,
            max_connections INTEGER DEFAULT 100,
            retention_days INTEGER DEFAULT 30,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Tenant secrets table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tenant_secrets (
            secret_id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            secret_key TEXT NOT NULL,
            secret_value TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id) ON DELETE CASCADE,
            UNIQUE(tenant_id, secret_key)
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Tenant usage tracking table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS tenant_usage (
            id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            messages_sent INTEGER DEFAULT 0,
            messages_received INTEGER DEFAULT 0,
            storage_used_mb REAL DEFAULT 0.0,
            bandwidth_used_gb REAL DEFAULT 0.0,
            active_connections INTEGER DEFAULT 0,
            last_activity TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id) ON DELETE CASCADE
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // SMTP configuration table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS smtp_config (
            id TEXT PRIMARY KEY,
            host TEXT NOT NULL,
            port INTEGER NOT NULL,
            username TEXT,
            password TEXT,
            from_email TEXT NOT NULL,
            use_tls BOOLEAN DEFAULT true,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Notification settings table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS notification_settings (
            id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL UNIQUE,
            enabled BOOLEAN DEFAULT true,
            recipient_email TEXT NOT NULL,
            notification_channels TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Notification events table (read-only - for audit)
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS notification_events (
            event_id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            severity TEXT NOT NULL,
            created_at TEXT NOT NULL
        )
        "#,
        [],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    // Create indexes for notification_events
    conn.execute("CREATE INDEX IF NOT EXISTS idx_notification_events_type ON notification_events(event_type)", [])
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    conn.execute("CREATE INDEX IF NOT EXISTS idx_notification_events_created_at ON notification_events(created_at)", [])
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    tracing::info!("Database initialized successfully");
    Ok(())
}

/// Insert default system configuration if not exists
pub fn insert_default_config(conn: &Connection) -> AdminResult<()> {
    let now = Utc::now().to_rfc3339();
    
    conn.execute(
        r#"
        INSERT OR IGNORE INTO system_config 
        (id, broker_url, max_brokers, replication_factor, log_level, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        rusqlite::params![
            "default",
            "http://localhost:6000",
            3,
            3,
            "info",
            &now,
            &now
        ],
    )
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    Ok(())
}
