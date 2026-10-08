// Canonical credential representation for hashing and signing
// Ensures consistent JSON serialization across all operations

use serde::{Deserialize, Serialize};
use serde_json::json;
use chrono::{DateTime, Utc};

/// Canonical representation of a credential for hashing and signing
/// This struct defines the exact fields and their order for consistent serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalCredential {
    pub id: String,
    pub user_id: String,
    pub version: i32,
    pub schema_hash: String,
    pub qr_code: String,
    pub qr_hash: String,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CanonicalCredential {
    /// Create a canonical credential from a credential's core fields
    pub fn new(
        id: String,
        user_id: String,
        version: i32,
        schema_hash: String,
        qr_code: String,
        qr_hash: String,
        status: String,
        signed_at: Option<DateTime<Utc>>,
        expires_at: Option<DateTime<Utc>>,
        revoked_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    ) -> Self {
        CanonicalCredential {
            id,
            user_id,
            version,
            schema_hash,
            qr_code,
            qr_hash,
            status,
            signed_at,
            expires_at,
            revoked_at,
            created_at,
        }
    }

    /// Serialize credential to canonical JSON string
    /// Uses sorted keys to ensure consistent output across runs
    pub fn to_canonical_json(&self) -> Result<String, serde_json::Error> {
        // Manually construct JSON with sorted keys to ensure consistency
        let json_value = json!({
            "created_at": self.created_at.to_rfc3339(),
            "expires_at": self.expires_at.map(|dt| dt.to_rfc3339()),
            "id": self.id,
            "qr_code": self.qr_code,
            "qr_hash": self.qr_hash,
            "revoked_at": self.revoked_at.map(|dt| dt.to_rfc3339()),
            "schema_hash": self.schema_hash,
            "signed_at": self.signed_at.map(|dt| dt.to_rfc3339()),
            "status": self.status,
            "user_id": self.user_id,
            "version": self.version,
        });

        // Serialize to compact JSON (no extra whitespace)
        serde_json::to_string(&json_value)
    }

    /// Serialize credential to canonical JSON bytes
    pub fn to_canonical_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        self.to_canonical_json().map(|s| s.into_bytes())
    }

    /// Parse canonical JSON string back to CanonicalCredential
    pub fn from_canonical_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_json_consistency() {
        let cred = CanonicalCredential::new(
            "cred-123".to_string(),
            "user-456".to_string(),
            1,
            "abc123def456abc123def456abc123de",
            "qr-data-here".to_string(),
            "qr-hash-789".to_string(),
            "active".to_string(),
            None,
            None,
            None,
            Utc::now(),
        );

        // Serialize twice and verify they're identical
        let json1 = cred.to_canonical_json().expect("Failed to serialize");
        let json2 = cred.to_canonical_json().expect("Failed to serialize");

        assert_eq!(json1, json2, "Canonical JSON should be identical across multiple serializations");
    }

    #[test]
    fn test_canonical_json_roundtrip() {
        let now = Utc::now();
        let original = CanonicalCredential::new(
            "cred-123".to_string(),
            "user-456".to_string(),
            1,
            "abc123def456abc123def456abc123de",
            "qr-data-here".to_string(),
            "qr-hash-789".to_string(),
            "active".to_string(),
            Some(now),
            None,
            None,
            now,
        );

        let json = original.to_canonical_json().expect("Failed to serialize");
        let deserialized = CanonicalCredential::from_canonical_json(&json)
            .expect("Failed to deserialize");

        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.user_id, deserialized.user_id);
        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.schema_hash, deserialized.schema_hash);
        assert_eq!(original.status, deserialized.status);
    }

    #[test]
    fn test_canonical_json_has_sorted_keys() {
        let cred = CanonicalCredential::new(
            "cred-123".to_string(),
            "user-456".to_string(),
            1,
            "abc123def456abc123def456abc123de",
            "qr-data".to_string(),
            "qr-hash".to_string(),
            "active".to_string(),
            None,
            None,
            None,
            Utc::now(),
        );

        let json = cred.to_canonical_json().expect("Failed to serialize");

        // Verify keys appear in alphabetical order
        let created_at_pos = json.find("\"created_at\"").expect("created_at not found");
        let expires_at_pos = json.find("\"expires_at\"").expect("expires_at not found");
        let id_pos = json.find("\"id\"").expect("id not found");
        let user_id_pos = json.find("\"user_id\"").expect("user_id not found");

        assert!(created_at_pos < expires_at_pos, "created_at should come before expires_at");
        assert!(expires_at_pos < id_pos, "expires_at should come before id");
        assert!(user_id_pos > id_pos, "user_id should come after id");
    }

    #[test]
    fn test_canonical_json_compact_no_whitespace() {
        let cred = CanonicalCredential::new(
            "cred".to_string(),
            "user".to_string(),
            1,
            "hash".to_string(),
            "qr".to_string(),
            "qr-hash".to_string(),
            "active".to_string(),
            None,
            None,
            None,
            Utc::now(),
        );

        let json = cred.to_canonical_json().expect("Failed to serialize");

        // Compact JSON should not contain unnecessary whitespace
        assert!(!json.contains(": "), "Compact JSON should not have spaces after colons");
        assert!(!json.contains("  "), "Compact JSON should not have double spaces");
    }
}
