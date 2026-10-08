// Credential Expiration Management
// Handles expiration detection, warnings, and renewal logic

use crate::error::ApiError;
use crate::models::{Credential, CredentialStatus};
use chrono::{DateTime, Duration, Utc};

/// Expiration status of a credential
#[derive(Debug, Clone, PartialEq)]
pub enum ExpirationStatus {
    /// Credential is valid and far from expiration
    Valid,
    /// Credential is valid but approaching expiration (warning period)
    Warning,
    /// Credential is expired
    Expired,
    /// Credential never expires (no expiration date set)
    NoExpiration,
}

impl ExpirationStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ExpirationStatus::Valid => "valid",
            ExpirationStatus::Warning => "warning",
            ExpirationStatus::Expired => "expired",
            ExpirationStatus::NoExpiration => "no_expiration",
        }
    }
}

/// Expiration information for a credential
#[derive(Debug, Clone)]
pub struct ExpirationInfo {
    pub status: ExpirationStatus,
    pub expires_at: Option<DateTime<Utc>>,
    pub time_remaining: Option<Duration>,
    pub is_expired: bool,
    pub days_remaining: Option<i64>,
    pub percentage_remaining: Option<f64>,
}

impl ExpirationInfo {
    /// Create expiration info for a credential
    pub fn from_credential(credential: &Credential) -> Self {
        let now = Utc::now();
        
        if let Some(expires_at) = credential.expires_at {
            let time_remaining = if now >= expires_at {
                None
            } else {
                Some(expires_at - now)
            };

            let is_expired = now >= expires_at;
            let days_remaining = time_remaining.map(|d| d.num_days());

            // Calculate percentage remaining based on creation time
            let total_duration = expires_at - credential.created_at;
            let time_used = now - credential.created_at;
            let percentage_remaining = if total_duration.num_seconds() > 0 {
                Some(100.0 * (1.0 - (time_used.num_seconds() as f64 / total_duration.num_seconds() as f64)))
            } else {
                None
            };

            let status = if is_expired {
                ExpirationStatus::Expired
            } else if let Some(duration) = time_remaining {
                // Warning if less than 30 days remaining
                if duration.num_days() < 30 {
                    ExpirationStatus::Warning
                } else {
                    ExpirationStatus::Valid
                }
            } else {
                ExpirationStatus::Valid
            };

            ExpirationInfo {
                status,
                expires_at: Some(expires_at),
                time_remaining,
                is_expired,
                days_remaining,
                percentage_remaining,
            }
        } else {
            ExpirationInfo {
                status: ExpirationStatus::NoExpiration,
                expires_at: None,
                time_remaining: None,
                is_expired: false,
                days_remaining: None,
                percentage_remaining: None,
            }
        }
    }
}

/// Credential Expiration Manager
#[derive(Debug, Clone)]
pub struct ExpirationManager {
    warning_threshold_days: i64, // Days before expiration to issue warning
}

impl ExpirationManager {
    /// Create a new expiration manager with default warning threshold (30 days)
    pub fn new() -> Self {
        ExpirationManager {
            warning_threshold_days: 30,
        }
    }

    /// Create a new expiration manager with custom warning threshold
    pub fn with_threshold(threshold_days: i64) -> Result<Self, ApiError> {
        if threshold_days < 1 {
            return Err(ApiError::ValidationError(
                "Warning threshold must be at least 1 day".to_string(),
            ));
        }

        Ok(ExpirationManager {
            warning_threshold_days: threshold_days,
        })
    }

    /// Check if a credential is expired
    pub fn is_expired(&self, credential: &Credential) -> bool {
        if let Some(expires_at) = credential.expires_at {
            Utc::now() >= expires_at
        } else {
            false
        }
    }

    /// Check if a credential is approaching expiration (in warning period)
    pub fn is_expiring_soon(&self, credential: &Credential) -> bool {
        if let Some(expires_at) = credential.expires_at {
            let now = Utc::now();
            let days_remaining = (expires_at - now).num_days();
            days_remaining > 0 && days_remaining <= self.warning_threshold_days
        } else {
            false
        }
    }

    /// Get expiration status
    pub fn get_status(&self, credential: &Credential) -> ExpirationStatus {
        if self.is_expired(credential) {
            ExpirationStatus::Expired
        } else if self.is_expiring_soon(credential) {
            ExpirationStatus::Warning
        } else if credential.expires_at.is_some() {
            ExpirationStatus::Valid
        } else {
            ExpirationStatus::NoExpiration
        }
    }

    /// Get expiration info
    pub fn get_info(&self, credential: &Credential) -> ExpirationInfo {
        ExpirationInfo::from_credential(credential)
    }

    /// Get days remaining until expiration
    pub fn get_days_remaining(&self, credential: &Credential) -> Option<i64> {
        if let Some(expires_at) = credential.expires_at {
            let now = Utc::now();
            if now >= expires_at {
                Some(0)
            } else {
                Some((expires_at - now).num_days())
            }
        } else {
            None
        }
    }

    /// Check if credential will expire within a specified number of days
    pub fn will_expire_in(&self, credential: &Credential, days: i64) -> bool {
        if let Some(days_remaining) = self.get_days_remaining(credential) {
            days_remaining > 0 && days_remaining <= days
        } else {
            false
        }
    }

    /// Get expiration percentage (0-100)
    pub fn get_expiration_percentage(&self, credential: &Credential) -> Option<f64> {
        if let Some(expires_at) = credential.expires_at {
            let total_duration = expires_at - credential.created_at;
            let time_used = Utc::now() - credential.created_at;
            
            if total_duration.num_seconds() > 0 {
                Some(100.0 * (time_used.num_seconds() as f64 / total_duration.num_seconds() as f64))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Filter credentials that are expired
    pub fn get_expired_credentials<'a>(
        &self,
        credentials: Vec<&'a Credential>,
    ) -> Vec<&'a Credential> {
        credentials
            .into_iter()
            .filter(|c| self.is_expired(c))
            .collect()
    }

    /// Filter credentials that are expiring soon
    pub fn get_expiring_soon_credentials<'a>(
        &self,
        credentials: Vec<&'a Credential>,
    ) -> Vec<&'a Credential> {
        credentials
            .into_iter()
            .filter(|c| self.is_expiring_soon(c))
            .collect()
    }

    /// Filter credentials that are valid
    pub fn get_valid_credentials<'a>(
        &self,
        credentials: Vec<&'a Credential>,
    ) -> Vec<&'a Credential> {
        credentials
            .into_iter()
            .filter(|c| !self.is_expired(c))
            .collect()
    }

    /// Calculate renewal deadline (days before expiration)
    pub fn calculate_renewal_deadline(
        &self,
        credential: &Credential,
        renewal_window_days: i64,
    ) -> Result<Option<DateTime<Utc>>, ApiError> {
        if renewal_window_days < 1 {
            return Err(ApiError::ValidationError(
                "Renewal window must be at least 1 day".to_string(),
            ));
        }

        if let Some(expires_at) = credential.expires_at {
            Ok(Some(expires_at - Duration::days(renewal_window_days)))
        } else {
            Ok(None)
        }
    }

    /// Check if credential can be renewed
    pub fn can_renew(&self, credential: &Credential) -> bool {
        // Can renew if not revoked and not too long expired
        if credential.status == CredentialStatus::Revoked.as_str() {
            return false;
        }

        if let Some(expires_at) = credential.expires_at {
            let now = Utc::now();
            let days_expired = (now - expires_at).num_days();
            // Can renew up to 90 days after expiration
            days_expired <= 90
        } else {
            true // Never expires, can always renew
        }
    }

    /// Set warning threshold
    pub fn set_warning_threshold(&mut self, days: i64) -> Result<(), ApiError> {
        if days < 1 {
            return Err(ApiError::ValidationError(
                "Warning threshold must be at least 1 day".to_string(),
            ));
        }

        self.warning_threshold_days = days;
        Ok(())
    }

    /// Get warning threshold
    pub fn get_warning_threshold(&self) -> i64 {
        self.warning_threshold_days
    }
}

/// Validate expiration date
pub fn validate_expiration_date(expires_at: DateTime<Utc>) -> Result<DateTime<Utc>, ApiError> {
    if expires_at <= Utc::now() {
        return Err(ApiError::ValidationError(
            "Expiration date must be in the future".to_string(),
        ));
    }

    Ok(expires_at)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_credential(expires_at: Option<DateTime<Utc>>) -> Credential {
        let now = Utc::now();
        Credential {
            id: "cred-123".to_string(),
            user_id: "user-456".to_string(),
            public_key_hash: "hash".to_string(),
            qr_code: "qr".to_string(),
            qr_hash: "qr_hash".to_string(),
            status: CredentialStatus::Active.as_str().to_string(),
            version: 1,
            schema_hash: "schema".to_string(),
            credential_hash: "cred_hash".to_string(),
            signature: "sig".to_string(),
            signed_at: Some(now),
            expires_at,
            revoked_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_expiration_manager_creation() {
        let manager = ExpirationManager::new();
        assert_eq!(manager.get_warning_threshold(), 30);
    }

    #[test]
    fn test_expiration_manager_custom_threshold() {
        let result = ExpirationManager::with_threshold(60);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().get_warning_threshold(), 60);
    }

    #[test]
    fn test_expiration_manager_invalid_threshold() {
        let result = ExpirationManager::with_threshold(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_expired_false() {
        let manager = ExpirationManager::new();
        let future = Utc::now() + Duration::days(30);
        let credential = create_test_credential(Some(future));

        assert!(!manager.is_expired(&credential));
    }

    #[test]
    fn test_is_expired_true() {
        let manager = ExpirationManager::new();
        let past = Utc::now() - Duration::days(1);
        let credential = create_test_credential(Some(past));

        assert!(manager.is_expired(&credential));
    }

    #[test]
    fn test_is_expiring_soon() {
        let manager = ExpirationManager::new();
        let soon = Utc::now() + Duration::days(15);
        let credential = create_test_credential(Some(soon));

        assert!(manager.is_expiring_soon(&credential));
    }

    #[test]
    fn test_is_expiring_soon_false() {
        let manager = ExpirationManager::new();
        let far_future = Utc::now() + Duration::days(100);
        let credential = create_test_credential(Some(far_future));

        assert!(!manager.is_expiring_soon(&credential));
    }

    #[test]
    fn test_get_status_valid() {
        let manager = ExpirationManager::new();
        let future = Utc::now() + Duration::days(100);
        let credential = create_test_credential(Some(future));

        let status = manager.get_status(&credential);
        assert_eq!(status, ExpirationStatus::Valid);
    }

    #[test]
    fn test_get_status_warning() {
        let manager = ExpirationManager::new();
        let soon = Utc::now() + Duration::days(15);
        let credential = create_test_credential(Some(soon));

        let status = manager.get_status(&credential);
        assert_eq!(status, ExpirationStatus::Warning);
    }

    #[test]
    fn test_get_status_expired() {
        let manager = ExpirationManager::new();
        let past = Utc::now() - Duration::days(1);
        let credential = create_test_credential(Some(past));

        let status = manager.get_status(&credential);
        assert_eq!(status, ExpirationStatus::Expired);
    }

    #[test]
    fn test_get_status_no_expiration() {
        let manager = ExpirationManager::new();
        let credential = create_test_credential(None);

        let status = manager.get_status(&credential);
        assert_eq!(status, ExpirationStatus::NoExpiration);
    }

    #[test]
    fn test_get_days_remaining() {
        let manager = ExpirationManager::new();
        let future = Utc::now() + Duration::days(30);
        let credential = create_test_credential(Some(future));

        let days = manager.get_days_remaining(&credential);
        assert!(days.is_some());
        assert_eq!(days.unwrap(), 30);
    }

    #[test]
    fn test_get_days_remaining_none() {
        let manager = ExpirationManager::new();
        let credential = create_test_credential(None);

        let days = manager.get_days_remaining(&credential);
        assert!(days.is_none());
    }

    #[test]
    fn test_will_expire_in() {
        let manager = ExpirationManager::new();
        let future = Utc::now() + Duration::days(15);
        let credential = create_test_credential(Some(future));

        assert!(manager.will_expire_in(&credential, 30));
        assert!(!manager.will_expire_in(&credential, 10));
    }

    #[test]
    fn test_get_expiration_percentage() {
        let manager = ExpirationManager::new();
        let future = Utc::now() + Duration::days(365);
        let credential = create_test_credential(Some(future));

        let percentage = manager.get_expiration_percentage(&credential);
        assert!(percentage.is_some());
        // Should be close to 0% since just created
        assert!(percentage.unwrap() < 1.0);
    }

    #[test]
    fn test_can_renew() {
        let manager = ExpirationManager::new();
        let past = Utc::now() - Duration::days(30);
        let credential = create_test_credential(Some(past));

        assert!(manager.can_renew(&credential));
    }

    #[test]
    fn test_can_renew_revoked() {
        let manager = ExpirationManager::new();
        let mut credential = create_test_credential(Some(Utc::now() + Duration::days(30)));
        credential.status = CredentialStatus::Revoked.as_str().to_string();

        assert!(!manager.can_renew(&credential));
    }

    #[test]
    fn test_expiration_info_creation() {
        let future = Utc::now() + Duration::days(30);
        let credential = create_test_credential(Some(future));

        let info = ExpirationInfo::from_credential(&credential);
        assert!(!info.is_expired);
        assert!(info.time_remaining.is_some());
        assert!(info.days_remaining.is_some());
    }

    #[test]
    fn test_set_warning_threshold() {
        let mut manager = ExpirationManager::new();
        let result = manager.set_warning_threshold(60);

        assert!(result.is_ok());
        assert_eq!(manager.get_warning_threshold(), 60);
    }

    #[test]
    fn test_validate_expiration_date_valid() {
        let future = Utc::now() + Duration::days(30);
        let result = validate_expiration_date(future);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_expiration_date_past_fails() {
        let past = Utc::now() - Duration::days(1);
        let result = validate_expiration_date(past);
        assert!(result.is_err());
    }
}
