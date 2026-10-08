use sha2::{Sha256, Digest};
use crate::error::ApiError;
use crate::validators;

/// Calculate SHA256 hash and return as hex string
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Calculate SHA256 hash of string and return as hex string
pub fn sha256_hex_str(data: &str) -> String {
    sha256_hex(data.as_bytes())
}

/// Calculate SHA256 hash and return as bytes
pub fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Calculate SHA256 hash of string and return as bytes
pub fn sha256_bytes_str(data: &str) -> Vec<u8> {
    sha256_bytes(data.as_bytes())
}

/// Verify that data matches the provided hash
pub fn verify_sha256(data: &[u8], hash_hex: &str) -> bool {
    sha256_hex(data) == hash_hex
}

/// Verify that string data matches the provided hash
pub fn verify_sha256_str(data: &str, hash_hex: &str) -> bool {
    verify_sha256(data.as_bytes(), hash_hex)
}

// ============================================================================
// CREDENTIAL-SPECIFIC HASHING FUNCTIONS
// ============================================================================

/// Hash a canonical credential JSON representation
/// Returns SHA256 hash as hex string
pub fn hash_credential(canonical_json: &str) -> Result<String, ApiError> {
    if canonical_json.is_empty() {
        return Err(ApiError::CryptoError("Cannot hash empty credential".to_string()));
    }
    
    Ok(sha256_hex_str(canonical_json))
}

/// Validate and return schema_hash ensuring it's exactly 32 bytes (64 hex chars)
/// Delegates to validators module
pub fn validate_schema_hash(schema_hash: &str) -> Result<String, ApiError> {
    validators::validate_schema_hash(schema_hash)
}

/// Verify a credential hash matches its canonical JSON representation
pub fn verify_credential_hash(canonical_json: &str, expected_hash: &str) -> Result<bool, ApiError> {
    let computed_hash = hash_credential(canonical_json)?;
    Ok(computed_hash == expected_hash)
}

/// Hash QR code data
pub fn hash_qr_code(qr_data: &str) -> Result<String, ApiError> {
    if qr_data.is_empty() {
        return Err(ApiError::CryptoError("Cannot hash empty QR code".to_string()));
    }
    
    Ok(sha256_hex_str(qr_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let data = b"test data";
        let hash = sha256_hex(data);
        // Known SHA256 hash of "test data"
        assert_eq!(hash, "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9");
    }

    #[test]
    fn test_sha256_verify() {
        let data = "test message";
        let hash = sha256_hex_str(data);
        assert!(verify_sha256_str(data, &hash));
        assert!(!verify_sha256_str("different data", &hash));
    }

    #[test]
    fn test_hash_credential() {
        let json = r#"{"test":"credential"}"#;
        let hash = hash_credential(json).expect("Failed to hash credential");
        assert_eq!(hash.len(), 64, "SHA256 hex should be 64 characters");
        
        // Hash should be deterministic
        let hash2 = hash_credential(json).expect("Failed to hash credential");
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_credential_empty_fails() {
        let result = hash_credential("");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schema_hash_valid() {
        // Valid 64-character hex string (32 bytes)
        let valid_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";
        let result = validate_schema_hash(valid_hash);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), valid_hash);
    }

    #[test]
    fn test_validate_schema_hash_too_short() {
        let short_hash = "abc123de";
        let result = validate_schema_hash(short_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schema_hash_too_long() {
        let long_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123deabc";
        let result = validate_schema_hash(long_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_schema_hash_invalid_hex() {
        let invalid_hex = "ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ";
        let result = validate_schema_hash(invalid_hex);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_credential_hash() {
        let json = r#"{"id":"test","version":1}"#;
        let hash = hash_credential(json).expect("Failed to hash");
        
        let verified = verify_credential_hash(json, &hash).expect("Failed to verify");
        assert!(verified);
        
        let different_json = r#"{"id":"test","version":2}"#;
        let verified_different = verify_credential_hash(different_json, &hash).expect("Failed to verify");
        assert!(!verified_different);
    }

    #[test]
    fn test_hash_qr_code() {
        let qr_data = "https://example.com/verify?id=abc123";
        let hash = hash_qr_code(qr_data).expect("Failed to hash QR code");
        assert_eq!(hash.len(), 64, "SHA256 hex should be 64 characters");
    }

    #[test]
    fn test_hash_qr_code_empty_fails() {
        let result = hash_qr_code("");
        assert!(result.is_err());
    }
}
