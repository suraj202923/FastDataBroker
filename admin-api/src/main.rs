use actix_web::{web, App, HttpServer, middleware};

mod models;
mod handlers;
mod json_store;
mod broker;
mod error;
mod config;
mod auth;
mod cache;

use models::*;
use error::*;
use json_store::init_storage;
use cache::JsonCache;

/// Application state shared across handlers
pub struct AppState {
    broker_url: String,
    cache: JsonCache,
}

/// Main entry point for the admin API
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        )
        .with_target(true)
        .with_thread_ids(true)
        .init();

    // Load configuration
    let config = config::AppConfig::from_env();
    tracing::info!("Starting FastDataBroker Admin API on {}", config.server_addr);
    tracing::info!("Broker URL: {}", config.broker_url);

    // Initialize JSON storage
    init_storage()
        .expect("Failed to initialize storage");

    // Initialize cache (1000 entries max)
    let cache = JsonCache::new(1000);
    tracing::info!("Cache initialized with capacity: 1000");

    // Build list of allowed admin API keys from environment
    let admin_keys = std::env::var("ADMIN_API_KEYS")
        .unwrap_or_else(|_| "admin-key-default-change-me".to_string())
        .split(',')
        .map(|k| k.trim().to_string())
        .collect::<Vec<_>>();
    
    tracing::info!("Admin API keys loaded: {} keys available", admin_keys.len());

    let app_state = web::Data::new(AppState {
        broker_url: config.broker_url.clone(),
        cache,
    });

    tracing::info!("Admin API initialized successfully");

    // Start HTTP server
    HttpServer::new(move || {
        let admin_keys = admin_keys.clone();
        
        App::new()
            .app_data(app_state.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::trim())
            .wrap(auth::ApiKeyMiddleware::new(admin_keys))
            // Health endpoints (no auth required)
            .route("/health", web::get().to(handlers::health::health_check))
            .route("/health/detailed", web::get().to(handlers::health::health_detailed))
            // API documentation
            .route("/openapi.json", web::get().to(openapi_spec))
            .service(actix_files::Files::new("/swagger-ui", "./swagger-ui").show_files_listing())
            // Tenant management endpoints
            .route("/api/v1/tenants", web::get().to(handlers::tenant::list_tenants))
            .route("/api/v1/tenants", web::post().to(handlers::tenant::create_tenant))
            .route("/api/v1/tenants/{tenant_id}", web::get().to(handlers::tenant::get_tenant))
            .route("/api/v1/tenants/{tenant_id}", web::put().to(handlers::tenant::update_tenant))
            .route("/api/v1/tenants/{tenant_id}", web::delete().to(handlers::tenant::delete_tenant))
            // Tenant secrets endpoints
            .route("/api/v1/tenants/{tenant_id}/secrets", web::get().to(handlers::tenant::get_tenant_secrets))
            .route("/api/v1/tenants/{tenant_id}/secrets", web::post().to(handlers::tenant::create_tenant_secret))
            .route("/api/v1/tenants/{tenant_id}/secrets", web::put().to(handlers::tenant::update_tenant_secret))
            .route("/api/v1/tenants/{tenant_id}/secrets/{secret_id}", web::delete().to(handlers::tenant::delete_tenant_secret))
            // Tenant usage and limits endpoints
            .route("/api/v1/tenants/{tenant_id}/usage", web::get().to(handlers::tenant::get_tenant_usage))
            .route("/api/v1/tenants/{tenant_id}/limits", web::get().to(handlers::tenant::get_tenant_limits))
            .route("/api/v1/tenants/{tenant_id}/limits", web::put().to(handlers::tenant::update_tenant_limits))
            .route("/api/v1/tenants/{tenant_id}/limits/reset", web::post().to(handlers::tenant::reset_tenant_limits))
    })
    .bind(&config.server_addr)?
    .run()
    .await
}

/// Simple OpenAPI spec endpoint
async fn openapi_spec() -> actix_web::HttpResponse {
    let spec = serde_json::json!({
        "openapi": "3.0.0",
        "info": {
            "title": "FastDataBroker Admin API",
            "version": "0.1.0",
            "description": "Admin API for FastDataBroker - Tenant and configuration management"
        },
        "servers": [
            {
                "url": "http://localhost:8080",
                "description": "Development server"
            }
        ],
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                }
            }
        },
        "security": [
            {
                "ApiKeyAuth": []
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "tags": ["Health"],
                    "summary": "Health check (no auth)",
                    "responses": {
                        "200": {
                            "description": "Server is healthy"
                        }
                    }
                }
            },
            "/api/v1/tenants": {
                "get": {
                    "tags": ["Tenants"],
                    "summary": "List all tenants",
                    "responses": {
                        "200": {
                            "description": "List of tenants"
                        },
                        "401": {
                            "description": "Unauthorized - missing or invalid API key"
                        }
                    }
                },
                "post": {
                    "tags": ["Tenants"],
                    "summary": "Create new tenant",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object"
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Tenant created"
                        },
                        "401": {
                            "description": "Unauthorized"
                        }
                    }
                }
            },
            "/api/v1/tenants/{tenant_id}": {
                "get": {
                    "tags": ["Tenants"],
                    "summary": "Get tenant details",
                    "parameters": [
                        {
                            "name": "tenant_id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Tenant details"
                        },
                        "404": {
                            "description": "Tenant not found"
                        }
                    }
                }
            }
        }
    });
    
    actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .json(spec)
}

