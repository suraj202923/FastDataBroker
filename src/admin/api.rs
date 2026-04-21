// Admin API Server - Routes and setup
use actix_web::{web, App, HttpServer, middleware};
use crate::admin::handlers::AdminHandlers;
use crate::admin::logs::TenantLogger;
use crate::admin::metrics::MetricsCollector;
use crate::config::AppSettings;
use std::sync::Arc;

pub struct AdminApiServer;

impl AdminApiServer {
    pub async fn start(
        bind_address: &str,
        app_settings: web::Data<AppSettings>,
        logger: web::Data<TenantLogger>,
        metrics: web::Data<MetricsCollector>,
    ) -> std::io::Result<()> {
        println!("🚀 Starting Admin API on {}", bind_address);

        HttpServer::new(move || {
            let handlers = AdminHandlers::new(
                app_settings.clone(),
                logger.clone(),
                metrics.clone(),
            );

            App::new()
                .app_data(app_settings.clone())
                .app_data(logger.clone())
                .app_data(metrics.clone())
                .wrap(middleware::Logger::default())
                .configure(Self::config_routes(handlers))
        })
        .bind(bind_address)?
        .run()
        .await
    }

    fn config_routes(handlers: AdminHandlers) -> impl Fn(&mut web::ServiceConfig) {
        let handlers = Arc::new(handlers);

        move |cfg: &mut web::ServiceConfig| {
            let h = handlers.clone();

            cfg.service(
                web::scope("/api/admin")
                    // Health check
                    .route("/health", web::get().to({
                        let h = h.clone();
                        move || {
                            let h = h.clone();
                            async move { h.health().await }
                        }
                    }))
                    // Tenant Configuration
                    .route("/tenants", web::get().to({
                        let h = h.clone();
                        move || {
                            let h = h.clone();
                            async move { h.list_tenants().await }
                        }
                    }))
                    .route("/tenants/{tenant_id}", web::get().to({
                        let h = h.clone();
                        move |id| {
                            let h = h.clone();
                            async move { h.get_tenant(id).await }
                        }
                    }))
                    .route("/tenants/{tenant_id}", web::put().to({
                        let h = h.clone();
                        move |id, req| {
                            let h = h.clone();
                            async move { h.update_tenant(id, req).await }
                        }
                    }))
                    // Logs
                    .route("/tenants/{tenant_id}/logs", web::get().to({
                        let h = h.clone();
                        move |id, query| {
                            let h = h.clone();
                            async move { h.get_logs(id, query).await }
                        }
                    }))
                    .route("/tenants/{tenant_id}/errors", web::get().to({
                        let h = h.clone();
                        move |id, query| {
                            let h = h.clone();
                            async move { h.get_error_logs(id, query).await }
                        }
                    }))
                    .route("/tenants/{tenant_id}/rate-limits", web::get().to({
                        let h = h.clone();
                        move |id, query| {
                            let h = h.clone();
                            async move { h.get_rate_limit_logs(id, query).await }
                        }
                    }))
                    .route("/tenants/{tenant_id}/log-stats", web::get().to({
                        let h = h.clone();
                        move |id| {
                            let h = h.clone();
                            async move { h.get_log_stats(id).await }
                        }
                    }))
                    // Metrics & Limits
                    .route("/tenants/{tenant_id}/metrics", web::get().to({
                        let h = h.clone();
                        move |id| {
                            let h = h.clone();
                            async move { h.get_metrics(id).await }
                        }
                    }))
                    .route("/tenants/{tenant_id}/limits", web::get().to({
                        let h = h.clone();
                        move |id| {
                            let h = h.clone();
                            async move { h.get_limits(id).await }
                        }
                    }))
            )
        }
    }
}
