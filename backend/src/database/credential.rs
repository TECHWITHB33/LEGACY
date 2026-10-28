use crate::models::Credential;
use sqlx::SqlitePool;

pub async fn get_credential_by_id(pool: &SqlitePool, id: &str) -> Result<Option<Credential>, sqlx::Error> {
    sqlx::query_as::<_, Credential>("SELECT id, user_id, credential_type, status, created_at, updated_at FROM credentials WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}
