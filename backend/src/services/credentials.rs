use crate::error::ApiError;
use crate::models::Credential;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct CredentialService {
    pool: Arc<SqlitePool>,
}

impl CredentialService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        CredentialService { pool }
    }

    /// Create a new credential
    pub async fn create(
        &self,
        credential_id: &str,
        user_id: &str,
        qr_code: &str,
        qr_hash: &str,
        credential_hash: Option<&str>,
        signature: Option<&str>,
    ) -> Result<Credential, ApiError> {
        sqlx::query!(
            r#"INSERT INTO credentials 
               (id, user_id, public_key_hash, qr_code, qr_hash, status, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            credential_id,
            user_id,
            credential_hash.unwrap_or(""),
            qr_code,
            qr_hash,
        )
        .execute(self.pool.as_ref())
        .await?;

        let credential = self.get_by_id(credential_id)
            .await?
            .ok_or(ApiError::InternalError)?;

        Ok(credential)
    }

    /// Get credential by ID
    pub async fn get_by_id(&self, credential_id: &str) -> Result<Option<Credential>, ApiError> {
        let credential = sqlx::query_as::<_, Credential>(
            "SELECT * FROM credentials WHERE id = ?"
        )
        .bind(credential_id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        Ok(credential)
    }

    /// List credentials for a user
    pub async fn list_by_user(&self, user_id: &str) -> Result<Vec<Credential>, ApiError> {
        let credentials = sqlx::query_as::<_, Credential>(
            "SELECT * FROM credentials WHERE user_id = ? ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(credentials)
    }

    /// Revoke a credential
    pub async fn revoke(&self, credential_id: &str) -> Result<(), ApiError> {
        sqlx::query!("UPDATE credentials SET status = 'revoked' WHERE id = ?")
            .bind(credential_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    /// Expire a credential
    pub async fn expire(&self, credential_id: &str) -> Result<(), ApiError> {
        sqlx::query!("UPDATE credentials SET status = 'expired' WHERE id = ?")
            .bind(credential_id)
            .execute(self.pool.as_ref())
            .await?;

        Ok(())
    }

    /// Verify credential is valid (not revoked, not expired)
    pub async fn is_valid(&self, credential_id: &str) -> Result<bool, ApiError> {
        let credential = self.get_by_id(credential_id)
            .await?
            .ok_or(ApiError::CredentialNotFound)?;

        Ok(credential.status == "active")
    }
}
