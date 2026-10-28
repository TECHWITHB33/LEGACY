use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct PaymentsService;

impl PaymentsService {
    pub async fn create_payment(
        _pool: Arc<SqlitePool>,
        _user_id: &str,
        _amount: &str,
    ) -> Result<String, ApiError> {
        Ok("payment_id".to_string())
    }
}
