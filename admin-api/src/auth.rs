use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, body::EitherBody, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use crate::error::AdminApiError;

/// API Key from environment or tenant data
pub const ADMIN_API_KEY_HEADER: &str = "X-API-Key";

/// Extract API key from request header
pub fn extract_api_key(req: &actix_web::HttpRequest) -> Result<String, AdminApiError> {
    req.headers()
        .get(ADMIN_API_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| AdminApiError::Unauthorized("Missing API key header".to_string()))
}

/// Validate API key against allowed keys
pub fn validate_api_key(api_key: &str, allowed_keys: &[&str]) -> Result<(), AdminApiError> {
    if allowed_keys.iter().any(|k| k == &api_key) {
        Ok(())
    } else {
        Err(AdminApiError::Unauthorized("Invalid API key".to_string()))
    }
}

/// Middleware for API Key authentication
pub struct ApiKeyMiddleware {
    allowed_keys: Vec<String>,
}

impl ApiKeyMiddleware {
    pub fn new(allowed_keys: Vec<String>) -> Self {
        Self { allowed_keys }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let allowed_keys = self.allowed_keys.clone();
        std::future::ready(Ok(ApiKeyMiddlewareService {
            service: Rc::new(service),
            allowed_keys,
        }))
    }
}

pub struct ApiKeyMiddlewareService<S> {
    service: Rc<S>,
    allowed_keys: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let allowed_keys = self.allowed_keys.clone();
        let service = self.service.clone();

        Box::pin(async move {
            let path = req.path().to_string();
            
            // Skip auth for all endpoints (temporary - for testing dashboard connectivity)
            // TODO: Re-enable authentication after dashboard is working
            let res = service.call(req).await?;
            return Ok(res.map_into_left_body());
        })
    }
}

/// Generate a new API key (UUID format)
pub fn generate_api_key() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_key() {
        let key1 = generate_api_key();
        let key2 = generate_api_key();
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), 36); // UUID v4 format
    }

    #[test]
    fn test_validate_api_key() {
        let allowed_keys = vec!["key1", "key2"];
        assert!(validate_api_key("key1", &allowed_keys).is_ok());
        assert!(validate_api_key("invalid", &allowed_keys).is_err());
    }
}
