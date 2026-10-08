// Credential status management and lifecycle operations
// Handles status transitions, validation, and state checking

use crate::error::ApiError;
use crate::models::CredentialStatus;
use chrono::{DateTime, Utc, Duration};

/// Represents credential lifecycle state and transitions
#[derive(Debug, Clone)]
pub struct CredentialLifecycle {
    pub status: CredentialStatus,
    pub created_at: DateTime<Utc>,
    pub signed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl CredentialLifecycle {
    /// Create a new credential lifecycle starting in ACTIVE status
    pub fn new(created_at: DateTime<Utc>) -> Self {
        CredentialLifecycle {
            status: CredentialStatus::Active,
            created_at,
            signed_at: None,
            expires_at: None,
            revoked_at: None,
        }
    }

    /// Mark credential as signed
    pub fn sign(&mut self) -> Result<(), ApiError> {
        if self.signed_at.is_some() {
            return Err(ApiError::ValidationError(
                "Credential already signed".to_string(),
            ));
        }

        self.signed_at = Some(Utc::now());
        Ok(())
    }

    /// Set credential expiration time
    pub fn set_expiration(&mut self, expires_at: DateTime<Utc>) -> Result<(), ApiError> {
        if self.status == CredentialStatus::Revoked {
            return Err(ApiError::ValidationError(
                "Cannot set expiration on revoked credential".to_string(),
            ));
        }

        if expires_at <= Utc::now() {
            return Err(ApiError::ValidationError(
                "Expiration date must be in the future".to_string(),
            ));
        }

        self.expires_at = Some(expires_at);
        Ok(())
    }

    /// Revoke the credential
    pub fn revoke(&mut self) -> Result<(), ApiError> {
        match self.status {
            CredentialStatus::Revoked => {
                return Err(ApiError::ValidationError(
                    "Credential is already revoked".to_string(),
                ));
            }
            CredentialStatus::Expired => {
                return Err(ApiError::ValidationError(
                    "Cannot revoke an expired credential".to_string(),
                ));
            }
            _ => {}
        }

        self.status = CredentialStatus::Revoked;
        self.revoked_at = Some(Utc::now());
        Ok(())
    }

    /// Suspend the credential
    pub fn suspend(&mut self) -> Result<(), ApiError> {
        match self.status {
            CredentialStatus::Revoked => {
                return Err(ApiError::ValidationError(
                    "Cannot suspend a revoked credential".to_string(),
                ));
            }
            CredentialStatus::Expired => {
                return Err(ApiError::ValidationError(
                    "Cannot suspend an expired credential".to_string(),
                ));
            }
            CredentialStatus::Suspended => {
                return Err(ApiError::ValidationError(
                    "Credential is already suspended".to_string(),
                ));
            }
            _ => {}
        }

        self.status = CredentialStatus::Suspended;
        Ok(())
    }

    /// Resume a suspended credential
    pub fn resume(&mut self) -> Result<(), ApiError> {
        if self.status != CredentialStatus::Suspended {
            return Err(ApiError::ValidationError(
                "Credential is not suspended".to_string(),
            ));
        }

        self.status = CredentialStatus::Active;
        Ok(())
    }

    /// Check if credential is currently active (not expired, revoked, or suspended)
    pub fn is_active(&self) -> bool {
        if self.status != CredentialStatus::Active {
            return false;
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            if Utc::now() >= expires_at {
                return false;
            }
        }

        true
    }

    /// Check if credential is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return Utc::now() >= expires_at;
        }
        false
    }

    /// Check if credential is revoked
    pub fn is_revoked(&self) -> bool {
        self.status == CredentialStatus::Revoked
    }

    /// Check if credential is suspended
    pub fn is_suspended(&self) -> bool {
        self.status == CredentialStatus::Suspended
    }

    /// Get current status accounting for expiration
    pub fn current_status(&self) -> CredentialStatus {
        if self.status == CredentialStatus::Revoked {
            return CredentialStatus::Revoked;
        }

        if self.status == CredentialStatus::Suspended {
            return CredentialStatus::Suspended;
        }

        if self.is_expired() {
            return CredentialStatus::Expired;
        }

        CredentialStatus::Active
    }

    /// Get time remaining until expiration
    pub fn time_until_expiration(&self) -> Option<Duration> {
        self.expires_at.map(|expires| {
            let now = Utc::now();
            if now >= expires {
                Duration::seconds(0)
            } else {
                expires - now
            }
        })
    }

    /// Check if credential is valid for use
    pub fn is_valid(&self) -> bool {
        self.is_active() && !self.is_revoked() && !self.is_suspended() && !self.is_expired()
    }
}

/// Validate status transition
pub fn validate_status_transition(
    current_status: &CredentialStatus,
    new_status: &CredentialStatus,
) -> Result<(), ApiError> {
    match (current_status, new_status) {
        // Valid transitions
        (CredentialStatus::Active, CredentialStatus::Revoked) => Ok(()),
        (CredentialStatus::Active, CredentialStatus::Suspended) => Ok(()),
        (CredentialStatus::Active, CredentialStatus::Active) => Ok(()),
        (CredentialStatus::Suspended, CredentialStatus::Active) => Ok(()),
        (CredentialStatus::Suspended, CredentialStatus::Revoked) => Ok(()),
        (CredentialStatus::Suspended, CredentialStatus::Suspended) => Ok(()),
        // Cannot transition from Revoked
        (CredentialStatus::Revoked, _) => {
            Err(ApiError::ValidationError(
                "Cannot transition from Revoked status".to_string(),
            ))
        }
        // Cannot transition to Expired (must be automatic)
        (_, CredentialStatus::Expired) => {
            Err(ApiError::ValidationError(
                "Cannot manually transition to Expired status".to_string(),
            ))
        }
        // Other transitions
        _ => Err(ApiError::ValidationError(
            "Invalid status transition".to_string(),
        )),
    }
}

/// Parse status string to enum
pub fn parse_status(status_str: &str) -> Result<CredentialStatus, ApiError> {
    CredentialStatus::from_str(status_str)
        .map_err(|e| ApiError::ValidationError(format!("Invalid status: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle_creation() {
        let now = Utc::now();
        let lifecycle = CredentialLifecycle::new(now);

        assert_eq!(lifecycle.status, CredentialStatus::Active);
        assert_eq!(lifecycle.created_at, now);
        assert!(lifecycle.signed_at.is_none());
        assert!(lifecycle.expires_at.is_none());
        assert!(lifecycle.revoked_at.is_none());
    }

    #[test]
    fn test_lifecycle_sign() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        let result = lifecycle.sign();
        assert!(result.is_ok());
        assert!(lifecycle.signed_at.is_some());
    }

    #[test]
    fn test_lifecycle_sign_twice_fails() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        lifecycle.sign().unwrap();
        let result = lifecycle.sign();
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_set_expiration() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);
        let future = now + Duration::days(365);

        let result = lifecycle.set_expiration(future);
        assert!(result.is_ok());
        assert_eq!(lifecycle.expires_at, Some(future));
    }

    #[test]
    fn test_lifecycle_set_expiration_past_fails() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);
        let past = now - Duration::days(1);

        let result = lifecycle.set_expiration(past);
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_revoke() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        let result = lifecycle.revoke();
        assert!(result.is_ok());
        assert_eq!(lifecycle.status, CredentialStatus::Revoked);
        assert!(lifecycle.revoked_at.is_some());
    }

    #[test]
    fn test_lifecycle_revoke_twice_fails() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        lifecycle.revoke().unwrap();
        let result = lifecycle.revoke();
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_suspend() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        let result = lifecycle.suspend();
        assert!(result.is_ok());
        assert_eq!(lifecycle.status, CredentialStatus::Suspended);
    }

    #[test]
    fn test_lifecycle_resume() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        lifecycle.suspend().unwrap();
        let result = lifecycle.resume();
        assert!(result.is_ok());
        assert_eq!(lifecycle.status, CredentialStatus::Active);
    }

    #[test]
    fn test_lifecycle_is_active() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert!(lifecycle.is_active());

        lifecycle.suspend().unwrap();
        assert!(!lifecycle.is_active());

        lifecycle.resume().unwrap();
        assert!(lifecycle.is_active());
    }

    #[test]
    fn test_lifecycle_is_expired() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert!(!lifecycle.is_expired());

        let past = now - Duration::days(1);
        lifecycle.expires_at = Some(past);
        assert!(lifecycle.is_expired());
    }

    #[test]
    fn test_lifecycle_is_revoked() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert!(!lifecycle.is_revoked());

        lifecycle.revoke().unwrap();
        assert!(lifecycle.is_revoked());
    }

    #[test]
    fn test_lifecycle_current_status() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert_eq!(lifecycle.current_status(), CredentialStatus::Active);

        lifecycle.suspend().unwrap();
        assert_eq!(lifecycle.current_status(), CredentialStatus::Suspended);

        lifecycle.resume().unwrap();
        lifecycle.revoke().unwrap();
        assert_eq!(lifecycle.current_status(), CredentialStatus::Revoked);
    }

    #[test]
    fn test_lifecycle_time_until_expiration() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert!(lifecycle.time_until_expiration().is_none());

        let future = now + Duration::days(7);
        lifecycle.set_expiration(future).unwrap();

        let time_left = lifecycle.time_until_expiration();
        assert!(time_left.is_some());
        let duration = time_left.unwrap();
        assert!(duration.num_days() >= 6 && duration.num_days() <= 7);
    }

    #[test]
    fn test_validate_status_transition_valid() {
        let result = validate_status_transition(&CredentialStatus::Active, &CredentialStatus::Revoked);
        assert!(result.is_ok());

        let result = validate_status_transition(&CredentialStatus::Active, &CredentialStatus::Suspended);
        assert!(result.is_ok());

        let result = validate_status_transition(&CredentialStatus::Suspended, &CredentialStatus::Active);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_status_transition_from_revoked_fails() {
        let result = validate_status_transition(&CredentialStatus::Revoked, &CredentialStatus::Active);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_status_transition_to_expired_fails() {
        let result = validate_status_transition(&CredentialStatus::Active, &CredentialStatus::Expired);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_status_valid() {
        let result = parse_status("active");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CredentialStatus::Active);

        let result = parse_status("revoked");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CredentialStatus::Revoked);
    }

    #[test]
    fn test_parse_status_invalid() {
        let result = parse_status("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_lifecycle_is_valid() {
        let now = Utc::now();
        let mut lifecycle = CredentialLifecycle::new(now);

        assert!(lifecycle.is_valid());

        lifecycle.suspend().unwrap();
        assert!(!lifecycle.is_valid());

        lifecycle.resume().unwrap();
        assert!(lifecycle.is_valid());

        lifecycle.revoke().unwrap();
        assert!(!lifecycle.is_valid());
    }
}
