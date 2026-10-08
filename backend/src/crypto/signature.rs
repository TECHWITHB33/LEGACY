use crate::error::ApiError;
use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Digital signature pair (public key + signature)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    pub signature_hex: String,
    pub public_key_hex: String,
}

/// Generate EdDSA keypair for digital signatures
pub fn generate_signature_keypair() -> Result<(String, String), ApiError> {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let signing_key_hex = hex::encode(signing_key.to_bytes());
    let verifying_key_hex = hex::encode(verifying_key.to_bytes());

    Ok((signing_key_hex, verifying_key_hex))
}

/// Sign data with private key
pub fn sign_data(message: &[u8], private_key_hex: &str) -> Result<String, ApiError> {
    let private_key_bytes = hex::decode(private_key_hex)
        .map_err(|e| ApiError::CryptoError(format!("Invalid hex key: {}", e)))?;

    if private_key_bytes.len() != 32 {
        return Err(ApiError::CryptoError(
            "Private key must be 32 bytes".to_string(),
        ));
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&private_key_bytes);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let signature = signing_key.sign(message);

    Ok(hex::encode(signature.to_bytes()))
}

/// Verify signature with public key
pub fn verify_signature(
    message: &[u8],
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, ApiError> {
    let signature_bytes = hex::decode(signature_hex)
        .map_err(|e| ApiError::CryptoError(format!("Invalid signature hex: {}", e)))?;

    let public_key_bytes = hex::decode(public_key_hex)
        .map_err(|e| ApiError::CryptoError(format!("Invalid public key hex: {}", e)))?;

    if signature_bytes.len() != 64 {
        return Err(ApiError::CryptoError(
            "Signature must be 64 bytes".to_string(),
        ));
    }

    if public_key_bytes.len() != 32 {
        return Err(ApiError::CryptoError(
            "Public key must be 32 bytes".to_string(),
        ));
    }

    let mut sig_bytes = [0u8; 64];
    sig_bytes.copy_from_slice(&signature_bytes);
    let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);

    let mut pub_key_bytes = [0u8; 32];
    pub_key_bytes.copy_from_slice(&public_key_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pub_key_bytes)
        .map_err(|e| ApiError::CryptoError(format!("Invalid public key: {}", e)))?;

    match verifying_key.verify(message, &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Sign a string message
pub fn sign_message(message: &str, private_key_hex: &str) -> Result<String, ApiError> {
    sign_data(message.as_bytes(), private_key_hex)
}

/// Verify a string message signature
pub fn verify_message(
    message: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, ApiError> {
    verify_signature(message.as_bytes(), signature_hex, public_key_hex)
}

// ============================================================================
// CREDENTIAL-SPECIFIC SIGNING FUNCTIONS
// ============================================================================

/// Sign credential hash with issuer's private key
/// 
/// # Arguments
/// * `credential_hash` - SHA-256 hash of credential (canonical JSON representation)
/// * `private_key_hex` - Issuer's private key in hex format
/// 
/// # Returns
/// * `Ok(String)` - EdDSA signature in hex format (128 characters)
/// * `Err(ApiError)` - If signing fails
pub fn sign_credential(credential_hash: &str, private_key_hex: &str) -> Result<String, ApiError> {
    if credential_hash.is_empty() {
        return Err(ApiError::CryptoError(
            "Credential hash cannot be empty".to_string(),
        ));
    }

    if credential_hash.len() != 64 {
        return Err(ApiError::CryptoError(
            "Credential hash must be 64 hex characters (32 bytes)".to_string(),
        ));
    }

    let credential_bytes = hex::decode(credential_hash)
        .map_err(|e| ApiError::CryptoError(format!("Invalid credential hash: {}", e)))?;

    sign_data(&credential_bytes, private_key_hex)
}

/// Sign multiple credential fields together (canonical JSON bytes)
pub fn sign_credential_bytes(
    canonical_json_bytes: &[u8],
    private_key_hex: &str,
) -> Result<String, ApiError> {
    if canonical_json_bytes.is_empty() {
        return Err(ApiError::CryptoError(
            "Credential data cannot be empty".to_string(),
        ));
    }

    sign_data(canonical_json_bytes, private_key_hex)
}

/// Create a credential signature with metadata
/// Returns both the signature and the signing public key
pub fn create_credential_signature(
    credential_hash: &str,
    private_key_hex: &str,
) -> Result<DigitalSignature, ApiError> {
    // Verify private key format (32 bytes = 64 hex chars)
    if private_key_hex.len() != 64 {
        return Err(ApiError::CryptoError(
            "Private key must be 64 hex characters (32 bytes)".to_string(),
        ));
    }

    let private_key_bytes = hex::decode(private_key_hex)
        .map_err(|e| ApiError::CryptoError(format!("Invalid private key hex: {}", e)))?;

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&private_key_bytes);

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    // Sign the credential hash
    let signature = sign_credential(credential_hash, private_key_hex)?;

    Ok(DigitalSignature {
        signature_hex: signature,
        public_key_hex: hex::encode(verifying_key.to_bytes()),
    })
}

// ============================================================================
// CREDENTIAL VERIFICATION FUNCTIONS
// ============================================================================

/// Verify a credential's signature against its hash and public key
/// 
/// # Arguments
/// * `credential_hash` - SHA-256 hash of credential (64 hex chars)
/// * `signature_hex` - EdDSA signature (128 hex chars)
/// * `public_key_hex` - Issuer's public key (64 hex chars)
/// 
/// # Returns
/// * `Ok(true)` - Signature is valid
/// * `Ok(false)` - Signature is invalid
/// * `Err(ApiError)` - If verification fails due to format errors
pub fn verify_credential_signature(
    credential_hash: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, ApiError> {
    if credential_hash.is_empty() {
        return Err(ApiError::CryptoError(
            "Credential hash cannot be empty".to_string(),
        ));
    }

    if credential_hash.len() != 64 {
        return Err(ApiError::CryptoError(
            "Credential hash must be 64 hex characters (32 bytes)".to_string(),
        ));
    }

    let credential_bytes = hex::decode(credential_hash)
        .map_err(|e| ApiError::CryptoError(format!("Invalid credential hash: {}", e)))?;

    verify_signature(&credential_bytes, signature_hex, public_key_hex)
}

/// Verify a credential signature from canonical JSON representation
/// 
/// # Arguments
/// * `canonical_json` - Canonical JSON string of the credential
/// * `signature_hex` - EdDSA signature (128 hex chars)
/// * `public_key_hex` - Issuer's public key (64 hex chars)
/// 
/// # Returns
/// * `Ok(true)` - Signature is valid
/// * `Ok(false)` - Signature is invalid
/// * `Err(ApiError)` - If verification fails
pub fn verify_credential_signature_json(
    canonical_json: &str,
    signature_hex: &str,
    public_key_hex: &str,
) -> Result<bool, ApiError> {
    if canonical_json.is_empty() {
        return Err(ApiError::CryptoError(
            "Canonical JSON cannot be empty".to_string(),
        ));
    }

    verify_signature(canonical_json.as_bytes(), signature_hex, public_key_hex)
}

/// Verify credential authenticity using credential hash and public key
/// This is the primary verification method for credentials
pub fn verify_credential_authenticity(
    credential_hash: &str,
    stored_signature: &str,
    issuer_public_key: &str,
) -> Result<bool, ApiError> {
    // Use credential_hash verification instead of canonical JSON
    verify_credential_signature(credential_hash, stored_signature, issuer_public_key)
}

/// Batch verify multiple signatures with the same public key
pub fn verify_multiple_credentials(
    credentials: Vec<(&str, &str)>, // (credential_hash, signature)
    public_key_hex: &str,
) -> Result<Vec<bool>, ApiError> {
    let mut results = Vec::new();

    for (credential_hash, signature) in credentials {
        let verified = verify_credential_signature(credential_hash, signature, public_key_hex)?;
        results.push(verified);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let result = generate_signature_keypair();
        assert!(result.is_ok());
        let (private_key, public_key) = result.unwrap();
        assert_eq!(private_key.len(), 64); // 32 bytes in hex = 64 chars
        assert_eq!(public_key.len(), 64);
    }

    #[test]
    fn test_sign_verify() {
        let (private_key, public_key) = generate_signature_keypair().unwrap();
        let message = b"Test message";

        let signature = sign_data(message, &private_key).unwrap();
        let verified = verify_signature(message, &signature, &public_key).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_signature_verification_fails_with_wrong_message() {
        let (private_key, public_key) = generate_signature_keypair().unwrap();
        let message = b"Test message";
        let wrong_message = b"Wrong message";

        let signature = sign_data(message, &private_key).unwrap();
        let verified = verify_signature(wrong_message, &signature, &public_key).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_sign_credential_valid() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";

        let signature = sign_credential(credential_hash, &private_key);
        assert!(signature.is_ok());
        let sig = signature.unwrap();
        assert_eq!(sig.len(), 128); // 64 bytes in hex = 128 chars
    }

    #[test]
    fn test_sign_credential_empty_fails() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let result = sign_credential("", &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_credential_invalid_hash_length() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let short_hash = "abc123de";
        let result = sign_credential(short_hash, &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_credential_invalid_hex() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let invalid_hash = "Z".repeat(64);
        let result = sign_credential(&invalid_hash, &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_sign_credential_bytes() {
        let (private_key, public_key) = generate_signature_keypair().unwrap();
        let data = br#"{"id":"cred-123","user_id":"user-456","version":1}"#;

        let signature = sign_credential_bytes(data, &private_key);
        assert!(signature.is_ok());

        let sig = signature.unwrap();
        let verified = verify_signature(data, &sig, &public_key).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_create_credential_signature() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";

        let result = create_credential_signature(credential_hash, &private_key);
        assert!(result.is_ok());

        let sig = result.unwrap();
        assert_eq!(sig.signature_hex.len(), 128);
        assert_eq!(sig.public_key_hex.len(), 64);
    }

    #[test]
    fn test_create_credential_signature_invalid_private_key() {
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";
        let invalid_key = "short";

        let result = create_credential_signature(credential_hash, invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_credential_signature_round_trip() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";

        let sig = create_credential_signature(credential_hash, &private_key).unwrap();

        // Verify the signature with the public key from the same keypair
        let verified = verify_signature(
            hex::decode(credential_hash).unwrap().as_slice(),
            &sig.signature_hex,
            &sig.public_key_hex,
        )
        .unwrap();

        assert!(verified);
    }

    #[test]
    fn test_verify_credential_signature() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";

        let sig = create_credential_signature(credential_hash, &private_key).unwrap();
        let verified = verify_credential_signature(credential_hash, &sig.signature_hex, &sig.public_key_hex);

        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }

    #[test]
    fn test_verify_credential_signature_empty_hash() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let result = verify_credential_signature("", "", &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_credential_signature_invalid_hash_length() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let result = verify_credential_signature("short", "", &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_credential_signature_wrong_hash() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";
        let wrong_hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let sig = create_credential_signature(credential_hash, &private_key).unwrap();
        let verified = verify_credential_signature(wrong_hash, &sig.signature_hex, &sig.public_key_hex);

        assert!(verified.is_ok());
        assert!(!verified.unwrap()); // Should not verify with different hash
    }

    #[test]
    fn test_verify_credential_signature_json() {
        let (private_key, public_key) = generate_signature_keypair().unwrap();
        let canonical_json = r#"{"id":"cred-123","user_id":"user-456","version":1}"#;

        let sig = sign_credential_bytes(canonical_json.as_bytes(), &private_key).unwrap();
        let verified = verify_credential_signature_json(canonical_json, &sig, &public_key);

        assert!(verified.is_ok());
        assert!(verified.unwrap());

        // Test with wrong public key
        let (_, wrong_public_key) = generate_signature_keypair().unwrap();
        let verified_wrong_key = verify_credential_signature_json(canonical_json, &sig, &wrong_public_key);
        assert!(verified_wrong_key.is_ok());
        assert!(!verified_wrong_key.unwrap());
    }

    #[test]
    fn test_verify_credential_authenticity() {
        let (private_key, _) = generate_signature_keypair().unwrap();
        let credential_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";

        let sig = create_credential_signature(credential_hash, &private_key).unwrap();
        let verified = verify_credential_authenticity(credential_hash, &sig.signature_hex, &sig.public_key_hex);

        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }

    #[test]
    fn test_verify_multiple_credentials() {
        let (private_key, _) = generate_signature_keypair().unwrap();

        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de";
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

        let sig1 = create_credential_signature(hash1, &private_key).unwrap();
        let sig2 = create_credential_signature(hash2, &private_key).unwrap();

        let credentials = vec![
            (hash1, sig1.signature_hex.as_str()),
            (hash2, sig2.signature_hex.as_str()),
        ];

        let results = verify_multiple_credentials(credentials, &sig1.public_key_hex);
        assert!(results.is_ok());

        let verified = results.unwrap();
        assert_eq!(verified.len(), 2);
        assert!(verified[0]);
        assert!(verified[1]);
    }
}
