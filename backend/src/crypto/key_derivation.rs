use crate::error::ApiError;
use sha2::{Sha256, Digest};

const SALT_SIZE: usize = 16;
const ITERATIONS: u32 = 100_000;
const KEY_SIZE: usize = 32;

/// PBKDF2-SHA256 key derivation parameters
#[derive(Debug, Clone)]
pub struct KeyDerivationParams {
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub key_size: usize,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            salt: generate_salt(),
            iterations: ITERATIONS,
            key_size: KEY_SIZE,
        }
    }
}

/// Generate a random salt
pub fn generate_salt() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..SALT_SIZE).map(|_| rng.gen()).collect()
}

/// Derive a key from a password using PBKDF2-SHA256
/// Returns (derived_key_hex, salt_hex)
pub fn derive_key(
    password: &str,
    salt: Option<Vec<u8>>,
) -> Result<(String, String), ApiError> {
    let salt = salt.unwrap_or_else(generate_salt);
    let salt_hex = hex::encode(&salt);

    let derived_key = pbkdf2_sha256(password.as_bytes(), &salt, ITERATIONS, KEY_SIZE)
        .map_err(|e| ApiError::CryptoError(format!("Key derivation failed: {}", e)))?;

    let derived_key_hex = hex::encode(&derived_key);

    Ok((derived_key_hex, salt_hex))
}

/// Derive key with custom parameters
pub fn derive_key_with_params(
    password: &str,
    params: &KeyDerivationParams,
) -> Result<String, ApiError> {
    pbkdf2_sha256(
        password.as_bytes(),
        &params.salt,
        params.iterations,
        params.key_size,
    )
    .map(|key| hex::encode(&key))
    .map_err(|e| ApiError::CryptoError(format!("Key derivation failed: {}", e)))
}

/// Verify a password against a derived key
pub fn verify_key(
    password: &str,
    derived_key_hex: &str,
    salt_hex: &str,
) -> Result<bool, ApiError> {
    let salt = hex::decode(salt_hex)
        .map_err(|e| ApiError::CryptoError(format!("Invalid salt hex: {}", e)))?;

    let (new_key_hex, _) = derive_key(password, Some(salt))?;

    // Constant-time comparison to prevent timing attacks
    Ok(constant_time_compare(
        derived_key_hex.as_bytes(),
        new_key_hex.as_bytes(),
    ))
}

/// PBKDF2-SHA256 implementation
fn pbkdf2_sha256(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_size: usize,
) -> Result<Vec<u8>, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    type HmacSha256 = Hmac<Sha256>;

    let mut result = vec![0u8; key_size];

    let num_blocks = (key_size + 31) / 32; // Ceiling division for SHA256 (32 bytes)

    for block_idx in 1..=num_blocks {
        let mut u = {
            let mut hmac = HmacSha256::new_from_slice(password)
                .map_err(|_| "Invalid HMAC key length".to_string())?;
            hmac.update(salt);
            hmac.update(&block_idx.to_be_bytes());
            hmac.finalize().into_bytes().to_vec()
        };

        let mut t = u.clone();

        for _ in 1..iterations {
            let mut hmac = HmacSha256::new_from_slice(password)
                .map_err(|_| "Invalid HMAC key length".to_string())?;
            hmac.update(&u);
            u = hmac.finalize().into_bytes().to_vec();

            for (i, byte) in u.iter().enumerate() {
                t[i] ^= byte;
            }
        }

        let start_idx = (block_idx as usize - 1) * 32;
        let end_idx = std::cmp::min(start_idx + 32, key_size);
        result[start_idx..end_idx].copy_from_slice(&t[..(end_idx - start_idx)]);
    }

    Ok(result)
}

/// Constant-time comparison to prevent timing attacks
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }

    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let password = "test_password";
        let (key, salt) = derive_key(password, None).unwrap();

        assert!(!key.is_empty());
        assert!(!salt.is_empty());
        assert_eq!(key.len(), KEY_SIZE * 2); // hex encoded
    }

    #[test]
    fn test_verify_key() {
        let password = "test_password";
        let (derived_key, salt) = derive_key(password, None).unwrap();

        assert!(verify_key(password, &derived_key, &salt).unwrap());
        assert!(!verify_key("wrong_password", &derived_key, &salt).unwrap());
    }

    #[test]
    fn test_constant_time_compare() {
        let a = b"hello";
        let b_same = b"hello";
        let b_diff = b"world";

        assert!(constant_time_compare(a, b_same));
        assert!(!constant_time_compare(a, b_diff));
    }
}
