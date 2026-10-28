use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct VerificationService;

impl VerificationService {
    pub async fn verify(
        _pool: Arc<SqlitePool>,
        _credential_id: &str,
    ) -> Result<bool, ApiError> {
        Ok(true)
    }
}
