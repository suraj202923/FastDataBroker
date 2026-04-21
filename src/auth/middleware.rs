// Authentication middleware for Actix-web
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, body::EitherBody, HttpResponse,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use crate::auth::Authorization;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        // Skip auth for login and health endpoints
        let path = req.path();
        if path.contains("/login") || path.contains("/health") || path.contains("/api/public") {
            let srv = self.service.clone();
            return Box::pin(async move {
                srv.call(req).await.map(|res| res.map_into_left_body())
            });
        }

        // Extract token from header
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| {
                if h.starts_with("Bearer ") {
                    Some(h[7..].to_string())
                } else {
                    None
                }
            });

        match token {
            Some(_token) => {
                // Token validation happens in handler
                let srv = self.service.clone();
                Box::pin(async move {
                    srv.call(req).await.map(|res| res.map_into_left_body())
                })
            }
            None => {
                Box::pin(async move {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Missing authorization token",
                        "details": "Please provide 'Authorization: Bearer <token>' header"
                    }));
                    Ok(req.into_response(response.map_into_right_body()))
                })
            }
        }
    }
}

/// Helper to extract token from request
pub fn extract_token(auth_header: Option<&str>) -> Option<String> {
    auth_header
        .and_then(|h| {
            if h.starts_with("Bearer ") {
                Some(h[7..].to_string())
            } else {
                None
            }
        })
}

/// Helper to validate token and return user/role
pub fn validate_token_helper(
    token: &str,
    auth: &Authorization,
) -> Result<(String, String), String> {
    auth.validate_token(token)
}
