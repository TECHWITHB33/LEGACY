use crate::error::ApiError;
use crate::models::CredentialStatus;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct RevocationManager;

impl RevocationManager {
    pub async fn revoke(_credential_id: &str) -> Result<(), ApiError> {
        Ok(())
    }
}
