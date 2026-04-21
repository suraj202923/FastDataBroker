// Authentication API handlers
use actix_web::{web, HttpResponse, Result, HttpRequest};
use serde_json::json;
use std::sync::Mutex;
use crate::auth::{
    AuthConfig, Authorization, LoginRequest, CreateUserRequest, extract_token, 
    validate_token_helper, UserInfo
};

pub struct AuthHandlers {
    pub auth: web::Data<Mutex<Authorization>>,
}

impl AuthHandlers {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            auth: web::Data::new(Mutex::new(Authorization::new(config))),
        }
    }

    // ==================== Public Endpoints ====================

    /// POST /api/auth/login - User login
    pub async fn login(&self, req: web::Json<LoginRequest>) -> Result<HttpResponse> {
        let mut auth = match self.auth.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return Ok(HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "error": "Internal server error"
                })))
            }
        };

        match auth.authenticate(&req.username, &req.password) {
            Ok(token) => {
                let expires_in = (token.expires_at.timestamp() - chrono::Utc::now().timestamp()) as i64;
                Ok(HttpResponse::Ok().json(json!({
                    "success": true,
                    "token": token.token,
                    "user": token.user,
                    "role": token.role,
                    "expires_in_seconds": expires_in,
                    "expires_at": token.expires_at.to_rfc3339(),
                })))
            }
            Err(err) => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": err,
                })))
            }
        }
    }

    /// POST /api/auth/logout - Revoke token
    pub async fn logout(&self, http_req: HttpRequest) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let mut auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match auth.revoke_token(&token) {
                    Ok(_) => {
                        Ok(HttpResponse::Ok().json(json!({
                            "success": true,
                            "message": "Logged out successfully"
                        })))
                    }
                    Err(err) => {
                        Ok(HttpResponse::BadRequest().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }

    /// GET /api/auth/me - Get current user info
    pub async fn get_current_user(&self, http_req: HttpRequest) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match validate_token_helper(&token, &auth) {
                    Ok((username, role)) => {
                        if let Some(user_info) = auth.get_user(&username) {
                            Ok(HttpResponse::Ok().json(json!({
                                "success": true,
                                "user": user_info,
                                "role": role,
                            })))
                        } else {
                            Ok(HttpResponse::NotFound().json(json!({
                                "success": false,
                                "error": "User not found"
                            })))
                        }
                    }
                    Err(err) => {
                        Ok(HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }

    // ==================== Protected Endpoints (Admin Only) ====================

    /// GET /api/auth/users - List all users (admin only)
    pub async fn list_users(&self, http_req: HttpRequest) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match validate_token_helper(&token, &auth) {
                    Ok((_username, role)) => {
                        if role != "admin" {
                            return Ok(HttpResponse::Forbidden().json(json!({
                                "success": false,
                                "error": "Admin role required"
                            })));
                        }

                        let users = auth.list_users();
                        Ok(HttpResponse::Ok().json(json!({
                            "success": true,
                            "users": users,
                            "total": users.len(),
                        })))
                    }
                    Err(err) => {
                        Ok(HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }

    /// POST /api/auth/users - Create new user (admin only)
    pub async fn create_user(
        &self,
        http_req: HttpRequest,
        req: web::Json<CreateUserRequest>,
    ) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let mut auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match validate_token_helper(&token, &auth) {
                    Ok((_username, role)) => {
                        if role != "admin" {
                            return Ok(HttpResponse::Forbidden().json(json!({
                                "success": false,
                                "error": "Admin role required"
                            })));
                        }

                        match auth.create_user(req.into_inner()) {
                            Ok(message) => {
                                Ok(HttpResponse::Created().json(json!({
                                    "success": true,
                                    "message": message,
                                })))
                            }
                            Err(err) => {
                                Ok(HttpResponse::BadRequest().json(json!({
                                    "success": false,
                                    "error": err,
                                })))
                            }
                        }
                    }
                    Err(err) => {
                        Ok(HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }

    /// PUT /api/auth/users/{username}/disable - Disable user (admin only)
    pub async fn disable_user(
        &self,
        http_req: HttpRequest,
        username: web::Path<String>,
    ) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let mut auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match validate_token_helper(&token, &auth) {
                    Ok((_user, role)) => {
                        if role != "admin" {
                            return Ok(HttpResponse::Forbidden().json(json!({
                                "success": false,
                                "error": "Admin role required"
                            })));
                        }

                        match auth.disable_user(&username) {
                            Ok(message) => {
                                Ok(HttpResponse::Ok().json(json!({
                                    "success": true,
                                    "message": message,
                                })))
                            }
                            Err(err) => {
                                Ok(HttpResponse::BadRequest().json(json!({
                                    "success": false,
                                    "error": err,
                                })))
                            }
                        }
                    }
                    Err(err) => {
                        Ok(HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }

    /// PUT /api/auth/users/{username}/enable - Enable user (admin only)
    pub async fn enable_user(
        &self,
        http_req: HttpRequest,
        username: web::Path<String>,
    ) -> Result<HttpResponse> {
        let token = extract_token(
            http_req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok()),
        );

        match token {
            Some(token) => {
                let mut auth = match self.auth.lock() {
                    Ok(guard) => guard,
                    Err(_) => {
                        return Ok(HttpResponse::InternalServerError().json(json!({
                            "success": false,
                            "error": "Internal server error"
                        })))
                    }
                };

                match validate_token_helper(&token, &auth) {
                    Ok((_user, role)) => {
                        if role != "admin" {
                            return Ok(HttpResponse::Forbidden().json(json!({
                                "success": false,
                                "error": "Admin role required"
                            })));
                        }

                        match auth.enable_user(&username) {
                            Ok(message) => {
                                Ok(HttpResponse::Ok().json(json!({
                                    "success": true,
                                    "message": message,
                                })))
                            }
                            Err(err) => {
                                Ok(HttpResponse::BadRequest().json(json!({
                                    "success": false,
                                    "error": err,
                                })))
                            }
                        }
                    }
                    Err(err) => {
                        Ok(HttpResponse::Unauthorized().json(json!({
                            "success": false,
                            "error": err
                        })))
                    }
                }
            }
            None => {
                Ok(HttpResponse::Unauthorized().json(json!({
                    "success": false,
                    "error": "Missing authorization token"
                })))
            }
        }
    }
}
