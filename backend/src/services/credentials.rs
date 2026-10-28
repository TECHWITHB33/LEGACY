use crate::error::ApiError;
use crate::models::Credential;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct CredentialsService;

impl CredentialsService {
    pub async fn get_credential(
        _pool: Arc<SqlitePool>,
        _id: &str,
    ) -> Result<Credential, ApiError> {
        Err(ApiError::CredentialNotFound)
    }
}
