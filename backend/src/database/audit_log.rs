use crate::models::AuditLog;
use sqlx::SqlitePool;

pub async fn get_audit_log_by_id(pool: &SqlitePool, id: &str) -> Result<Option<AuditLog>, sqlx::Error> {
    sqlx::query_as::<_, AuditLog>(
        "SELECT id, user_id, event_type, resource_type, resource_id, details, created_at FROM audit_logs WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}
