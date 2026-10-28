//! Audit logging module for compliance and security

use crate::error::ApiError;
use crate::models::AuditLog;
use sqlx::SqlitePool;
use chrono::Utc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub enum AuditEvent {
    InvoiceCreated,
    InvoiceStateChanged,
    PaymentSubmitted,
    PaymentAuthorized,
    SettlementRecorded,
}

impl AuditEvent {
    pub fn as_str(&self) -> &str {
        match self {
            AuditEvent::InvoiceCreated => "InvoiceCreated",
            AuditEvent::InvoiceStateChanged => "InvoiceStateChanged",
            AuditEvent::PaymentSubmitted => "PaymentSubmitted",
            AuditEvent::PaymentAuthorized => "PaymentAuthorized",
            AuditEvent::SettlementRecorded => "SettlementRecorded",
        }
    }
}

/// Log an audit event
pub async fn log_event(
    pool: &SqlitePool,
    user_id: &str,
    event: AuditEvent,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
) -> Result<(), ApiError> {
    let id = Uuid::new_v4().to_string();
    
    tracing::info!(
        "Audit event: user_id={}, event={}, resource_type={}, resource_id={:?}",
        user_id, event.as_str(), resource_type, resource_id
    );

    sqlx::query!(
        r#"INSERT INTO audit_logs (id, user_id, event_type, resource_type, resource_id, details, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?)"#,
        id,
        user_id,
        event.as_str(),
        resource_type,
        resource_id,
        details,
        Utc::now(),
    )
    .execute(pool)
    .await?;

    Ok(())
}
