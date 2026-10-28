use crate::error::ApiError;
use crate::models::{RegisterRequest, LoginRequest};
use crate::jwt::JwtClaims;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::config::Config;

pub struct AuthenticationService;

impl AuthenticationService {
    pub async fn register(
        _pool: Arc<SqlitePool>,
        _req: &RegisterRequest,
    ) -> Result<String, ApiError> {
        // Placeholder
        Ok("registered".to_string())
    }

    pub async fn login(
        _pool: Arc<SqlitePool>,
        _req: &LoginRequest,
        _config: &Config,
    ) -> Result<String, ApiError> {
        // Placeholder
        Ok("token".to_string())
    }
}
