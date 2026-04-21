// Authentication module - Token-based authorization
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,  // SHA256 hash
    pub roles: Vec<String>,     // e.g., ["admin", "operator", "viewer"]
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
pub struct Authorization {
    pub users: Vec<User>,
    pub token_expiry_hours: u32,
    pub jwt_secret: String,
}

impl Authorization {
    pub fn new(users: Vec<User>, token_expiry_hours: u32, jwt_secret: String) -> Self {
        Self {
            users,
            token_expiry_hours,
            jwt_secret,
        }
    }

    /// Hash password using SHA256
    pub fn hash_password(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify password against hash
    pub fn verify_password(password: &str, hash: &str) -> bool {
        Self::hash_password(password) == hash
    }

    /// Find user by username
    pub fn find_user(&self, username: &str) -> Option<&User> {
        self.users.iter().find(|u| u.username == username)
    }

    /// Find user by username (mutable)
    pub fn find_user_mut(&mut self, username: &str) -> Option<&mut User> {
        self.users.iter_mut().find(|u| u.username == username)
    }

    /// Authenticate user and return token
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<AuthToken, String> {
        // Find user
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        // Check if enabled
        if !user.enabled {
            return Err("User is disabled".to_string());
        }

        // Verify password
        if !Self::verify_password(password, &user.password_hash) {
            return Err("Invalid password".to_string());
        }

        // Generate token
        let token = self.generate_token(username)?;

        // Update last login
        if let Some(user) = self.find_user_mut(username) {
            user.last_login = Some(Utc::now());
            user.active_tokens.push(token.token.clone());
        }

        Ok(token)
    }

    /// Generate JWT token
    pub fn generate_token(&self, username: &str) -> Result<AuthToken, String> {
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        let role = user.roles.first().cloned().unwrap_or_else(|| "viewer".to_string());
        let expires_at = Utc::now() + Duration::hours(self.token_expiry_hours as i64);

        // Simple token format: username:timestamp:role:hash
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

    /// Validate token
    pub fn validate_token(&self, token: &str) -> Result<(String, String), String> {
        // Parse token: username:timestamp:hash
        let parts: Vec<&str> = token.split(':').collect();
        if parts.len() != 3 {
            return Err("Invalid token format".to_string());
        }

        let username = parts[0];
        let _timestamp = parts[1]
            .parse::<i64>()
            .map_err(|_| "Invalid timestamp".to_string())?;

        // Find user
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        // Check if user is enabled
        if !user.enabled {
            return Err("User is disabled".to_string());
        }

        // Check if token is in active tokens
        if !user.active_tokens.contains(&token.to_string()) {
            return Err("Token not active".to_string());
        }

        let role = user.roles.first().cloned().unwrap_or_else(|| "viewer".to_string());

        Ok((username.to_string(), role))
    }

    /// Create new user
    pub fn create_user(&mut self, req: CreateUserRequest) -> Result<String, String> {
        // Check if user exists
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

    /// Update user password
    pub fn change_password(&mut self, username: &str, old_password: &str, new_password: &str) -> Result<String, String> {
        let user = self
            .find_user(username)
            .ok_or_else(|| "User not found".to_string())?;

        if !Self::verify_password(old_password, &user.password_hash) {
            return Err("Old password incorrect".to_string());
        }

        if let Some(user) = self.find_user_mut(username) {
            user.password_hash = Self::hash_password(new_password);
            user.active_tokens.clear(); // Invalidate all tokens
        }

        Ok("Password changed successfully".to_string())
    }

    /// Revoke token
    pub fn revoke_token(&mut self, token: &str) -> Result<String, String> {
        for user in &mut self.users {
            if let Some(pos) = user.active_tokens.iter().position(|t| t == token) {
                user.active_tokens.remove(pos);
                return Ok("Token revoked successfully".to_string());
            }
        }
        Err("Token not found".to_string())
    }

    /// Disable user
    pub fn disable_user(&mut self, username: &str) -> Result<String, String> {
        if let Some(user) = self.find_user_mut(username) {
            user.enabled = false;
            user.active_tokens.clear(); // Invalidate all tokens
            Ok(format!("User '{}' disabled", username))
        } else {
            Err("User not found".to_string())
        }
    }

    /// Enable user
    pub fn enable_user(&mut self, username: &str) -> Result<String, String> {
        if let Some(user) = self.find_user_mut(username) {
            user.enabled = true;
            Ok(format!("User '{}' enabled", username))
        } else {
            Err("User not found".to_string())
        }
    }

    /// List all users (without sensitive data)
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

    /// Get user info
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let password = "test123";
        let hash = Authorization::hash_password(password);
        assert!(Authorization::verify_password(password, &hash));
        assert!(!Authorization::verify_password("wrong", &hash));
    }

    #[test]
    fn test_user_authentication() {
        let users = vec![User {
            username: "admin".to_string(),
            email: "admin@example.com".to_string(),
            password_hash: Authorization::hash_password("password123"),
            roles: vec!["admin".to_string()],
            enabled: true,
            created_at: Utc::now(),
            last_login: None,
            active_tokens: Vec::new(),
        }];

        let mut auth = Authorization::new(users, 24, "secret".to_string());

        let token = auth.authenticate("admin", "password123").unwrap();
        assert!(token.token.len() > 0);

        let (user, role) = auth.validate_token(&token.token).unwrap();
        assert_eq!(user, "admin");
        assert_eq!(role, "admin");
    }
}
