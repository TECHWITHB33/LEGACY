//! JWT token handling for authentication

use crate::error::ApiError;
use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
}

pub fn create_token(user_id: &str, config: &Config) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::InternalError)?
        .as_secs();

    let claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now,
        exp: now + 3600, // 1 hour
    };

    // Placeholder: in production, use jsonwebtoken crate
    Ok(format!("token-{}", user_id))
}

pub fn verify_token(_token: &str, _config: &Config) -> Result<JwtClaims, ApiError> {
    // Placeholder: in production, use jsonwebtoken crate
    Err(ApiError::AuthenticationError("Invalid token".to_string()))
}
