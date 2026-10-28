use crate::error::ApiError;
use crate::models::Credential;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct CredentialIssuerService;

impl CredentialIssuerService {
    pub async fn issue_credential(
        _pool: Arc<SqlitePool>,
        _user_id: &str,
        _credential_type: &str,
    ) -> Result<Credential, ApiError> {
        Err(ApiError::CredentialNotFound)
    }
}
