use crate::error::ApiError;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct VerificationService {
    pool: Arc<SqlitePool>,
}

impl VerificationService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        VerificationService { pool }
    }

    /// Verify a credential is valid, not revoked, and not expired
    pub async fn verify_credential(
        &self,
        credential_id: &str,
    ) -> Result<VerificationResult, ApiError> {
        // Look up credential
        let credential = sqlx::query!(
            "SELECT id, status, expires_at FROM credentials WHERE id = ?"
        )
        .bind(credential_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or(ApiError::CredentialNotFound)?;

        // Check if revoked
        if credential.status == "revoked" {
            return Ok(VerificationResult::Revoked);
        }

        // Check if expired
        if credential.status == "expired" {
            return Ok(VerificationResult::Expired);
        }

        // Check expiration date if set
        if let Some(expires_at) = credential.expires_at {
            if expires_at < chrono::Utc::now() {
                return Ok(VerificationResult::Expired);
            }
        }

        // Valid credential
        Ok(VerificationResult::Valid)
    }

    /// Verify credential signature
    pub async fn verify_signature(
        &self,
        credential_id: &str,
        expected_signature: &str,
    ) -> Result<bool, ApiError> {
        // TODO: Implement signature verification
        // For now, return true as placeholder
        Ok(true)
    }

    /// Verify QR hash
    pub async fn verify_qr_hash(
        &self,
        credential_id: &str,
        expected_qr_hash: &str,
    ) -> Result<bool, ApiError> {
        let credential = sqlx::query!(
            "SELECT qr_hash FROM credentials WHERE id = ?"
        )
        .bind(credential_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or(ApiError::CredentialNotFound)?;

        Ok(credential.qr_hash == expected_qr_hash)
    }
}

#[derive(Debug, Clone)]
pub enum VerificationResult {
    Valid,
    Invalid,
    Revoked,
    Expired,
    Tampered,
    NotFound,
}

impl VerificationResult {
    pub fn as_str(&self) -> &'static str {
        match self {
            VerificationResult::Valid => "VALID",
            VerificationResult::Invalid => "INVALID",
            VerificationResult::Revoked => "REVOKED",
            VerificationResult::Expired => "EXPIRED",
            VerificationResult::Tampered => "TAMPERED",
            VerificationResult::NotFound => "NOT_FOUND",
        }
    }
}
