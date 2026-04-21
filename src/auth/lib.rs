// Auth module exports
pub mod middleware;

pub use middleware::{AuthMiddleware, extract_token, validate_token_helper};

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub roles: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    #[serde(skip)]
    pub active_tokens: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub user: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub user: Option<String>,
    pub expires_in_seconds: i64,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthConfig {
    pub users: Vec<User>,
    pub token_expiry_hours: u32,
    pub jwt_secret: String,
}

pub struct Authorization {
    pub users: Vec<User>,
    pub token_expiry_hours: u32,
    pub jwt_secret: String,
}

impl Authorization {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            users: config.users,
            token_expiry_hours: config.token_expiry_hours,
            jwt_secret: config.jwt_secret,
        }
    }

    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_password(password: &str, hash: &str) -> bool {
        Self::hash_password(password) == hash
    }

    pub fn find_user(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    pub fn find_user_mut(&mut self, username: &str) -> Option<&mut User> {
        self.users.iter_mut().find(|u| u.username == username)
    }

    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<AuthToken, String> {
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        if !user.enabled {
            return Err("User is disabled".to_string());
        }

        if !Self::verify_password(password, &user.password_hash) {
            return Err("Invalid password".to_string());
        }

        let token = self.generate_token(username)?;

        if let Some(user) = self.find_user_mut(username) {
            user.last_login = Some(Utc::now());
            user.active_tokens.push(token.token.clone());
        }

        Ok(token)
    }

    pub fn generate_token(&self, username: &str) -> Result<AuthToken, String> {
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        let role = user.roles.first().cloned().unwrap_or_else(|| "viewer".to_string());
        let expires_at = Utc::now() + Duration::hours(self.token_expiry_hours as i64);

        let timestamp = Utc::now().timestamp();
        let token = format!(
            "{}:{}:{}",
            username,
            timestamp,
            Self::hash_password(&format!("{}{}{}secret", username, timestamp, self.jwt_secret))
                .chars()
                .take(32)
                .collect::<String>()
        );

        Ok(AuthToken {
            token,
            user: username.to_string(),
            role,
            expires_at,
            created_at: Utc::now(),
        })
    }

    pub fn validate_token(&self, token: &str) -> Result<(String, String), String> {
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 3 {
            return Err("Invalid token format".to_string());
        }

        let username = parts[0];

        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        if !user.enabled {
            return Err("User is disabled".to_string());
        }

        if !user.active_tokens.contains(&token.to_string()) {
            return Err("Token not active".to_string());
        }

        let role = user.roles.first().cloned().unwrap_or_else(|| "viewer".to_string());

        Ok((username.to_string(), role))
    }

    pub fn create_user(&mut self, req: CreateUserRequest) -> Result<String, String> {
        if self.find_user(&req.username).is_some() {
            return Err("User already exists".to_string());
        }

        let user = User {
            username: req.username.clone(),
            email: req.email,
            password_hash: Self::hash_password(&req.password),
            roles: if req.roles.is_empty() {
                vec!["viewer".to_string()]
            } else {
                req.roles
            },
            enabled: true,
            created_at: Utc::now(),
            last_login: None,
            active_tokens: Vec::new(),
        };

        self.users.push(user);
        Ok(format!("User '{}' created successfully", req.username))
    }

    pub fn list_users(&self) -> Vec<UserInfo> {
        self.users
            .iter()
            .map(|u| UserInfo {
                username: u.username.clone(),
                email: u.email.clone(),
                roles: u.roles.clone(),
                enabled: u.enabled,
                created_at: u.created_at,
                last_login: u.last_login,
            })
            .collect()
    }

    pub fn get_user(&self, username: &str) -> Option<UserInfo> {
        self.find_user(username).map(|u| UserInfo {
            username: u.username.clone(),
            email: u.email.clone(),
            roles: u.roles.clone(),
            enabled: u.enabled,
            created_at: u.created_at,
            last_login: u.last_login,
        })
    }

    pub fn disable_user(&mut self, username: &str) -> Result<String, String> {
        if let Some(user) = self.find_user_mut(username) {
            user.enabled = false;
            user.active_tokens.clear();
            Ok(format!("User '{}' disabled", username))
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn enable_user(&mut self, username: &str) -> Result<String, String> {
        if let Some(user) = self.find_user_mut(username) {
            user.enabled = true;
            Ok(format!("User '{}' enabled", username))
        } else {
            Err("User not found".to_string())
        }
    }

    pub fn change_password(&mut self, username: &str, old_password: &str, new_password: &str) -> Result<String, String> {
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        if !Self::verify_password(old_password, &user.password_hash) {
            return Err("Old password incorrect".to_string());
        }

        if let Some(user) = self.find_user_mut(username) {
            user.password_hash = Self::hash_password(new_password);
            user.active_tokens.clear();
        }

        Ok("Password changed successfully".to_string())
    }

    pub fn revoke_token(&mut self, token: &str) -> Result<String, String> {
        for user in &mut self.users {
            if let Some(pos) = user.active_tokens.iter().position(|t| t == token) {
                user.active_tokens.remove(pos);
                return Ok("Token revoked successfully".to_string());
            }
        }
        Err("Token not found".to_string())
    }
}
