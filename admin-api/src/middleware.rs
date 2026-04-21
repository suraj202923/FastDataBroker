use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, body::EitherBody,
};
use futures::future::LocalBoxFuture;
use std::rc::Rc;
use uuid::Uuid;

/// Middleware to add request tracking ID to all requests
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_service(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RequestIdMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = Uuid::new_v4().to_string();
        let path = req.path().to_string();
        let method = req.method().to_string();

        // Add request ID to extensions for logging
        req.extensions_mut().insert(request_id.clone());

        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            "Incoming request"
        );

        let service = self.service.clone();

        Box::pin(async move {
            let res = service.call(req).await?;
            tracing::info!(
                request_id = %request_id,
                status = res.status().as_u16(),
                "Request completed"
            );
            Ok(res)
        })
    }
}

/// Extract request ID from request extensions
pub fn get_request_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_generation() {
        let id = Uuid::new_v4().to_string();
        assert_eq!(id.len(), 36);
    }
}
