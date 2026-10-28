use crate::error::ApiError;
use crate::models::{Credential, CredentialStatus};
use chrono::{DateTime, Duration, Utc};

pub struct ExpirationManager;

impl ExpirationManager {
    pub fn check_expiration(_credential: &Credential) -> Result<CredentialStatus, ApiError> {
        Ok(CredentialStatus::Active)
    }
}
