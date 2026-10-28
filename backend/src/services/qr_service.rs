use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct QrService;

impl QrService {
    pub async fn generate_qr(
        _pool: Arc<SqlitePool>,
        _credential_id: &str,
    ) -> Result<String, ApiError> {
        Ok("qr_code".to_string())
    }
}
