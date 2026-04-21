use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde_json::json;
use std::fmt;
use chrono::Utc;

#[derive(Debug)]
pub enum AdminApiError {
    NotFound(String),
    BadRequest(String),
    Unauthorized(String),
    Conflict(String),
    InternalError(String),
    BrokerError(String),
    DatabaseError(String),
}

impl fmt::Display for AdminApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdminApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AdminApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AdminApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AdminApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AdminApiError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AdminApiError::BrokerError(msg) => write!(f, "Broker error: {}", msg),
            AdminApiError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl ResponseError for AdminApiError {
    fn error_response(&self) -> HttpResponse {
        let (status, error_type, message) = match self {
            AdminApiError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone()),
            AdminApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AdminApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", msg.clone()),
            AdminApiError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AdminApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.clone()),
            AdminApiError::BrokerError(msg) => (StatusCode::BAD_GATEWAY, "BROKER_ERROR", msg.clone()),
            AdminApiError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", msg.clone()),
        };

        HttpResponse::build(status).json(json!({
            "error": error_type,
            "code": status.as_u16(),
            "message": message,
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AdminApiError::NotFound(_) => StatusCode::NOT_FOUND,
            AdminApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AdminApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AdminApiError::Conflict(_) => StatusCode::CONFLICT,
            AdminApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AdminApiError::BrokerError(_) => StatusCode::BAD_GATEWAY,
            AdminApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type AdminResult<T> = Result<T, AdminApiError>;
