use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AuditService {
    pool: Arc<SqlitePool>,
}

impl AuditService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        AuditService { pool }
    }

    /// Get audit logs for a user
    pub async fn get_user_logs(&self, user_id: &str, limit: i64) -> Result<Vec<AuditLogEntry>, ApiError> {
        let logs = sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, action, resource_type, resource_id, details, timestamp FROM audit_logs WHERE user_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(logs)
    }

    /// Get audit logs for a resource
    pub async fn get_resource_logs(
        &self,
        resource_type: &str,
        resource_id: &str,
        limit: i64,
    ) -> Result<Vec<AuditLogEntry>, ApiError> {
        let logs = sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, action, resource_type, resource_id, details, timestamp FROM audit_logs WHERE resource_type = ? AND resource_id = ? ORDER BY timestamp DESC LIMIT ?"
        )
        .bind(resource_type)
        .bind(resource_id)
        .bind(limit)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(logs)
    }

    /// Get all audit logs (for admin)
    pub async fn get_all_logs(&self, limit: i64, offset: i64) -> Result<Vec<AuditLogEntry>, ApiError> {
        let logs = sqlx::query_as::<_, AuditLogEntry>(
            "SELECT id, action, resource_type, resource_id, details, timestamp FROM audit_logs ORDER BY timestamp DESC LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(logs)
    }

    /// Get specific event count
    pub async fn count_events(&self, action: &str) -> Result<i64, ApiError> {
        let result = sqlx::query!("SELECT COUNT(*) as count FROM audit_logs WHERE action = ?")
            .bind(action)
            .fetch_one(self.pool.as_ref())
            .await?;

        Ok(result.count)
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: String,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub timestamp: Option<String>,
}
