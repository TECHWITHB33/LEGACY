// Credential Issuer Service
// Orchestrates credential creation, signing, hashing, and lifecycle management

use crate::crypto::{
    CanonicalCredential, hash::*, signature::*,
};
use crate::credential_status::CredentialLifecycle;
use crate::credential_versioning::VersioningSystem;
use crate::error::ApiError;
use crate::models::{Credential, CredentialStatus};
use crate::validators;
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

/// Credential Issuer Service
/// Manages credential creation, signing, verification, and lifecycle
pub struct CredentialIssuerService {
    issuer_name: String,
    issuer_public_key: String,
    issuer_private_key: String, // Kept in memory during service lifetime only
}

impl CredentialIssuerService {
    /// Create a new credential issuer service
    /// 
    /// # Arguments
    /// * `issuer_name` - Name of the issuing authority
    /// * `issuer_public_key` - Public key for verification (hex encoded)
    /// * `issuer_private_key` - Private key for signing (hex encoded) - DO NOT STORE IN CODE
    pub fn new(
        issuer_name: String,
        issuer_public_key: String,
        issuer_private_key: String,
    ) -> Result<Self, ApiError> {
        if issuer_name.is_empty() {
            return Err(ApiError::ValidationError(
                "Issuer name cannot be empty".to_string(),
            ));
        }

        if issuer_public_key.len() != 64 {
            return Err(ApiError::ValidationError(
                "Public key must be 64 hex characters (32 bytes)".to_string(),
            ));
        }

        if issuer_private_key.len() != 64 {
            return Err(ApiError::ValidationError(
                "Private key must be 64 hex characters (32 bytes)".to_string(),
            ));
        }

        Ok(CredentialIssuerService {
            issuer_name,
            issuer_public_key,
            issuer_private_key,
        })
    }

    /// Create a new credential
    pub fn issue_credential(
        &self,
        user_id: String,
        schema_hash: String,
        qr_code: String,
        expires_in_days: Option<u32>,
    ) -> Result<(Credential, String), ApiError> {
        // Validate inputs
        validate_schema_hash(&schema_hash)?;
        validators::validate_qr_code(&qr_code)?;

        // Generate credential ID
        let credential_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // Calculate QR hash
        let qr_hash = hash_qr_code(&qr_code)?;

        // Create canonical representation for hashing
        let canonical = CanonicalCredential::new(
            credential_id.clone(),
            user_id.clone(),
            1, // Initial version
            schema_hash.clone(),
            qr_code.clone(),
            qr_hash.clone(),
            CredentialStatus::Active.as_str().to_string(),
            Some(now),
            expires_in_days
                .map(|d| now + Duration::days(d as i64))
                .or(Some(now + Duration::days(365))), // Default 1 year
            None,
            now,
        );

        // Hash credential
        let credential_hash = hash_credential_object(&canonical)
            .map_err(|_| ApiError::CryptoError("Failed to hash credential".to_string()))?;

        // Sign credential
        let signature = sign_credential(&credential_hash, &self.issuer_private_key)?;

        // Validate signature format
        validators::validate_signature(&signature)?;

        // Calculate expiration
        let expires_at = expires_in_days
            .map(|d| now + Duration::days(d as i64))
            .or(Some(now + Duration::days(365)));

        // Create credential record
        let credential = Credential {
            id: credential_id.clone(),
            user_id,
            public_key_hash: crate::crypto::hash::sha256_hex_str(&self.issuer_public_key),
            qr_code,
            qr_hash,
            status: CredentialStatus::Active.as_str().to_string(),
            version: 1,
            schema_hash,
            credential_hash: credential_hash.clone(),
            signature,
            signed_at: Some(now),
            expires_at,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        };

        Ok((credential, credential_id))
    }

    /// Verify a credential's signature and integrity
    pub fn verify_credential(
        &self,
        credential: &Credential,
    ) -> Result<bool, ApiError> {
        // Verify schema hash format
        validate_schema_hash(&credential.schema_hash)?;

        // Verify credential hash format
        validators::validate_credential_hash(&credential.credential_hash)?;

        // Verify signature format
        validators::validate_signature(&credential.signature)?;

        // Verify signature with issuer's public key
        verify_credential_signature(
            &credential.credential_hash,
            &credential.signature,
            &self.issuer_public_key,
        )
    }

    /// Verify credential against canonical JSON
    pub fn verify_credential_json(
        &self,
        credential: &Credential,
        canonical_json: &str,
    ) -> Result<bool, ApiError> {
        // Hash the canonical JSON
        let computed_hash = hash_credential(canonical_json)?;

        // Compare with stored hash
        if computed_hash != credential.credential_hash {
            return Ok(false);
        }

        // Verify signature
        verify_credential_signature(
            &credential.credential_hash,
            &credential.signature,
            &self.issuer_public_key,
        )
    }

    /// Get issuer information
    pub fn get_issuer_info(&self) -> IssuerInfo {
        IssuerInfo {
            name: self.issuer_name.clone(),
            public_key: self.issuer_public_key.clone(),
        }
    }

    /// Create credential lifecycle manager
    pub fn create_lifecycle(&self, credential: &Credential) -> Result<CredentialLifecycle, ApiError> {
        let mut lifecycle = CredentialLifecycle::new(credential.created_at);

        lifecycle.status = CredentialStatus::from_str(&credential.status)
            .map_err(|e| ApiError::ValidationError(e))?;

        if let Some(signed_at) = credential.signed_at {
            lifecycle.signed_at = Some(signed_at);
        }

        lifecycle.expires_at = credential.expires_at;
        lifecycle.revoked_at = credential.revoked_at;

        Ok(lifecycle)
    }

    /// Create versioning system for credential
    pub fn create_versioning(&self) -> VersioningSystem {
        VersioningSystem::new()
    }

    /// Revoke a credential
    pub fn revoke_credential(
        &self,
        credential: &mut Credential,
    ) -> Result<(), ApiError> {
        if credential.status == CredentialStatus::Revoked.as_str() {
            return Err(ApiError::ValidationError(
                "Credential is already revoked".to_string(),
            ));
        }

        credential.status = CredentialStatus::Revoked.as_str().to_string();
        credential.revoked_at = Some(Utc::now());
        credential.updated_at = Utc::now();

        Ok(())
    }

    /// Check if credential is expired
    pub fn is_expired(&self, credential: &Credential) -> bool {
        if let Some(expires_at) = credential.expires_at {
            return Utc::now() >= expires_at;
        }
        false
    }

    /// Get credential status
    pub fn get_status(&self, credential: &Credential) -> CredentialStatus {
        // Check if revoked
        if credential.status == CredentialStatus::Revoked.as_str() {
            return CredentialStatus::Revoked;
        }

        // Check if suspended
        if credential.status == CredentialStatus::Suspended.as_str() {
            return CredentialStatus::Suspended;
        }

        // Check if expired
        if self.is_expired(credential) {
            return CredentialStatus::Expired;
        }

        CredentialStatus::Active
    }

    /// Get remaining validity period
    pub fn get_remaining_validity(&self, credential: &Credential) -> Option<Duration> {
        credential.expires_at.map(|expires| {
            let now = Utc::now();
            if now >= expires {
                Duration::seconds(0)
            } else {
                expires - now
            }
        })
    }

    /// Verify multiple credentials
    pub fn verify_multiple(
        &self,
        credentials: Vec<&Credential>,
    ) -> Result<Vec<bool>, ApiError> {
        let mut results = Vec::new();

        for credential in credentials {
            let verified = self.verify_credential(credential)?;
            results.push(verified);
        }

        Ok(results)
    }
}

/// Issuer information
#[derive(Debug, Clone)]
pub struct IssuerInfo {
    pub name: String,
    pub public_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_issuer() -> Result<CredentialIssuerService, ApiError> {
        let (private_key, public_key) = crate::crypto::signature::generate_signature_keypair()?;
        CredentialIssuerService::new(
            "Test Issuer".to_string(),
            public_key,
            private_key,
        )
    }

    #[test]
    fn test_issuer_creation() {
        let result = create_test_issuer();
        assert!(result.is_ok());
    }

    #[test]
    fn test_issuer_creation_empty_name_fails() {
        let result = CredentialIssuerService::new(
            "".to_string(),
            "0".repeat(64),
            "0".repeat(64),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_credential() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let result = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365));
        assert!(result.is_ok());

        let (credential, _) = result.unwrap();
        assert_eq!(credential.version, 1);
        assert_eq!(credential.status, CredentialStatus::Active.as_str());
        assert!(credential.signed_at.is_some());
        assert!(credential.expires_at.is_some());
    }

    #[test]
    fn test_verify_credential() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let (credential, _) = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365)).unwrap();
        let verified = issuer.verify_credential(&credential);

        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }

    #[test]
    fn test_get_issuer_info() {
        let issuer = create_test_issuer().unwrap();
        let info = issuer.get_issuer_info();

        assert_eq!(info.name, "Test Issuer");
        assert_eq!(info.public_key.len(), 64);
    }

    #[test]
    fn test_revoke_credential() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let (mut credential, _) = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365)).unwrap();
        let result = issuer.revoke_credential(&mut credential);

        assert!(result.is_ok());
        assert_eq!(credential.status, CredentialStatus::Revoked.as_str());
        assert!(credential.revoked_at.is_some());
    }

    #[test]
    fn test_is_expired() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let (credential, _) = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365)).unwrap();
        assert!(!issuer.is_expired(&credential));
    }

    #[test]
    fn test_get_status() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let (credential, _) = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365)).unwrap();
        let status = issuer.get_status(&credential);

        assert_eq!(status, CredentialStatus::Active);
    }

    #[test]
    fn test_get_remaining_validity() {
        let issuer = create_test_issuer().unwrap();
        let user_id = "user-123".to_string();
        let schema_hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let qr_code = "https://example.com/verify?id=abc123".to_string();

        let (credential, _) = issuer.issue_credential(user_id, schema_hash, qr_code, Some(365)).unwrap();
        let remaining = issuer.get_remaining_validity(&credential);

        assert!(remaining.is_some());
        let duration = remaining.unwrap();
        assert!(duration.num_days() >= 364 && duration.num_days() <= 365);
    }
}
