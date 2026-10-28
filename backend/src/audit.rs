use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

/// Audit event types
#[derive(Debug, Clone, Copy)]
pub enum AuditEvent {
    StudentCreated,
    StudentSuspended,
    CredentialIssued,
    CredentialVerified,
    CredentialRevoked,
    CredentialExpired,
    LoginSuccess,
    LoginFailure,
    BiometricAuthSuccess,
    BiometricAuthFailure,
    PaymentRequested,
    PaymentAuthorized,
    PaymentRejected,
    PaymentSubmitted,
    PaymentConfirmed,
    InvoiceCreated,
    InvoiceStateChanged,
    InvoiceRefunded,
    SettlementFeeRecorded,
    GlobalAdvancementExecuted,
}

impl AuditEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditEvent::StudentCreated => "STUDENT_CREATED",
            AuditEvent::StudentSuspended => "STUDENT_SUSPENDED",
            AuditEvent::CredentialIssued => "CREDENTIAL_ISSUED",
            AuditEvent::CredentialVerified => "CREDENTIAL_VERIFIED",
            AuditEvent::CredentialRevoked => "CREDENTIAL_REVOKED",
            AuditEvent::CredentialExpired => "CREDENTIAL_EXPIRED",
            AuditEvent::LoginSuccess => "LOGIN_SUCCESS",
            AuditEvent::LoginFailure => "LOGIN_FAILURE",
            AuditEvent::BiometricAuthSuccess => "BIOMETRIC_AUTH_SUCCESS",
            AuditEvent::BiometricAuthFailure => "BIOMETRIC_AUTH_FAILURE",
            AuditEvent::PaymentRequested => "PAYMENT_REQUESTED",
            AuditEvent::PaymentAuthorized => "PAYMENT_AUTHORIZED",
            AuditEvent::PaymentRejected => "PAYMENT_REJECTED",
            AuditEvent::PaymentSubmitted => "PAYMENT_SUBMITTED",
            AuditEvent::PaymentConfirmed => "PAYMENT_CONFIRMED",
            AuditEvent::InvoiceCreated => "INVOICE_CREATED",
            AuditEvent::InvoiceStateChanged => "INVOICE_STATE_CHANGED",
            AuditEvent::InvoiceRefunded => "INVOICE_REFUNDED",
            AuditEvent::SettlementFeeRecorded => "SETTLEMENT_FEE_RECORDED",
            AuditEvent::GlobalAdvancementExecuted => "GLOBAL_ADVANCEMENT_EXECUTED",
        }
    }
}

/// Log an audit event
pub async fn log_event(
    pool: &Arc<SqlitePool>,
    user_id: &str,
    event: AuditEvent,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
) -> Result<(), ApiError> {
    let audit_id = format!("AUD-{}", Uuid::new_v4());
    
    sqlx::query!(
        r#"INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp)
           VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
        audit_id,
        user_id,
        event.as_str(),
        resource_type,
        resource_id,
        details,
    )
    .execute(pool.as_ref())
    .await?;

    tracing::info!(
        "Audit logged: user={}, event={}, resource={}/{}",
        user_id,
        event.as_str(),
        resource_type,
        resource_id.unwrap_or("N/A")
    );

    Ok(())
}

/// Log an audit event without user context (e.g., for login failures)
pub async fn log_event_no_user(
    pool: &Arc<SqlitePool>,
    event: AuditEvent,
    resource_type: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
) -> Result<(), ApiError> {
    let audit_id = format!("AUD-{}", Uuid::new_v4());
    
    sqlx::query!(
        r#"INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp)
           VALUES (?, NULL, ?, ?, ?, ?, CURRENT_TIMESTAMP)"#,
        audit_id,
        event.as_str(),
        resource_type,
        resource_id,
        details,
    )
    .execute(pool.as_ref())
    .await?;

    tracing::info!(
        "Audit logged: event={}, resource={}/{}",
        event.as_str(),
        resource_type,
        resource_id.unwrap_or("N/A")
    );

    Ok(())
}

/// Helper macro for logging audit events with less boilerplate
#[macro_export]
macro_rules! audit_log {
    ($pool:expr, $user_id:expr, $event:expr, $resource_type:expr, $resource_id:expr, $details:expr) => {
        $crate::audit::log_event(
            $pool,
            $user_id,
            $event,
            $resource_type,
            $resource_id,
            $details,
        )
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to log audit event: {}", e);
        })
    };
}

/// Helper macro for logging events without user context
#[macro_export]
macro_rules! audit_log_no_user {
    ($pool:expr, $event:expr, $resource_type:expr, $resource_id:expr, $details:expr) => {
        $crate::audit::log_event_no_user(
            $pool,
            $event,
            $resource_type,
            $resource_id,
            $details,
        )
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to log audit event: {}", e);
        })
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_as_str() {
        assert_eq!(AuditEvent::StudentCreated.as_str(), "STUDENT_CREATED");
        assert_eq!(AuditEvent::LoginSuccess.as_str(), "LOGIN_SUCCESS");
        assert_eq!(
            AuditEvent::BiometricAuthSuccess.as_str(),
            "BIOMETRIC_AUTH_SUCCESS"
        );
    }
}
