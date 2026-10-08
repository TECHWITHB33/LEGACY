use crate::error::ApiError;
use crate::models::User;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct StudentService {
    pool: Arc<SqlitePool>,
}

impl StudentService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        StudentService { pool }
    }

    /// Get student by ID
    pub async fn get_by_id(&self, student_id: &str) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = ?"
        )
        .bind(student_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(user)
    }

    /// Get student by email
    pub async fn get_by_email(&self, email: &str) -> Result<Option<User>, ApiError> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = ?"
        )
        .bind(email)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(user)
    }

    /// List all students
    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<User>, ApiError> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(users)
    }

    /// Suspend student account
    pub async fn suspend(&self, student_id: &str) -> Result<(), ApiError> {
        sqlx::query!("UPDATE users SET status = 'suspended' WHERE id = ?")
            .bind(student_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    /// Reactivate student account
    pub async fn reactivate(&self, student_id: &str) -> Result<(), ApiError> {
        sqlx::query!("UPDATE users SET status = 'active' WHERE id = ?")
            .bind(student_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }
}
