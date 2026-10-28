use crate::error::ApiError;
use crate::crypto::key_derivation;

pub fn hash_password(password: &str) -> Result<(String, String), ApiError> {
    let salt = "salt".to_string();
    let hash = key_derivation::derive_key(password, &salt)?;
    Ok((hash, salt))
}

pub fn verify_password(password: &str, hash: &str, salt: &str) -> Result<bool, ApiError> {
    let derived = key_derivation::derive_key(password, salt)?;
    Ok(derived == hash)
}
