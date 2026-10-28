use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AuditService;

impl AuditService {
    pub async fn log(
        _pool: Arc<SqlitePool>,
        _user_id: &str,
        _event: &str,
    ) -> Result<(), ApiError> {
        Ok(())
    }
}
