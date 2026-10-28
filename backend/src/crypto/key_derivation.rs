use crate::error::ApiError;
use sha2::{Sha256, Digest};

pub fn derive_key(password: &str, salt: &str) -> Result<String, ApiError> {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
