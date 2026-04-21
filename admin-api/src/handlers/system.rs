use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use crate::{models::*, error::{AdminApiError, AdminResult}, AppState};

/// Get system configuration
pub async fn get_system_config(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let config: SystemConfig = sqlx::query_as(
        "SELECT id, broker_url, max_brokers, replication_factor, log_level, created_at, updated_at FROM system_config WHERE id = ?",
    )
    .bind("default")
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AdminApiError::NotFound("System configuration not found".to_string()))?;

    Ok(HttpResponse::Ok().json(config))
}

/// Update system configuration
pub async fn update_system_config(
    state: web::Data<AppState>,
    req: web::Json<SystemConfigRequest>,
) -> AdminResult<HttpResponse> {
    let now = Utc::now().to_rfc3339();

    let broker_url = req.broker_url.clone().unwrap_or_else(|| state.broker_url.clone());
    let max_brokers = req.max_brokers.unwrap_or(3);
    let replication_factor = req.replication_factor.unwrap_or(3);
    let log_level = req.log_level.clone().unwrap_or_else(|| "info".to_string());

    sqlx::query(
        "UPDATE system_config SET broker_url = ?, max_brokers = ?, replication_factor = ?, log_level = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&broker_url)
    .bind(max_brokers)
    .bind(replication_factor)
    .bind(&log_level)
    .bind(&now)
    .bind("default")
    .execute(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let config = SystemConfig {
        id: "default".to_string(),
        broker_url,
        max_brokers,
        replication_factor,
        log_level,
        created_at: "".to_string(),
        updated_at: now,
    };

    Ok(HttpResponse::Ok().json(config))
}
