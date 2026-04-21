use serde::{Deserialize, Serialize};
use crate::error::{AdminApiError, AdminResult};
use chrono::Utc;
use std::fs;
use std::path::PathBuf;

const TENANTS_DIR: &str = "tenants";

/// Tenant data structure stored as JSON
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantData {
    pub tenant_id: String,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub status: String,
    pub max_message_size: i32,
    pub rate_limit_rps: i32,
    pub max_connections: i32,
    pub retention_days: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Secret data structure stored as JSON
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecretData {
    pub secret_id: String,
    pub tenant_id: String,
    pub secret_key: String,
    pub secret_value: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Initialize tenants directory
pub fn init_storage() -> AdminResult<()> {
    if !PathBuf::from(TENANTS_DIR).exists() {
        fs::create_dir(TENANTS_DIR)
            .map_err(|e| AdminApiError::DatabaseError(format!("Failed to create tenants directory: {}", e)))?;
    }
    Ok(())
}

/// Save tenant as JSON file
pub fn save_tenant(tenant: &TenantData) -> AdminResult<()> {
    init_storage()?;
    
    let filename = format!("{}/{}.json", TENANTS_DIR, &tenant.tenant_id);
    let json = serde_json::to_string_pretty(tenant)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to serialize tenant: {}", e)))?;
    
    fs::write(&filename, json)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to write tenant file: {}", e)))?;
    
    tracing::info!("Saved tenant: {} to {}", tenant.tenant_id, filename);
    Ok(())
}

/// Load tenant from JSON file
pub fn load_tenant(tenant_id: &str) -> AdminResult<TenantData> {
    let filename = format!("{}/{}.json", TENANTS_DIR, tenant_id);
    
    let json = fs::read_to_string(&filename)
        .map_err(|_| AdminApiError::NotFound(format!("Tenant not found: {}", tenant_id)))?;
    
    let tenant = serde_json::from_str(&json)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to parse tenant JSON: {}", e)))?;
    
    Ok(tenant)
}

/// List all tenants
pub fn list_all_tenants() -> AdminResult<Vec<TenantData>> {
    init_storage()?;
    
    let entries = fs::read_dir(TENANTS_DIR)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read tenants directory: {}", e)))?;
    
    let mut tenants = Vec::new();
    
    for entry in entries {
        let entry = entry
            .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read directory entry: {}", e)))?;
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let json = fs::read_to_string(&path)
                .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read file: {}", e)))?;
            
            if let Ok(tenant) = serde_json::from_str::<TenantData>(&json) {
                tenants.push(tenant);
            }
        }
    }
    
    // Sort by created_at descending
    tenants.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(tenants)
}

/// Delete tenant
pub fn delete_tenant(tenant_id: &str) -> AdminResult<()> {
    let filename = format!("{}/{}.json", TENANTS_DIR, tenant_id);
    
    fs::remove_file(&filename)
        .map_err(|_| AdminApiError::NotFound(format!("Tenant not found: {}", tenant_id)))?;
    
    tracing::info!("Deleted tenant: {}", tenant_id);
    Ok(())
}

/// Save tenant secret
pub fn save_secret(secret: &SecretData) -> AdminResult<()> {
    init_storage()?;
    
    let secret_dir = format!("{}/{}/.secrets", TENANTS_DIR, &secret.tenant_id);
    fs::create_dir_all(&secret_dir)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to create secrets directory: {}", e)))?;
    
    let filename = format!("{}/{}.json", secret_dir, secret.secret_id);
    let json = serde_json::to_string_pretty(secret)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to serialize secret: {}", e)))?;
    
    fs::write(&filename, json)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to write secret file: {}", e)))?;
    
    tracing::info!("Saved secret: {} for tenant: {}", secret.secret_id, secret.tenant_id);
    Ok(())
}

/// Load secret
pub fn load_secret(tenant_id: &str, secret_id: &str) -> AdminResult<SecretData> {
    let filename = format!("{}/{}/.secrets/{}.json", TENANTS_DIR, tenant_id, secret_id);
    
    let json = fs::read_to_string(&filename)
        .map_err(|_| AdminApiError::NotFound(format!("Secret not found: {}", secret_id)))?;
    
    let secret = serde_json::from_str(&json)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to parse secret JSON: {}", e)))?;
    
    Ok(secret)
}

/// List all secrets for a tenant
pub fn list_secrets(tenant_id: &str) -> AdminResult<Vec<SecretData>> {
    let secret_dir = format!("{}/{}/.secrets", TENANTS_DIR, tenant_id);
    
    if !PathBuf::from(&secret_dir).exists() {
        return Ok(Vec::new());
    }
    
    let entries = fs::read_dir(&secret_dir)
        .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read secrets directory: {}", e)))?;
    
    let mut secrets = Vec::new();
    
    for entry in entries {
        let entry = entry
            .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read directory entry: {}", e)))?;
        
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let json = fs::read_to_string(&path)
                .map_err(|e| AdminApiError::DatabaseError(format!("Failed to read file: {}", e)))?;
            
            if let Ok(secret) = serde_json::from_str::<SecretData>(&json) {
                secrets.push(secret);
            }
        }
    }
    
    Ok(secrets)
}

/// Delete secret
pub fn delete_secret(tenant_id: &str, secret_id: &str) -> AdminResult<()> {
    let filename = format!("{}/{}/.secrets/{}.json", TENANTS_DIR, tenant_id, secret_id);
    
    fs::remove_file(&filename)
        .map_err(|_| AdminApiError::NotFound(format!("Secret not found: {}", secret_id)))?;
    
    tracing::info!("Deleted secret: {} for tenant: {}", secret_id, tenant_id);
    Ok(())
}
