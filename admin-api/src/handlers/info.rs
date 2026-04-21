use actix_web::HttpResponse;
use crate::models::*;
use crate::error::AdminResult;

/// Get API information and available endpoints
pub async fn api_info() -> AdminResult<HttpResponse> {
    let info = ApiInfo {
        name: "FastDataBroker Admin API".to_string(),
        version: "0.1.0".to_string(),
        description: "Lightweight REST API for managing FastDataBroker configuration, tenants, and system health".to_string(),
        endpoints: vec![
            // Health endpoints
            EndpointInfo {
                path: "/health".to_string(),
                method: "GET".to_string(),
                description: "Basic health check".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/health/detailed".to_string(),
                method: "GET".to_string(),
                description: "Detailed health status with metrics".to_string(),
                authentication: false,
            },
            // System configuration
            EndpointInfo {
                path: "/api/v1/system/config".to_string(),
                method: "GET".to_string(),
                description: "Get system configuration".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/system/config".to_string(),
                method: "PUT".to_string(),
                description: "Update system configuration".to_string(),
                authentication: false,
            },
            // Cluster management
            EndpointInfo {
                path: "/api/v1/cluster/environments".to_string(),
                method: "GET".to_string(),
                description: "List all cluster environments".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/cluster/environments".to_string(),
                method: "POST".to_string(),
                description: "Create new cluster environment".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/cluster/environments/{id}".to_string(),
                method: "GET".to_string(),
                description: "Get specific cluster environment".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/cluster/environments/{id}".to_string(),
                method: "PUT".to_string(),
                description: "Update cluster environment".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/cluster/environments/{id}".to_string(),
                method: "DELETE".to_string(),
                description: "Delete cluster environment".to_string(),
                authentication: false,
            },
            // Tenant management
            EndpointInfo {
                path: "/api/v1/tenants".to_string(),
                method: "GET".to_string(),
                description: "List all tenants".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants".to_string(),
                method: "POST".to_string(),
                description: "Create new tenant".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}".to_string(),
                method: "GET".to_string(),
                description: "Get tenant details".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}".to_string(),
                method: "PUT".to_string(),
                description: "Update tenant".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}".to_string(),
                method: "DELETE".to_string(),
                description: "Delete tenant".to_string(),
                authentication: false,
            },
            // Tenant secrets
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/secrets".to_string(),
                method: "GET".to_string(),
                description: "Get tenant secrets".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/secrets".to_string(),
                method: "POST".to_string(),
                description: "Create tenant secret".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/secrets".to_string(),
                method: "PUT".to_string(),
                description: "Update tenant secret".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/secrets/{secret_id}".to_string(),
                method: "DELETE".to_string(),
                description: "Delete tenant secret".to_string(),
                authentication: false,
            },
            // Tenant usage and limits
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/usage".to_string(),
                method: "GET".to_string(),
                description: "Get tenant usage statistics".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/limits".to_string(),
                method: "GET".to_string(),
                description: "Get tenant limits".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/limits".to_string(),
                method: "PUT".to_string(),
                description: "Update tenant limits".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/tenants/{tenant_id}/limits/reset".to_string(),
                method: "POST".to_string(),
                description: "Reset tenant limits to defaults".to_string(),
                authentication: false,
            },
            // Notifications - SMTP
            EndpointInfo {
                path: "/api/v1/notifications/smtp".to_string(),
                method: "GET".to_string(),
                description: "Get SMTP configuration".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/notifications/smtp".to_string(),
                method: "PUT".to_string(),
                description: "Update SMTP configuration".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/notifications/smtp/test".to_string(),
                method: "POST".to_string(),
                description: "Test SMTP connection".to_string(),
                authentication: false,
            },
            // Notifications - Settings
            EndpointInfo {
                path: "/api/v1/notifications/settings".to_string(),
                method: "GET".to_string(),
                description: "Get notification settings".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/notifications/settings".to_string(),
                method: "PUT".to_string(),
                description: "Update notification settings".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/notifications/events".to_string(),
                method: "GET".to_string(),
                description: "List notification events".to_string(),
                authentication: false,
            },
            EndpointInfo {
                path: "/api/v1/notifications/events/{event_id}".to_string(),
                method: "GET".to_string(),
                description: "Get specific notification event".to_string(),
                authentication: false,
            },
        ],
        documentation: "https://github.com/suraj202923/fastdatabroker/docs/admin-api".to_string(),
    };

    Ok(HttpResponse::Ok().json(info))
}
