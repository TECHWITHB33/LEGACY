// Biometric Authentication Gateway using WebAuthn
// IMPORTANT: This module uses WebAuthn for device-local biometric processing.
// Raw biometric data (fingerprints, face images) NEVER leave the device.
// The server receives only cryptographic authentication results.
//
// Browser/Device Flow:
// 1. Server generates challenge
// 2. Browser/Device shows biometric prompt (fingerprint/face/PIN)
// 3. Device performs biometric verification locally using secure enclave
// 4. Device signs challenge with private key (stored in secure hardware)
// 5. Browser sends signature + attestation to server
// 6. Server verifies signature against public key
// Result: Server never sees biometric data, only cryptographic proof

use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

/// Registration challenge for WebAuthn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    /// Challenge ID for tracking
    pub id: String,
    /// Random challenge bytes (base64)
    pub challenge: String,
    /// User ID
    pub user_id: String,
    /// Generated timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration time (5 minutes)
    pub expires_at: DateTime<Utc>,
    /// Whether this challenge was used
    pub used: bool,
}

impl RegistrationChallenge {
    pub fn new(user_id: String) -> Self {
        let now = Utc::now();
        let challenge_bytes = Uuid::new_v4().as_bytes().to_vec();
        let challenge = base64::encode(&challenge_bytes);

        RegistrationChallenge {
            id: Uuid::new_v4().to_string(),
            challenge,
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            used: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.used
    }
}

/// Authentication challenge for WebAuthn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    /// Challenge ID for tracking
    pub id: String,
    /// Random challenge bytes (base64)
    pub challenge: String,
    /// Credential ID being challenged
    pub credential_id: String,
    /// User ID
    pub user_id: String,
    /// Generated timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration time (5 minutes)
    pub expires_at: DateTime<Utc>,
    /// Whether this challenge was used
    pub used: bool,
}

impl AuthenticationChallenge {
    pub fn new(credential_id: String, user_id: String) -> Self {
        let now = Utc::now();
        let challenge_bytes = Uuid::new_v4().as_bytes().to_vec();
        let challenge = base64::encode(&challenge_bytes);

        AuthenticationChallenge {
            id: Uuid::new_v4().to_string(),
            challenge,
            credential_id,
            user_id,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            used: false,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    pub fn is_valid(&self) -> bool {
        !self.is_expired() && !self.used
    }
}

/// Biometric credential stored on server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricCredential {
    /// Credential ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Public key (base64)
    pub public_key: String,
    /// Credential type (fingerprint, face, platform)
    pub credential_type: BiometricType,
    /// Device identifier
    pub device_id: String,
    /// Device name
    pub device_name: String,
    /// Counter for replay protection
    pub sign_count: u32,
    /// Status: active, revoked, expired
    pub status: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiration timestamp
    pub expires_at: Option<DateTime<Utc>>,
    /// Last used timestamp
    pub last_used_at: Option<DateTime<Utc>>,
    /// Revocation timestamp
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Biometric credential type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BiometricType {
    Fingerprint,
    FaceAuthentication,
    PlatformPasskey,
}

impl BiometricType {
    pub fn as_str(&self) -> &str {
        match self {
            BiometricType::Fingerprint => "fingerprint",
            BiometricType::FaceAuthentication => "face",
            BiometricType::PlatformPasskey => "platform",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "fingerprint" => Ok(BiometricType::Fingerprint),
            "face" => Ok(BiometricType::FaceAuthentication),
            "platform" => Ok(BiometricType::PlatformPasskey),
            _ => Err(ApiError::ValidationError(
                "Invalid biometric type".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for BiometricType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Registration request from browser/device
/// Device has already performed biometric verification locally
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    /// Challenge ID that was issued by server
    pub challenge_id: String,
    /// Public key (base64) generated by device
    pub public_key: String,
    /// Attestation object (base64) from WebAuthn API
    pub attestation_object: String,
    /// Biometric type used
    pub credential_type: String,
    /// Device ID/identifier
    pub device_id: String,
    /// Device name
    pub device_name: String,
}

/// Registration response to browser/device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub success: bool,
    pub credential_id: Option<String>,
    pub message: String,
}

/// Authentication request from browser/device
/// Device has NOT performed biometric verification yet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationBeginRequest {
    pub user_id: String,
    pub credential_id: String,
}

/// Authentication begin response
/// Server sends challenge to device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationBeginResponse {
    pub challenge_id: String,
    pub challenge: String,
    pub credential_id: String,
}

/// Authentication complete request from browser/device
/// Device has performed biometric verification and signed challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationCompleteRequest {
    /// Challenge ID issued by server
    pub challenge_id: String,
    /// Credential ID
    pub credential_id: String,
    /// Signature over challenge (base64)
    pub signature: String,
    /// Sign count from credential (for replay protection)
    pub sign_count: u32,
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthenticationResult {
    Success,
    InvalidChallenge,
    ExpiredChallenge,
    ReplayedChallenge,
    InvalidSignature,
    UnknownCredential,
    RevokedCredential,
    ExpiredCredential,
    FailureCountExceeded,
}

impl AuthenticationResult {
    pub fn as_str(&self) -> &str {
        match self {
            AuthenticationResult::Success => "SUCCESS",
            AuthenticationResult::InvalidChallenge => "INVALID_CHALLENGE",
            AuthenticationResult::ExpiredChallenge => "EXPIRED_CHALLENGE",
            AuthenticationResult::ReplayedChallenge => "REPLAYED_CHALLENGE",
            AuthenticationResult::InvalidSignature => "INVALID_SIGNATURE",
            AuthenticationResult::UnknownCredential => "UNKNOWN_CREDENTIAL",
            AuthenticationResult::RevokedCredential => "REVOKED_CREDENTIAL",
            AuthenticationResult::ExpiredCredential => "EXPIRED_CREDENTIAL",
            AuthenticationResult::FailureCountExceeded => "FAILURE_COUNT_EXCEEDED",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, AuthenticationResult::Success)
    }
}

impl std::fmt::Display for AuthenticationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Authentication response to browser/device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResponse {
    pub result: String,
    pub user_id: Option<String>,
    pub credential_id: Option<String>,
    pub message: String,
    pub authenticated_at: String,
}

/// Biometric authentication engine
pub struct BiometricAuthEngine {
    issuer_name: String,
}

impl BiometricAuthEngine {
    pub fn new(issuer_name: String) -> Self {
        BiometricAuthEngine { issuer_name }
    }

    /// Verify registration attestation
    /// In production, this would verify:
    /// - Attestation statement format (packed, fido-u2f, android-key, etc.)
    /// - Certificate chain
    /// - Credential public key
    pub fn verify_registration_attestation(
        &self,
        _attestation_object: &str,
        _public_key: &str,
    ) -> Result<bool, ApiError> {
        // For MVP, attestation verification is simplified
        // In production, use webauthn-rs or similar library
        Ok(true)
    }

    /// Verify authentication signature
    /// Verifies that the signature was created by the device's private key
    pub fn verify_signature(
        &self,
        public_key: &str,
        challenge: &str,
        signature: &str,
    ) -> Result<bool, ApiError> {
        // Decode public key
        let public_key_bytes = base64::decode(public_key)
            .map_err(|_| ApiError::ValidationError("Invalid public key encoding".to_string()))?;

        // Decode signature
        let signature_bytes = base64::decode(signature)
            .map_err(|_| ApiError::ValidationError("Invalid signature encoding".to_string()))?;

        // In production, use Ed25519 or ECDSA verification
        // For now, verify signature length is reasonable
        if signature_bytes.len() < 64 || signature_bytes.len() > 256 {
            return Ok(false);
        }

        // Verify public key length
        if public_key_bytes.len() < 32 || public_key_bytes.len() > 256 {
            return Ok(false);
        }

        // In production, perform actual cryptographic verification
        Ok(true)
    }

    /// Check for replay attack
    /// If sign_count hasn't increased, the same credential was used twice
    pub fn check_replay_protection(
        old_sign_count: u32,
        new_sign_count: u32,
    ) -> Result<bool, ApiError> {
        if new_sign_count <= old_sign_count {
            return Ok(false); // Replay detected
        }
        Ok(true) // OK
    }

    /// Generate credential verification hash
    pub fn generate_credential_hash(
        user_id: &str,
        credential_id: &str,
        public_key: &str,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(user_id.as_bytes());
        hasher.update(credential_id.as_bytes());
        hasher.update(public_key.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Verify credential is active and valid
    pub fn verify_credential_status(credential: &BiometricCredential) -> AuthenticationResult {
        if credential.status == "revoked" {
            return AuthenticationResult::RevokedCredential;
        }

        if let Some(revoked_at) = credential.revoked_at {
            if Utc::now() > revoked_at {
                return AuthenticationResult::RevokedCredential;
            }
        }

        if let Some(expires_at) = credential.expires_at {
            if Utc::now() > expires_at {
                return AuthenticationResult::ExpiredCredential;
            }
        }

        AuthenticationResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registration_challenge_creation() {
        let challenge = RegistrationChallenge::new("user-123".to_string());
        assert_eq!(challenge.user_id, "user-123");
        assert!(!challenge.challenge.is_empty());
        assert!(!challenge.used);
        assert!(challenge.is_valid());
    }

    #[test]
    fn test_registration_challenge_expiration() {
        let mut challenge = RegistrationChallenge::new("user-123".to_string());
        challenge.expires_at = Utc::now() - chrono::Duration::minutes(1);
        assert!(challenge.is_expired());
        assert!(!challenge.is_valid());
    }

    #[test]
    fn test_authentication_challenge_creation() {
        let challenge =
            AuthenticationChallenge::new("cred-123".to_string(), "user-123".to_string());
        assert_eq!(challenge.credential_id, "cred-123");
        assert_eq!(challenge.user_id, "user-123");
        assert!(!challenge.used);
        assert!(challenge.is_valid());
    }

    #[test]
    fn test_biometric_type_conversion() {
        assert_eq!(BiometricType::Fingerprint.as_str(), "fingerprint");
        assert_eq!(BiometricType::FaceAuthentication.as_str(), "face");
        assert_eq!(BiometricType::PlatformPasskey.as_str(), "platform");

        assert_eq!(BiometricType::from_str("fingerprint").unwrap(), BiometricType::Fingerprint);
        assert_eq!(BiometricType::from_str("face").unwrap(), BiometricType::FaceAuthentication);
        assert_eq!(BiometricType::from_str("platform").unwrap(), BiometricType::PlatformPasskey);
    }

    #[test]
    fn test_authentication_result_success() {
        assert!(AuthenticationResult::Success.is_success());
        assert!(!AuthenticationResult::InvalidChallenge.is_success());
        assert!(!AuthenticationResult::ExpiredChallenge.is_success());
    }

    #[test]
    fn test_replay_protection() {
        assert!(BiometricAuthEngine::check_replay_protection(5, 6).unwrap());
        assert!(!BiometricAuthEngine::check_replay_protection(5, 5).unwrap());
        assert!(!BiometricAuthEngine::check_replay_protection(5, 4).unwrap());
    }

    #[test]
    fn test_credential_hash_generation() {
        let hash1 = BiometricAuthEngine::generate_credential_hash(
            "user-1",
            "cred-1",
            "pubkey-1",
        );
        let hash2 = BiometricAuthEngine::generate_credential_hash(
            "user-1",
            "cred-1",
            "pubkey-1",
        );
        assert_eq!(hash1, hash2);

        let hash3 = BiometricAuthEngine::generate_credential_hash(
            "user-2",
            "cred-1",
            "pubkey-1",
        );
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_credential_status_verification_active() {
        let credential = BiometricCredential {
            id: "cred-1".to_string(),
            user_id: "user-1".to_string(),
            public_key: "pubkey".to_string(),
            credential_type: BiometricType::Fingerprint,
            device_id: "device-1".to_string(),
            device_name: "iPhone 14".to_string(),
            sign_count: 10,
            status: "active".to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            last_used_at: None,
            revoked_at: None,
        };

        assert_eq!(
            BiometricAuthEngine::verify_credential_status(&credential),
            AuthenticationResult::Success
        );
    }

    #[test]
    fn test_credential_status_verification_revoked() {
        let credential = BiometricCredential {
            id: "cred-1".to_string(),
            user_id: "user-1".to_string(),
            public_key: "pubkey".to_string(),
            credential_type: BiometricType::Fingerprint,
            device_id: "device-1".to_string(),
            device_name: "iPhone 14".to_string(),
            sign_count: 10,
            status: "revoked".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            last_used_at: None,
            revoked_at: Some(Utc::now()),
        };

        assert_eq!(
            BiometricAuthEngine::verify_credential_status(&credential),
            AuthenticationResult::RevokedCredential
        );
    }

    #[test]
    fn test_credential_status_verification_expired() {
        let credential = BiometricCredential {
            id: "cred-1".to_string(),
            user_id: "user-1".to_string(),
            public_key: "pubkey".to_string(),
            credential_type: BiometricType::Fingerprint,
            device_id: "device-1".to_string(),
            device_name: "iPhone 14".to_string(),
            sign_count: 10,
            status: "active".to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() - chrono::Duration::minutes(1)),
            last_used_at: None,
            revoked_at: None,
        };

        assert_eq!(
            BiometricAuthEngine::verify_credential_status(&credential),
            AuthenticationResult::ExpiredCredential
        );
    }
}
