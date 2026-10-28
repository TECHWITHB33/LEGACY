use crate::error::ApiError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsaKeyPair {
    pub public_key: String,
    pub private_key: String,
}

pub fn generate_keypair() -> Result<RsaKeyPair, ApiError> {
    Ok(RsaKeyPair {
        public_key: "public-key".to_string(),
        private_key: "private-key".to_string(),
    })
}
