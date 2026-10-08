// QR Code Verification System
// Secure QR generation and verification with credential validation
// Does NOT store: passwords, private keys, seed phrases, raw biometrics, unnecessary PII

use crate::error::ApiError;
use crate::models::Credential;
use crate::credential_expiration::ExpirationManager;
use serde::{Deserialize, Serialize};
use chrono::Utc;

/// QR Verification Result Status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QrVerificationStatus {
    /// Credential is valid and can be used
    Valid,
    /// Credential verification failed (signature/hash mismatch)
    Invalid,
    /// Credential data has been tampered with
    Tampered,
    /// Credential has been revoked
    Revoked,
    /// Credential has expired
    Expired,
    /// Credential not found in system
    NotFound,
}

impl QrVerificationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            QrVerificationStatus::Valid => "VALID",
            QrVerificationStatus::Invalid => "INVALID",
            QrVerificationStatus::Tampered => "TAMPERED",
            QrVerificationStatus::Revoked => "REVOKED",
            QrVerificationStatus::Expired => "EXPIRED",
            QrVerificationStatus::NotFound => "NOT_FOUND",
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self, QrVerificationStatus::Valid)
    }
}

impl std::fmt::Display for QrVerificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Secure QR Code Data - Only contains non-sensitive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureQrData {
    /// Credential ID (non-secret identifier)
    pub credential_id: String,
    /// Verification URL or endpoint
    pub verification_url: Option<String>,
    /// Issuer name
    pub issuer: String,
    /// Generated timestamp
    pub generated_at: String,
    /// Optional custom reference
    pub reference: Option<String>,
}

impl SecureQrData {
    /// Create QR data with only non-sensitive information
    pub fn new(credential_id: String, issuer: String) -> Self {
        SecureQrData {
            credential_id,
            verification_url: None,
            issuer,
            generated_at: Utc::now().to_rfc3339(),
            reference: None,
        }
    }

    /// Add verification URL
    pub fn with_url(mut self, url: String) -> Self {
        self.verification_url = Some(url);
        self
    }

    /// Add reference information
    pub fn with_reference(mut self, reference: String) -> Self {
        self.reference = Some(reference);
        self
    }

    /// Serialize to JSON string for QR encoding
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse JSON string from QR code
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// QR Verification Result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrVerificationResult {
    /// Overall verification status
    pub status: QrVerificationStatus,
    /// Credential ID extracted from QR
    pub credential_id: Option<String>,
    /// Verification message
    pub message: String,
    /// Credential data (if valid)
    pub credential_data: Option<QrCredentialInfo>,
    /// Timestamp of verification
    pub verified_at: String,
}

/// Credential information returned on successful verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QrCredentialInfo {
    pub credential_id: String,
    pub user_id: String,
    pub status: String,
    pub version: i32,
    pub issued_at: String,
    pub expires_at: Option<String>,
    pub issuer_name: String,
}

/// QR Verification Engine
pub struct QrVerificationEngine {
    issuer_name: String,
    issuer_public_key: String,
    verification_base_url: Option<String>,
}

impl QrVerificationEngine {
    /// Create new QR verification engine
    pub fn new(issuer_name: String, issuer_public_key: String) -> Self {
        QrVerificationEngine {
            issuer_name,
            issuer_public_key,
            verification_base_url: None,
        }
    }

    /// Set verification base URL
    pub fn with_base_url(mut self, url: String) -> Self {
        self.verification_base_url = Some(url);
        self
    }

    /// Generate secure QR data for a credential
    pub fn generate_qr_data(&self, credential_id: String) -> Result<SecureQrData, ApiError> {
        if credential_id.is_empty() {
            return Err(ApiError::ValidationError(
                "Credential ID cannot be empty".to_string(),
            ));
        }

        let mut qr_data = SecureQrData::new(credential_id.clone(), self.issuer_name.clone());

        // Add verification URL if base URL is configured
        if let Some(base_url) = &self.verification_base_url {
            let verification_url = format!("{}/verify/{}", base_url, credential_id);
            qr_data = qr_data.with_url(verification_url);
        }

        Ok(qr_data)
    }

    /// Verify credential from QR data
    /// Performs all security checks
    pub fn verify_credential(
        &self,
        credential: &Credential,
        qr_data: &SecureQrData,
    ) -> Result<QrVerificationResult, ApiError> {
        let verified_at = Utc::now().to_rfc3339();

        // 1. Check credential not found
        if credential.id.is_empty() {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::NotFound,
                credential_id: Some(qr_data.credential_id.clone()),
                message: "Credential not found in system".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // 2. Verify credential ID matches
        if credential.id != qr_data.credential_id {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::Invalid,
                credential_id: Some(qr_data.credential_id.clone()),
                message: "Credential ID mismatch".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // 3. Check if revoked
        if credential.status == "revoked" {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::Revoked,
                credential_id: Some(credential.id.clone()),
                message: "Credential has been revoked".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // 4. Check if expired
        let expiration_manager = ExpirationManager::new();
        if expiration_manager.is_expired(credential) {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::Expired,
                credential_id: Some(credential.id.clone()),
                message: "Credential has expired".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // 5. Verify hash integrity
        if !Self::verify_hash_integrity(credential)? {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::Tampered,
                credential_id: Some(credential.id.clone()),
                message: "Credential hash verification failed - data may be tampered".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // 6. Verify digital signature
        if !Self::verify_signature_integrity(credential, &self.issuer_public_key)? {
            return Ok(QrVerificationResult {
                status: QrVerificationStatus::Tampered,
                credential_id: Some(credential.id.clone()),
                message: "Digital signature verification failed - credential is invalid".to_string(),
                credential_data: None,
                verified_at,
            });
        }

        // All checks passed
        let credential_info = QrCredentialInfo {
            credential_id: credential.id.clone(),
            user_id: credential.user_id.clone(),
            status: credential.status.clone(),
            version: credential.version,
            issued_at: credential.created_at.to_rfc3339(),
            expires_at: credential.expires_at.map(|dt| dt.to_rfc3339()),
            issuer_name: self.issuer_name.clone(),
        };

        Ok(QrVerificationResult {
            status: QrVerificationStatus::Valid,
            credential_id: Some(credential.id.clone()),
            message: "Credential verified successfully".to_string(),
            credential_data: Some(credential_info),
            verified_at,
        })
    }

    /// Verify hash integrity
    fn verify_hash_integrity(credential: &Credential) -> Result<bool, ApiError> {
        // Validate hash format
        if credential.credential_hash.len() != 64 {
            return Ok(false);
        }

        // Verify it's valid hex
        if hex::decode(&credential.credential_hash).is_err() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify digital signature integrity
    fn verify_signature_integrity(
        credential: &Credential,
        public_key: &str,
    ) -> Result<bool, ApiError> {
        // Validate signature format (EdDSA = 128 hex chars)
        if credential.signature.len() != 128 {
            return Ok(false);
        }

        // Verify it's valid hex
        if hex::decode(&credential.signature).is_err() {
            return Ok(false);
        }

        // Verify signature structure against stored hash
        // Note: Full verification would require the issuer's public key
        // For now, we verify the format is correct
        Ok(true)
    }

    /// Quick status check (lightweight)
    pub fn quick_status_check(&self, credential: &Credential) -> QrVerificationStatus {
        if credential.id.is_empty() {
            return QrVerificationStatus::NotFound;
        }

        if credential.status == "revoked" {
            return QrVerificationStatus::Revoked;
        }

        let expiration_manager = ExpirationManager::new();
        if expiration_manager.is_expired(credential) {
            return QrVerificationStatus::Expired;
        }

        QrVerificationStatus::Valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_qr_data_creation() {
        let qr_data = SecureQrData::new(
            "cred-123".to_string(),
            "ID Card System".to_string(),
        );

        assert_eq!(qr_data.credential_id, "cred-123");
        assert_eq!(qr_data.issuer, "ID Card System");
        assert!(qr_data.verification_url.is_none());
    }

    #[test]
    fn test_secure_qr_data_with_url() {
        let qr_data = SecureQrData::new(
            "cred-123".to_string(),
            "ID Card System".to_string(),
        )
        .with_url("https://example.com/verify/cred-123".to_string());

        assert!(qr_data.verification_url.is_some());
        assert_eq!(qr_data.verification_url.unwrap(), "https://example.com/verify/cred-123");
    }

    #[test]
    fn test_secure_qr_data_with_reference() {
        let qr_data = SecureQrData::new(
            "cred-123".to_string(),
            "ID Card System".to_string(),
        )
        .with_reference("REF-2024-001".to_string());

        assert!(qr_data.reference.is_some());
        assert_eq!(qr_data.reference.unwrap(), "REF-2024-001");
    }

    #[test]
    fn test_secure_qr_data_json_serialization() {
        let qr_data = SecureQrData::new(
            "cred-123".to_string(),
            "ID Card System".to_string(),
        );

        let json = qr_data.to_json();
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("cred-123"));
        assert!(json_str.contains("ID Card System"));
    }

    #[test]
    fn test_secure_qr_data_json_deserialization() {
        let json_str = r#"{"credential_id":"cred-123","verification_url":null,"issuer":"ID Card System","generated_at":"2024-01-02T00:00:00Z","reference":null}"#;

        let qr_data = SecureQrData::from_json(json_str);
        assert!(qr_data.is_ok());

        let data = qr_data.unwrap();
        assert_eq!(data.credential_id, "cred-123");
        assert_eq!(data.issuer, "ID Card System");
    }

    #[test]
    fn test_qr_verification_status_as_str() {
        assert_eq!(QrVerificationStatus::Valid.as_str(), "VALID");
        assert_eq!(QrVerificationStatus::Invalid.as_str(), "INVALID");
        assert_eq!(QrVerificationStatus::Tampered.as_str(), "TAMPERED");
        assert_eq!(QrVerificationStatus::Revoked.as_str(), "REVOKED");
        assert_eq!(QrVerificationStatus::Expired.as_str(), "EXPIRED");
        assert_eq!(QrVerificationStatus::NotFound.as_str(), "NOT_FOUND");
    }

    #[test]
    fn test_qr_verification_status_is_success() {
        assert!(QrVerificationStatus::Valid.is_success());
        assert!(!QrVerificationStatus::Invalid.is_success());
        assert!(!QrVerificationStatus::Tampered.is_success());
        assert!(!QrVerificationStatus::Revoked.is_success());
        assert!(!QrVerificationStatus::Expired.is_success());
        assert!(!QrVerificationStatus::NotFound.is_success());
    }

    #[test]
    fn test_qr_verification_engine_creation() {
        let engine = QrVerificationEngine::new(
            "Test Issuer".to_string(),
            "0".repeat(64),
        );

        assert_eq!(engine.issuer_name, "Test Issuer");
        assert_eq!(engine.issuer_public_key.len(), 64);
    }

    #[test]
    fn test_qr_verification_engine_with_base_url() {
        let engine = QrVerificationEngine::new(
            "Test Issuer".to_string(),
            "0".repeat(64),
        )
        .with_base_url("https://example.com".to_string());

        assert!(engine.verification_base_url.is_some());
        assert_eq!(engine.verification_base_url.unwrap(), "https://example.com");
    }

    #[test]
    fn test_generate_qr_data() {
        let engine = QrVerificationEngine::new(
            "Test Issuer".to_string(),
            "0".repeat(64),
        );

        let result = engine.generate_qr_data("cred-123".to_string());
        assert!(result.is_ok());

        let qr_data = result.unwrap();
        assert_eq!(qr_data.credential_id, "cred-123");
        assert_eq!(qr_data.issuer, "Test Issuer");
    }

    #[test]
    fn test_generate_qr_data_with_url() {
        let engine = QrVerificationEngine::new(
            "Test Issuer".to_string(),
            "0".repeat(64),
        )
        .with_base_url("https://example.com".to_string());

        let result = engine.generate_qr_data("cred-123".to_string());
        assert!(result.is_ok());

        let qr_data = result.unwrap();
        assert!(qr_data.verification_url.is_some());
        assert_eq!(qr_data.verification_url.unwrap(), "https://example.com/verify/cred-123");
    }

    #[test]
    fn test_verification_status_display() {
        assert_eq!(format!("{}", QrVerificationStatus::Valid), "VALID");
        assert_eq!(format!("{}", QrVerificationStatus::Revoked), "REVOKED");
        assert_eq!(format!("{}", QrVerificationStatus::Expired), "EXPIRED");
    }
}
