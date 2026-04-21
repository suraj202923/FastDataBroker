use actix_web::{web, HttpResponse};
use chrono::Utc;
use uuid::Uuid;
use serde_json;
use crate::{models::*, error::{AdminApiError, AdminResult}, AppState};

/// List all cluster environments
pub async fn list_cluster_environments(state: web::Data<AppState>) -> AdminResult<HttpResponse> {
    let clusters: Vec<ClusterEnvironment> = sqlx::query_as(
        "SELECT id, name, description, region, broker_addresses, replication_factor, status, created_at, updated_at FROM cluster_environments ORDER BY created_at DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let responses: Vec<ClusterEnvironmentResponse> = clusters.into_iter().map(|c| {
        let addresses: Vec<String> = serde_json::from_str(&c.broker_addresses).unwrap_or_default();
        ClusterEnvironmentResponse {
            id: c.id,
            name: c.name,
            description: c.description,
            region: c.region,
            broker_addresses: addresses,
            replication_factor: c.replication_factor,
            status: c.status,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }
    }).collect();

    Ok(HttpResponse::Ok().json(responses))
}

/// Get specific cluster environment
pub async fn get_cluster_environment(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let cluster: ClusterEnvironment = sqlx::query_as(
        "SELECT id, name, description, region, broker_addresses, replication_factor, status, created_at, updated_at FROM cluster_environments WHERE id = ?"
    )
    .bind(id.into_inner())
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?
    .ok_or_else(|| AdminApiError::NotFound("Cluster environment not found".to_string()))?;

    let addresses: Vec<String> = serde_json::from_str(&cluster.broker_addresses).unwrap_or_default();
    let response = ClusterEnvironmentResponse {
        id: cluster.id,
        name: cluster.name,
        description: cluster.description,
        region: cluster.region,
        broker_addresses: addresses,
        replication_factor: cluster.replication_factor,
        status: cluster.status,
        created_at: cluster.created_at,
        updated_at: cluster.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Create new cluster environment
pub async fn create_cluster_environment(
    state: web::Data<AppState>,
    req: web::Json<ClusterEnvironmentRequest>,
) -> AdminResult<HttpResponse> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let addresses_json = serde_json::to_string(&req.broker_addresses)
        .map_err(|e| AdminApiError::BadRequest(e.to_string()))?;
    let replication_factor = req.replication_factor.unwrap_or(3);

    sqlx::query(
        "INSERT INTO cluster_environments (id, name, description, region, broker_addresses, replication_factor, status, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&id)
    .bind(&req.name)
    .bind(&req.description.clone().unwrap_or_default())
    .bind(&req.region)
    .bind(&addresses_json)
    .bind(replication_factor)
    .bind("active")
    .bind(&now)
    .bind(&now)
    .execute(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("UNIQUE") {
            AdminApiError::Conflict("Cluster name already exists".to_string())
        } else {
            AdminApiError::DatabaseError(e.to_string())
        }
    })?;

    let response = ClusterEnvironmentResponse {
        id,
        name: req.name.clone(),
        description: req.description.clone().unwrap_or_default(),
        region: req.region.clone(),
        broker_addresses: req.broker_addresses.clone(),
        replication_factor,
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Update cluster environment
pub async fn update_cluster_environment(
    state: web::Data<AppState>,
    id: web::Path<String>,
    req: web::Json<ClusterEnvironmentRequest>,
) -> AdminResult<HttpResponse> {
    let cluster_id = id.into_inner();
    let now = Utc::now().to_rfc3339();
    let addresses_json = serde_json::to_string(&req.broker_addresses)
        .map_err(|e| AdminApiError::BadRequest(e.to_string()))?;

    sqlx::query(
        "UPDATE cluster_environments SET name = ?, description = ?, region = ?, broker_addresses = ?, replication_factor = ?, updated_at = ? WHERE id = ?"
    )
    .bind(&req.name)
    .bind(&req.description.clone().unwrap_or_default())
    .bind(&req.region)
    .bind(&addresses_json)
    .bind(req.replication_factor.unwrap_or(3))
    .bind(&now)
    .bind(&cluster_id)
    .execute(&state.db)
    .await
    .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?;

    let response = ClusterEnvironmentResponse {
        id: cluster_id,
        name: req.name.clone(),
        description: req.description.clone().unwrap_or_default(),
        region: req.region.clone(),
        broker_addresses: req.broker_addresses.clone(),
        replication_factor: req.replication_factor.unwrap_or(3),
        status: "active".to_string(),
        created_at: "".to_string(),
        updated_at: now,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Delete cluster environment
pub async fn delete_cluster_environment(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> AdminResult<HttpResponse> {
    let rows = sqlx::query("DELETE FROM cluster_environments WHERE id = ?")
        .bind(id.into_inner())
        .execute(&state.db)
        .await
        .map_err(|e| AdminApiError::DatabaseError(e.to_string()))?
        .rows_affected();

    if rows == 0 {
        return Err(AdminApiError::NotFound("Cluster environment not found".to_string()));
    }

    Ok(HttpResponse::NoContent().finish())
}
