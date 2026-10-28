use crate::models::Payment;
use sqlx::SqlitePool;

pub async fn get_payment_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as::<_, Payment>("SELECT id, user_id, amount, status, created_at FROM payments WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
