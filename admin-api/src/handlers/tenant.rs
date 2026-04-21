use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use serde_json::json;
use crate::{error::AdminResult, json_store, AppState};

/// List all tenants
pub async fn list_tenants(_state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let tenants = json_store::list_all_tenants()?;
    Ok(HttpResponse::Ok().json(tenants))
}

/// Create new tenant
pub async fn create_tenant(
    _state: web::Data<AppState>,
    req: web::Json<serde_json::Value>,
) -> AdminResult<HttpResponse> {
    let tenant_id = format!("tenant_{}", Uuid::new_v4());
    let api_key = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let tenant = json_store::TenantData {
        tenant_id: tenant_id.clone(),
        name: req["name"].as_str().unwrap_or("").to_string(),
        email: req["email"].as_str().unwrap_or("").to_string(),
        api_key: api_key.clone(),
        status: "active".to_string(),
        max_message_size: req["max_message_size"].as_i64().unwrap_or(10485760) as i32,
        rate_limit_rps: req["rate_limit_rps"].as_i64().unwrap_or(1000) as i32,
        max_connections: req["max_connections"].as_i64().unwrap_or(100) as i32,
        retention_days: req["retention_days"].as_i64().unwrap_or(30) as i32,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    json_store::save_tenant(&tenant)?;
    Ok(HttpResponse::Created().json(json!({
        "tenant_id": tenant.tenant_id,
        "name": tenant.name,
        "email": tenant.email,
        "api_key": api_key,
        "status": tenant.status,
        "created_at": tenant.created_at,
        "updated_at": tenant.updated_at
    })))
}

/// Get tenant details
pub async fn get_tenant(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let tenant = json_store::load_tenant(&tenant_id)?;
    Ok(HttpResponse::Ok().json(tenant))
}

/// Update tenant
pub async fn update_tenant(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
    req: web::Json<serde_json::Value>,
) -> AdminResult<HttpResponse> {
    let mut tenant = json_store::load_tenant(&tenant_id)?;
    let now = Utc::now().to_rfc3339();

    if let Some(name) = req["name"].as_str() {
        tenant.name = name.to_string();
    }
    if let Some(email) = req["email"].as_str() {
        tenant.email = email.to_string();
    }
    if let Some(status) = req["status"].as_str() {
        tenant.status = status.to_string();
    }
    
    tenant.updated_at = now;
    json_store::save_tenant(&tenant)?;
    Ok(HttpResponse::Ok().json(tenant))
}

/// Delete tenant
pub async fn delete_tenant(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    json_store::delete_tenant(&tenant_id)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get tenant secrets
pub async fn get_tenant_secrets(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let secrets = json_store::list_secrets(&tenant_id)?;
    Ok(HttpResponse::Ok().json(secrets))
}

/// Create tenant secret
pub async fn create_tenant_secret(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
    req: web::Json<serde_json::Value>,
) -> AdminResult<HttpResponse> {
    json_store::load_tenant(&tenant_id)?;

    let secret_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let secret = json_store::SecretData {
        secret_id: secret_id.clone(),
        tenant_id: tenant_id.to_string(),
        secret_key: req["secret_key"].as_str().unwrap_or("").to_string(),
        secret_value: req["secret_value"].as_str().unwrap_or("").to_string(),
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    json_store::save_secret(&secret)?;
    Ok(HttpResponse::Created().json(secret))
}

/// Update tenant secret
pub async fn update_tenant_secret(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
    req: web::Json<serde_json::Value>,
) -> AdminResult<HttpResponse> {
    json_store::load_tenant(&tenant_id)?;
    let secret_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let secret = json_store::SecretData {
        secret_id,
        tenant_id: tenant_id.to_string(),
        secret_key: req["secret_key"].as_str().unwrap_or("").to_string(),
        secret_value: req["secret_value"].as_str().unwrap_or("").to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    json_store::save_secret(&secret)?;
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Secret updated"
    })))
}

/// Delete tenant secret
pub async fn delete_tenant_secret(
    _state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> AdminResult<HttpResponse> {
    let (tenant_id, secret_id) = path.into_inner();
    json_store::delete_secret(&tenant_id, &secret_id)?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get tenant usage
pub async fn get_tenant_usage(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    json_store::load_tenant(&tenant_id)?;
    Ok(HttpResponse::Ok().json(json!({
        "tenant_id": tenant_id.to_string(),
        "messages_sent": 0,
        "messages_received": 0,
        "storage_used_mb": 0.0,
        "bandwidth_used_gb": 0.0,
        "active_connections": 0,
        "last_activity": Utc::now().to_rfc3339()
    })))
}

/// Get tenant limits
pub async fn get_tenant_limits(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let tenant = json_store::load_tenant(&tenant_id)?;
    Ok(HttpResponse::Ok().json(json!({
        "max_message_size": tenant.max_message_size,
        "rate_limit_rps": tenant.rate_limit_rps,
        "max_connections": tenant.max_connections,
        "retention_days": tenant.retention_days
    })))
}

/// Update tenant limits
pub async fn update_tenant_limits(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
    req: web::Json<serde_json::Value>,
) -> AdminResult<HttpResponse> {
    let mut tenant = json_store::load_tenant(&tenant_id)?;
    let now = Utc::now().to_rfc3339();

    if let Some(rps) = req["rate_limit_rps"].as_i64() {
        tenant.rate_limit_rps = rps as i32;
    }
    if let Some(conn) = req["max_connections"].as_i64() {
        tenant.max_connections = conn as i32;
    }
    if let Some(msg_size) = req["max_message_size"].as_i64() {
        tenant.max_message_size = msg_size as i32;
    }
    if let Some(retention) = req["retention_days"].as_i64() {
        tenant.retention_days = retention as i32;
    }
    
    tenant.updated_at = now;
    json_store::save_tenant(&tenant)?;
    
    Ok(HttpResponse::Ok().json(json!({
        "max_message_size": tenant.max_message_size,
        "rate_limit_rps": tenant.rate_limit_rps,
        "max_connections": tenant.max_connections,
        "retention_days": tenant.retention_days
    })))
}

/// Reset tenant limits to defaults
pub async fn reset_tenant_limits(
    _state: web::Data<AppState>,
    tenant_id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let mut tenant = json_store::load_tenant(&tenant_id)?;
    let now = Utc::now().to_rfc3339();

    tenant.max_message_size = 10485760;
    tenant.rate_limit_rps = 1000;
    tenant.max_connections = 100;
    tenant.retention_days = 30;
    tenant.updated_at = now;

    json_store::save_tenant(&tenant)?;

    Ok(HttpResponse::Ok().json(json!({
        "max_message_size": tenant.max_message_size,
        "rate_limit_rps": tenant.rate_limit_rps,
        "max_connections": tenant.max_connections,
        "retention_days": tenant.retention_days
    })))
}
