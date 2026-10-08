// Credential Revocation Management
// Handles revocation operations, tracking, and audit logging

use crate::error::ApiError;
use crate::models::CredentialStatus;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Revocation reason enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum RevocationReason {
    UserRequested,
    Compromised,
    Expired,
    Replaced,
    InstitutionRequest,
    SecurityBreach,
    Other(String),
}

impl RevocationReason {
    pub fn as_str(&self) -> &str {
        match self {
            RevocationReason::UserRequested => "user_requested",
            RevocationReason::Compromised => "compromised",
            RevocationReason::Expired => "expired",
            RevocationReason::Replaced => "replaced",
            RevocationReason::InstitutionRequest => "institution_request",
            RevocationReason::SecurityBreach => "security_breach",
            RevocationReason::Other(_) => "other",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "user_requested" => Ok(RevocationReason::UserRequested),
            "compromised" => Ok(RevocationReason::Compromised),
            "expired" => Ok(RevocationReason::Expired),
            "replaced" => Ok(RevocationReason::Replaced),
            "institution_request" => Ok(RevocationReason::InstitutionRequest),
            "security_breach" => Ok(RevocationReason::SecurityBreach),
            "other" => Ok(RevocationReason::Other("other".to_string())),
            _ => Err(format!("Unknown revocation reason: {}", s)),
        }
    }
}

/// Revocation record
#[derive(Debug, Clone)]
pub struct RevocationRecord {
    pub id: String,
    pub credential_id: String,
    pub user_id: String,
    pub reason: RevocationReason,
    pub reason_details: Option<String>,
    pub revoked_by: String, // User or system identifier
    pub revoked_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl RevocationRecord {
    /// Create a new revocation record
    pub fn new(
        credential_id: String,
        user_id: String,
        reason: RevocationReason,
        reason_details: Option<String>,
        revoked_by: String,
    ) -> Self {
        let now = Utc::now();
        RevocationRecord {
            id: Uuid::new_v4().to_string(),
            credential_id,
            user_id,
            reason,
            reason_details,
            revoked_by,
            revoked_at: now,
            created_at: now,
        }
    }
}

/// Credential Revocation Manager
#[derive(Debug, Clone)]
pub struct RevocationManager {
    revocation_records: Vec<RevocationRecord>,
}

impl RevocationManager {
    /// Create a new revocation manager
    pub fn new() -> Self {
        RevocationManager {
            revocation_records: Vec::new(),
        }
    }

    /// Record a credential revocation
    pub fn revoke(
        &mut self,
        credential_id: String,
        user_id: String,
        reason: RevocationReason,
        reason_details: Option<String>,
        revoked_by: String,
    ) -> Result<RevocationRecord, ApiError> {
        if credential_id.is_empty() {
            return Err(ApiError::ValidationError(
                "Credential ID cannot be empty".to_string(),
            ));
        }

        if user_id.is_empty() {
            return Err(ApiError::ValidationError(
                "User ID cannot be empty".to_string(),
            ));
        }

        if revoked_by.is_empty() {
            return Err(ApiError::ValidationError(
                "Revoked by cannot be empty".to_string(),
            ));
        }

        // Check if already revoked
        if self.is_revoked(&credential_id) {
            return Err(ApiError::ValidationError(
                "Credential is already revoked".to_string(),
            ));
        }

        let record = RevocationRecord::new(
            credential_id,
            user_id,
            reason,
            reason_details,
            revoked_by,
        );

        self.revocation_records.push(record.clone());
        Ok(record)
    }

    /// Check if a credential is revoked
    pub fn is_revoked(&self, credential_id: &str) -> bool {
        self.revocation_records
            .iter()
            .any(|r| r.credential_id == credential_id)
    }

    /// Get revocation record for a credential
    pub fn get_revocation_record(
        &self,
        credential_id: &str,
    ) -> Result<&RevocationRecord, ApiError> {
        self.revocation_records
            .iter()
            .find(|r| r.credential_id == credential_id)
            .ok_or_else(|| {
                ApiError::ValidationError(
                    "Credential revocation record not found".to_string(),
                )
            })
    }

    /// Get all revocations for a user
    pub fn get_user_revocations(&self, user_id: &str) -> Vec<&RevocationRecord> {
        self.revocation_records
            .iter()
            .filter(|r| r.user_id == user_id)
            .collect()
    }

    /// Get revocations by reason
    pub fn get_revocations_by_reason(&self, reason: &RevocationReason) -> Vec<&RevocationRecord> {
        self.revocation_records
            .iter()
            .filter(|r| r.reason == *reason)
            .collect()
    }

    /// Get revocations in a time range
    pub fn get_revocations_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<&RevocationRecord> {
        self.revocation_records
            .iter()
            .filter(|r| r.revoked_at >= start && r.revoked_at <= end)
            .collect()
    }

    /// Get total number of revocations
    pub fn get_total_revocations(&self) -> usize {
        self.revocation_records.len()
    }

    /// Get revocation statistics
    pub fn get_statistics(&self) -> RevocationStatistics {
        let total = self.revocation_records.len();

        let mut reason_counts = std::collections::HashMap::new();
        for record in &self.revocation_records {
            *reason_counts
                .entry(record.reason.as_str().to_string())
                .or_insert(0) += 1;
        }

        let earliest_revocation = self
            .revocation_records
            .iter()
            .min_by_key(|r| r.revoked_at)
            .map(|r| r.revoked_at);

        let latest_revocation = self
            .revocation_records
            .iter()
            .max_by_key(|r| r.revoked_at)
            .map(|r| r.revoked_at);

        RevocationStatistics {
            total_revocations: total,
            reason_distribution: reason_counts,
            earliest_revocation,
            latest_revocation,
        }
    }

    /// Clear all revocation records (use with caution)
    pub fn clear(&mut self) {
        self.revocation_records.clear();
    }

    /// Get all revocation records
    pub fn get_all_records(&self) -> Vec<&RevocationRecord> {
        self.revocation_records.iter().collect()
    }
}

/// Revocation statistics
#[derive(Debug, Clone)]
pub struct RevocationStatistics {
    pub total_revocations: usize,
    pub reason_distribution: std::collections::HashMap<String, usize>,
    pub earliest_revocation: Option<DateTime<Utc>>,
    pub latest_revocation: Option<DateTime<Utc>>,
}

/// Validate revocation reason
pub fn validate_revocation_reason(reason: &str) -> Result<RevocationReason, ApiError> {
    RevocationReason::from_str(reason)
        .map_err(|e| ApiError::ValidationError(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revocation_reason_as_str() {
        assert_eq!(RevocationReason::UserRequested.as_str(), "user_requested");
        assert_eq!(RevocationReason::Compromised.as_str(), "compromised");
        assert_eq!(RevocationReason::Expired.as_str(), "expired");
    }

    #[test]
    fn test_revocation_reason_from_str() {
        let result = RevocationReason::from_str("user_requested");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RevocationReason::UserRequested);
    }

    #[test]
    fn test_revocation_record_creation() {
        let record = RevocationRecord::new(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            Some("User requested revocation".to_string()),
            "system".to_string(),
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.credential_id, "cred-123");
        assert_eq!(record.user_id, "user-456");
    }

    #[test]
    fn test_revocation_manager_creation() {
        let manager = RevocationManager::new();
        assert_eq!(manager.get_total_revocations(), 0);
    }

    #[test]
    fn test_revoke_credential() {
        let mut manager = RevocationManager::new();
        let result = manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        );

        assert!(result.is_ok());
        assert_eq!(manager.get_total_revocations(), 1);
    }

    #[test]
    fn test_revoke_already_revoked_fails() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        let result = manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_is_revoked() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        assert!(manager.is_revoked("cred-123"));
        assert!(!manager.is_revoked("cred-456"));
    }

    #[test]
    fn test_get_revocation_record() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::Compromised,
            Some("Key compromised".to_string()),
            "admin".to_string(),
        ).unwrap();

        let record = manager.get_revocation_record("cred-123");
        assert!(record.is_ok());
        assert_eq!(record.unwrap().reason, RevocationReason::Compromised);
    }

    #[test]
    fn test_get_user_revocations() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        manager.revoke(
            "cred-124".to_string(),
            "user-456".to_string(),
            RevocationReason::Replaced,
            None,
            "system".to_string(),
        ).unwrap();

        let revocations = manager.get_user_revocations("user-456");
        assert_eq!(revocations.len(), 2);
    }

    #[test]
    fn test_get_revocations_by_reason() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::Compromised,
            None,
            "system".to_string(),
        ).unwrap();

        manager.revoke(
            "cred-124".to_string(),
            "user-789".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        let compromised = manager.get_revocations_by_reason(&RevocationReason::Compromised);
        assert_eq!(compromised.len(), 1);
    }

    #[test]
    fn test_get_statistics() {
        let mut manager = RevocationManager::new();
        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::Compromised,
            None,
            "system".to_string(),
        ).unwrap();

        manager.revoke(
            "cred-124".to_string(),
            "user-789".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        let stats = manager.get_statistics();
        assert_eq!(stats.total_revocations, 2);
        assert_eq!(stats.reason_distribution.len(), 2);
    }

    #[test]
    fn test_get_revocations_in_range() {
        let mut manager = RevocationManager::new();
        let now = Utc::now();

        manager.revoke(
            "cred-123".to_string(),
            "user-456".to_string(),
            RevocationReason::UserRequested,
            None,
            "system".to_string(),
        ).unwrap();

        let before = now - chrono::Duration::hours(1);
        let after = now + chrono::Duration::hours(1);

        let revocations = manager.get_revocations_in_range(before, after);
        assert_eq!(revocations.len(), 1);
    }

    #[test]
    fn test_validate_revocation_reason() {
        let result = validate_revocation_reason("compromised");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), RevocationReason::Compromised);
    }

    #[test]
    fn test_validate_revocation_reason_invalid() {
        let result = validate_revocation_reason("invalid_reason");
        assert!(result.is_err());
    }
}
