use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct PaymentService {
    pool: Arc<SqlitePool>,
}

impl PaymentService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        PaymentService { pool }
    }

    /// Create a payment request
    pub async fn create_payment_request(
        &self,
        payment_id: &str,
        user_id: &str,
        amount: f64,
        asset_code: &str,
        merchant_id: &str,
    ) -> Result<(), ApiError> {
        // Validate amount
        if amount <= 0.0 {
            return Err(ApiError::PaymentError("Amount must be positive".to_string()));
        }

        sqlx::query!(
            r#"INSERT INTO payments 
               (id, user_id, amount, asset_code, recipient_address, merchant_id, status, created_at)
               VALUES (?, ?, ?, ?, ?, ?, 'pending', CURRENT_TIMESTAMP)"#,
            payment_id,
            user_id,
            amount.to_string(),
            asset_code,
            merchant_id,
            merchant_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Get payment status
    pub async fn get_payment_status(&self, payment_id: &str) -> Result<Option<String>, ApiError> {
        let payment = sqlx::query!("SELECT status FROM payments WHERE id = ?")
            .bind(payment_id)
            .fetch_optional(self.pool.as_ref())
            .await?;

        Ok(payment.map(|p| p.status))
    }

    /// Update payment status
    pub async fn update_payment_status(
        &self,
        payment_id: &str,
        status: &str,
    ) -> Result<(), ApiError> {
        sqlx::query!("UPDATE payments SET status = ? WHERE id = ?")
            .bind(status)
            .bind(payment_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    /// List payments for user
    pub async fn list_user_payments(&self, user_id: &str) -> Result<Vec<PaymentInfo>, ApiError> {
        let payments = sqlx::query_as::<_, PaymentInfo>(
            "SELECT id, amount, asset_code, status, created_at FROM payments WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(payments)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentInfo {
    pub id: String,
    pub amount: String,
    pub asset_code: String,
    pub status: String,
    pub created_at: Option<String>,
}
