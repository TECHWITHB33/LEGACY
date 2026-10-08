// Credential versioning and history management
// Tracks credential versions, updates, and maintains version history

use crate::error::ApiError;
use chrono::{DateTime, Utc};

/// Represents a credential version entry
#[derive(Debug, Clone)]
pub struct CredentialVersion {
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub credential_hash: String,
    pub signature: String,
    pub changes: Option<String>, // JSON of what changed
}

/// Credential versioning system
#[derive(Debug, Clone)]
pub struct VersioningSystem {
    pub current_version: i32,
    pub versions: Vec<CredentialVersion>,
}

impl VersioningSystem {
    /// Create a new versioning system starting at version 1
    pub fn new() -> Self {
        VersioningSystem {
            current_version: 1,
            versions: Vec::new(),
        }
    }

    /// Create a new version record
    pub fn create_version(
        &mut self,
        credential_hash: String,
        signature: String,
        changes: Option<String>,
    ) -> Result<i32, ApiError> {
        if credential_hash.is_empty() {
            return Err(ApiError::ValidationError(
                "Credential hash cannot be empty".to_string(),
            ));
        }

        if signature.is_empty() {
            return Err(ApiError::ValidationError(
                "Signature cannot be empty".to_string(),
            ));
        }

        let version = CredentialVersion {
            version: self.current_version,
            created_at: Utc::now(),
            credential_hash,
            signature,
            changes,
        };

        self.versions.push(version);
        let version_number = self.current_version;
        self.current_version += 1;

        Ok(version_number)
    }

    /// Get a specific version by number
    pub fn get_version(&self, version_num: i32) -> Result<&CredentialVersion, ApiError> {
        self.versions
            .iter()
            .find(|v| v.version == version_num)
            .ok_or_else(|| {
                ApiError::ValidationError(format!("Version {} not found", version_num))
            })
    }

    /// Get the latest version
    pub fn get_latest_version(&self) -> Result<&CredentialVersion, ApiError> {
        self.versions
            .last()
            .ok_or_else(|| ApiError::ValidationError("No versions found".to_string()))
    }

    /// Get version history
    pub fn get_history(&self) -> Vec<&CredentialVersion> {
        self.versions.iter().collect()
    }

    /// Check if a version exists
    pub fn version_exists(&self, version_num: i32) -> bool {
        self.versions.iter().any(|v| v.version == version_num)
    }

    /// Get number of versions
    pub fn version_count(&self) -> usize {
        self.versions.len()
    }

    /// Verify a credential against a specific version
    pub fn verify_version(
        &self,
        version_num: i32,
        credential_hash: &str,
        signature: &str,
    ) -> Result<bool, ApiError> {
        let version = self.get_version(version_num)?;

        if version.credential_hash != credential_hash {
            return Ok(false);
        }

        if version.signature != signature {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get differences between two versions
    pub fn get_diff(
        &self,
        version1: i32,
        version2: i32,
    ) -> Result<(i32, i32), ApiError> {
        if !self.version_exists(version1) {
            return Err(ApiError::ValidationError(format!(
                "Version {} not found",
                version1
            )));
        }

        if !self.version_exists(version2) {
            return Err(ApiError::ValidationError(format!(
                "Version {} not found",
                version2
            )));
        }

        Ok((version1, version2))
    }

    /// Invalidate a version (mark as superseded)
    pub fn invalidate_version(&mut self, version_num: i32) -> Result<(), ApiError> {
        if let Some(version) = self.versions.iter_mut().find(|v| v.version == version_num) {
            // Mark as invalidated by setting a flag or updating
            // For now, we just validate it exists
            Ok(())
        } else {
            Err(ApiError::ValidationError(format!(
                "Version {} not found",
                version_num
            )))
        }
    }

    /// Get all versions created after a specific time
    pub fn get_versions_after(&self, timestamp: DateTime<Utc>) -> Vec<&CredentialVersion> {
        self.versions
            .iter()
            .filter(|v| v.created_at > timestamp)
            .collect()
    }

    /// Get version statistics
    pub fn get_stats(&self) -> VersioningStats {
        VersioningStats {
            total_versions: self.versions.len(),
            current_version: self.current_version,
            earliest_version: self.versions.first().map(|v| v.version),
            latest_version: self.versions.last().map(|v| v.version),
            earliest_timestamp: self.versions.first().map(|v| v.created_at),
            latest_timestamp: self.versions.last().map(|v| v.created_at),
        }
    }
}

/// Statistics about credential versions
#[derive(Debug, Clone)]
pub struct VersioningStats {
    pub total_versions: usize,
    pub current_version: i32,
    pub earliest_version: Option<i32>,
    pub latest_version: Option<i32>,
    pub earliest_timestamp: Option<DateTime<Utc>>,
    pub latest_timestamp: Option<DateTime<Utc>>,
}

/// Validate version number is positive
pub fn validate_version_number(version: i32) -> Result<i32, ApiError> {
    if version <= 0 {
        return Err(ApiError::ValidationError(
            "Version number must be greater than 0".to_string(),
        ));
    }

    Ok(version)
}

/// Check if version is incrementing correctly
pub fn validate_version_sequence(previous: i32, current: i32) -> Result<(), ApiError> {
    if current != previous + 1 {
        return Err(ApiError::ValidationError(
            format!("Version should increment by 1, got {} -> {}", previous, current)
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioning_system_creation() {
        let vs = VersioningSystem::new();
        assert_eq!(vs.current_version, 1);
        assert_eq!(vs.versions.len(), 0);
    }

    #[test]
    fn test_create_version() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let sig = "0".repeat(128);

        let version = vs.create_version(hash, sig, None);
        assert!(version.is_ok());
        assert_eq!(version.unwrap(), 1);
        assert_eq!(vs.current_version, 2);
        assert_eq!(vs.versions.len(), 1);
    }

    #[test]
    fn test_create_version_empty_hash_fails() {
        let mut vs = VersioningSystem::new();
        let result = vs.create_version("".to_string(), "sig".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_version_empty_signature_fails() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let result = vs.create_version(hash, "".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_version() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash.clone(), sig, None).unwrap();

        let version = vs.get_version(1);
        assert!(version.is_ok());
        assert_eq!(version.unwrap().version, 1);
        assert_eq!(version.unwrap().credential_hash, hash);
    }

    #[test]
    fn test_get_version_not_found() {
        let vs = VersioningSystem::new();
        let result = vs.get_version(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_latest_version() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash1, sig.clone(), None).unwrap();
        vs.create_version(hash2.clone(), sig, None).unwrap();

        let latest = vs.get_latest_version();
        assert!(latest.is_ok());
        assert_eq!(latest.unwrap().version, 2);
        assert_eq!(latest.unwrap().credential_hash, hash2);
    }

    #[test]
    fn test_get_history() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash1, sig.clone(), None).unwrap();
        vs.create_version(hash2, sig, None).unwrap();

        let history = vs.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[1].version, 2);
    }

    #[test]
    fn test_version_exists() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash, sig, None).unwrap();

        assert!(vs.version_exists(1));
        assert!(!vs.version_exists(2));
    }

    #[test]
    fn test_version_count() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        assert_eq!(vs.version_count(), 0);

        vs.create_version(hash1, sig.clone(), None).unwrap();
        assert_eq!(vs.version_count(), 1);

        vs.create_version(hash2, sig, None).unwrap();
        assert_eq!(vs.version_count(), 2);
    }

    #[test]
    fn test_verify_version() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash.clone(), sig.clone(), None).unwrap();

        let result = vs.verify_version(1, &hash, &sig);
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_verify_version_wrong_hash() {
        let mut vs = VersioningSystem::new();
        let hash = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let wrong_hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash, sig.clone(), None).unwrap();

        let result = vs.verify_version(1, &wrong_hash, &sig);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_get_diff() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash1, sig.clone(), None).unwrap();
        vs.create_version(hash2, sig, None).unwrap();

        let result = vs.get_diff(1, 2);
        assert!(result.is_ok());
        let (v1, v2) = result.unwrap();
        assert_eq!(v1, 1);
        assert_eq!(v2, 2);
    }

    #[test]
    fn test_get_versions_after() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        let before = Utc::now();
        vs.create_version(hash1, sig.clone(), None).unwrap();
        let after_first = Utc::now();
        vs.create_version(hash2, sig, None).unwrap();

        let versions_after = vs.get_versions_after(before);
        assert_eq!(versions_after.len(), 2);

        let versions_after_first = vs.get_versions_after(after_first);
        assert_eq!(versions_after_first.len(), 1);
    }

    #[test]
    fn test_get_stats() {
        let mut vs = VersioningSystem::new();
        let hash1 = "abc123def456abc123def456abc123deabc123def456abc123def456abc123de".to_string();
        let hash2 = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let sig = "0".repeat(128);

        vs.create_version(hash1, sig.clone(), None).unwrap();
        vs.create_version(hash2, sig, None).unwrap();

        let stats = vs.get_stats();
        assert_eq!(stats.total_versions, 2);
        assert_eq!(stats.current_version, 3);
        assert_eq!(stats.earliest_version, Some(1));
        assert_eq!(stats.latest_version, Some(2));
    }

    #[test]
    fn test_validate_version_number_positive() {
        let result = validate_version_number(1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);

        let result = validate_version_number(100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_version_number_zero_fails() {
        let result = validate_version_number(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_version_number_negative_fails() {
        let result = validate_version_number(-1);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_version_sequence_correct() {
        let result = validate_version_sequence(1, 2);
        assert!(result.is_ok());

        let result = validate_version_sequence(99, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_version_sequence_incorrect() {
        let result = validate_version_sequence(1, 3);
        assert!(result.is_err());

        let result = validate_version_sequence(5, 5);
        assert!(result.is_err());
    }
}
