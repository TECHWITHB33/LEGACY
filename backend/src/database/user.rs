use crate::models::User;
use sqlx::SqlitePool;

pub async fn get_user_by_id(pool: &SqlitePool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, password_salt, created_at FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_email(pool: &SqlitePool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT id, email, password_hash, password_salt, created_at FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(pool)
        .await
}
