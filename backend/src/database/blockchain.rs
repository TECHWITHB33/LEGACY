use crate::models::Block;
use sqlx::SqlitePool;

pub async fn get_block_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Block>, sqlx::Error> {
    sqlx::query_as::<_, Block>("SELECT id, transaction_hash, created_at FROM blocks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
