use crate::error::ApiError;
use sha2::{Sha256, Digest};

pub fn hash_data(data: &[u8]) -> Result<String, ApiError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn hash_string(s: &str) -> Result<String, ApiError> {
    hash_data(s.as_bytes())
}
