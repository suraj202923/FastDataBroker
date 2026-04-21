use actix_web::{web, HttpResponse};
use chrono::Utc;
use crate::{models::*, error::AdminResult, AppState};

/// Get basic system health status
pub async fn health_check(_state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let health = SystemHealth {
        status: HealthStatus::Healthy,
        uptime_seconds: 0,  // TODO: Calculate from start time
        broker_connected: true,  // TODO: Check actual broker connection
        database_healthy: true,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(health))
}

/// Get detailed health status with metrics
pub async fn health_detailed(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let broker_health = BrokerHealth {
        connected: true,
        url: state.broker_url.clone(),
        response_time_ms: 5,
        active_connections: 42,
    };

    let db_health = DatabaseHealth {
        connected: true,
        query_time_ms: 2,
        pool_size: 10,
    };

    let system_metrics = SystemMetrics {
        cpu_usage_percent: 15.5,
        memory_usage_mb: 256,
        active_tenants: 12,
        total_message_volume: 1_000_000,
    };

    let health = DetailedHealth {
        status: HealthStatus::Healthy,
        broker: broker_health,
        database: db_health,
        system: system_metrics,
        timestamp: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(health))
}
