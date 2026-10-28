use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub challenge_id: String,
    pub challenge: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricAuthRequest {
    pub challenge_id: String,
    pub response: String,
}

pub async fn initiate_auth(_user_id: &str) -> Result<AuthenticationChallenge, ApiError> {
    Ok(AuthenticationChallenge {
        challenge_id: "challenge-id".to_string(),
        challenge: "challenge-data".to_string(),
    })
}

pub async fn verify_auth(_request: &BiometricAuthRequest) -> Result<bool, ApiError> {
    Ok(true)
}
