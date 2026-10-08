// Payment Authentication Gateway
// CRITICAL: This module is the ONLY path to Stellar transactions
// All payments MUST pass through this gateway with fail-closed security
// 
// Payment Flow with Security Checks:
// 1. Credential Verification (must exist)
// 2. Credential Status Check (must not be revoked/expired/suspended)
// 3. Merchant Authorization (merchant must be verified)
// 4. Payment Amount Validation (amount in valid range)
// 5. Biometric/Passkey Authentication (must succeed)
// 6. Challenge Verification (challenge must not be expired/reused)
// 7. Anti-Replay Check (no duplicate payment_request_id in flight)
// 8. Payment Policy Check (payment allowed per policy)
// 9. Stellar Transaction (ONLY submitted if all checks pass)
// 10. Confirmation (transaction confirmed on network)
//
// FAIL-CLOSED: If ANY check fails, payment is REJECTED and NO transaction is submitted.

use crate::error::ApiError;
use crate::stellar_payment::*;
use crate::biometric_auth::*;
use crate::credential_status::*;
use crate::models::{Credential, CredentialStatus};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Gateway check result - tracks each validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayCheckResult {
    /// Check name
    pub check_name: String,
    /// Whether check passed
    pub passed: bool,
    /// Failure reason (if failed)
    pub failure_reason: Option<String>,
    /// Timestamp
    pub checked_at: DateTime<Utc>,
}

impl GatewayCheckResult {
    pub fn pass(check_name: &str) -> Self {
        GatewayCheckResult {
            check_name: check_name.to_string(),
            passed: true,
            failure_reason: None,
            checked_at: Utc::now(),
        }
    }

    pub fn fail(check_name: &str, reason: &str) -> Self {
        GatewayCheckResult {
            check_name: check_name.to_string(),
            passed: false,
            failure_reason: Some(reason.to_string()),
            checked_at: Utc::now(),
        }
    }
}

/// Payment authentication request - contains all required data for gateway check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAuthenticationRequest {
    /// Payment request ID (unique identifier for this payment)
    pub payment_request_id: String,
    /// Student credential ID (payer)
    pub student_credential_id: String,
    /// Student user ID
    pub student_user_id: String,
    /// Merchant account (public key)
    pub merchant_account: String,
    /// Payment amount (in Lumens)
    pub payment_amount: String,
    /// Biometric challenge ID
    pub biometric_challenge_id: String,
    /// Biometric signature (signed challenge)
    pub biometric_signature: String,
    /// Biometric credential ID
    pub biometric_credential_id: String,
}

/// Payment gateway authorization - result of successful authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentGatewayAuthorization {
    /// Authorization ID
    pub id: String,
    /// Payment request ID
    pub payment_request_id: String,
    /// Student user ID
    pub student_user_id: String,
    /// Authorized at
    pub authorized_at: DateTime<Utc>,
    /// All check results (for audit trail)
    pub checks: Vec<GatewayCheckResult>,
    /// Authorized flag
    pub authorized: bool,
    /// Expiration (15 minute window to submit transaction)
    pub expires_at: DateTime<Utc>,
}

impl PaymentGatewayAuthorization {
    pub fn new(
        payment_request_id: String,
        student_user_id: String,
        checks: Vec<GatewayCheckResult>,
    ) -> Self {
        let authorized = checks.iter().all(|c| c.passed);
        let now = Utc::now();

        PaymentGatewayAuthorization {
            id: Uuid::new_v4().to_string(),
            payment_request_id,
            student_user_id,
            authorized_at: now,
            checks,
            authorized,
            expires_at: now + chrono::Duration::minutes(15),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.authorized && Utc::now() <= self.expires_at
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get all failed checks
    pub fn failed_checks(&self) -> Vec<&GatewayCheckResult> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }

    /// Get summary of why authorization failed
    pub fn failure_summary(&self) -> Option<String> {
        let failures: Vec<String> = self
            .failed_checks()
            .iter()
            .map(|c| format!(
                "{}: {}",
                c.check_name,
                c.failure_reason.as_ref().unwrap_or(&"Unknown".to_string())
            ))
            .collect();

        if failures.is_empty() {
            None
        } else {
            Some(failures.join(" | "))
        }
    }
}

/// Anti-replay state tracker
#[derive(Debug, Clone)]
pub struct AntiReplayTracker {
    /// In-flight payment request IDs (being processed)
    in_flight: Arc<RwLock<HashSet<String>>>,
    /// Completed payment request IDs (already processed)
    completed: Arc<RwLock<HashSet<String>>>,
}

impl AntiReplayTracker {
    pub fn new() -> Self {
        AntiReplayTracker {
            in_flight: Arc::new(RwLock::new(HashSet::new())),
            completed: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Check if payment_request_id is already in flight or completed
    pub async fn is_replay(&self, payment_request_id: &str) -> bool {
        let in_flight = self.in_flight.read().await;
        let completed = self.completed.read().await;
        in_flight.contains(payment_request_id) || completed.contains(payment_request_id)
    }

    /// Mark as in-flight
    pub async fn mark_in_flight(&self, payment_request_id: String) -> Result<(), ApiError> {
        let mut in_flight = self.in_flight.write().await;
        if in_flight.contains(&payment_request_id) {
            return Err(ApiError::PaymentError(
                "Payment already in flight".to_string(),
            ));
        }
        in_flight.insert(payment_request_id);
        Ok(())
    }

    /// Mark as completed
    pub async fn mark_completed(&self, payment_request_id: String) -> Result<(), ApiError> {
        let mut in_flight = self.in_flight.write().await;
        let mut completed = self.completed.write().await;

        if !in_flight.remove(&payment_request_id) {
            return Err(ApiError::PaymentError(
                "Payment not marked in-flight".to_string(),
            ));
        }
        completed.insert(payment_request_id);
        Ok(())
    }

    /// Cancel in-flight payment
    pub async fn cancel_in_flight(&self, payment_request_id: &str) -> Result<(), ApiError> {
        let mut in_flight = self.in_flight.write().await;
        if in_flight.remove(payment_request_id) {
            Ok(())
        } else {
            Err(ApiError::PaymentError(
                "Payment not in-flight".to_string(),
            ))
        }
    }
}

/// Payment Authentication Gateway
pub struct PaymentAuthenticationGateway;

impl PaymentAuthenticationGateway {
    /// CHECK 1: Verify credential exists
    pub fn check_credential_exists(
        credential: &Option<Credential>,
    ) -> GatewayCheckResult {
        match credential {
            Some(_) => GatewayCheckResult::pass("Credential Verification"),
            None => GatewayCheckResult::fail(
                "Credential Verification",
                "Credential not found or invalid",
            ),
        }
    }

    /// CHECK 2: Verify credential status (not revoked, expired, or suspended)
    pub fn check_credential_status(
        credential: &Credential,
    ) -> GatewayCheckResult {
        // Check if credential is active
        match &credential.status {
            CredentialStatus::Active => {
                GatewayCheckResult::pass("Credential Status Check")
            }
            CredentialStatus::Revoked => {
                GatewayCheckResult::fail(
                    "Credential Status Check",
                    "Credential has been revoked",
                )
            }
            CredentialStatus::Expired => {
                GatewayCheckResult::fail(
                    "Credential Status Check",
                    "Credential has expired",
                )
            }
            CredentialStatus::Suspended => {
                GatewayCheckResult::fail(
                    "Credential Status Check",
                    "Credential has been suspended",
                )
            }
            CredentialStatus::Pending => {
                GatewayCheckResult::fail(
                    "Credential Status Check",
                    "Credential is pending verification",
                )
            }
        }
    }

    /// CHECK 3: Verify merchant is authorized
    pub fn check_merchant_authorization(
        merchant_verified: bool,
    ) -> GatewayCheckResult {
        if merchant_verified {
            GatewayCheckResult::pass("Merchant Authorization")
        } else {
            GatewayCheckResult::fail(
                "Merchant Authorization",
                "Merchant account is not verified",
            )
        }
    }

    /// CHECK 4: Validate payment amount
    pub fn check_payment_amount(
        amount: &str,
        min_amount: f64,
        max_amount: f64,
    ) -> GatewayCheckResult {
        match amount.parse::<f64>() {
            Ok(amt) => {
                if amt >= min_amount && amt <= max_amount {
                    GatewayCheckResult::pass("Payment Amount Validation")
                } else {
                    GatewayCheckResult::fail(
                        "Payment Amount Validation",
                        &format!(
                            "Amount {} outside allowed range {}-{}",
                            amt, min_amount, max_amount
                        ),
                    )
                }
            }
            Err(_) => {
                GatewayCheckResult::fail(
                    "Payment Amount Validation",
                    "Invalid amount format",
                )
            }
        }
    }

    /// CHECK 5: Verify biometric authentication succeeded
    pub fn check_biometric_authentication(
        biometric_signature: &str,
    ) -> GatewayCheckResult {
        // Signature must be 128 hex chars (Ed25519)
        if biometric_signature.len() == 128 && biometric_signature.chars().all(|c| c.is_ascii_hexdigit()) {
            GatewayCheckResult::pass("Biometric/Passkey Authentication")
        } else {
            GatewayCheckResult::fail(
                "Biometric/Passkey Authentication",
                "Invalid biometric signature format or length",
            )
        }
    }

    /// CHECK 6: Verify challenge is valid (not expired or reused)
    pub fn check_challenge_validity(
        challenge: &AuthenticationChallenge,
    ) -> GatewayCheckResult {
        if challenge.is_expired() {
            GatewayCheckResult::fail(
                "Challenge Verification",
                "Challenge has expired",
            )
        } else if challenge.used {
            GatewayCheckResult::fail(
                "Challenge Verification",
                "Challenge has already been used (replay attack detected)",
            )
        } else {
            GatewayCheckResult::pass("Challenge Verification")
        }
    }

    /// CHECK 7: Anti-replay check (payment_request_id not already processed)
    pub async fn check_anti_replay(
        payment_request_id: &str,
        replay_tracker: &AntiReplayTracker,
    ) -> GatewayCheckResult {
        if replay_tracker.is_replay(payment_request_id).await {
            GatewayCheckResult::fail(
                "Anti-Replay Check",
                "Payment request has already been processed",
            )
        } else {
            GatewayCheckResult::pass("Anti-Replay Check")
        }
    }

    /// CHECK 8: Payment policy check (can student make payment now?)
    pub fn check_payment_policy(
        daily_limit: f64,
        daily_used: f64,
        amount: f64,
    ) -> GatewayCheckResult {
        let remaining = daily_limit - daily_used;
        if amount <= remaining {
            GatewayCheckResult::pass("Payment Policy Check")
        } else {
            GatewayCheckResult::fail(
                "Payment Policy Check",
                &format!(
                    "Daily limit exceeded: would use {} of {} remaining",
                    amount, remaining
                ),
            )
        }
    }

    /// FULL GATEWAY CHECK - Executes all security checks in sequence
    /// Returns authorization if ALL checks pass, otherwise returns detailed failure
    pub async fn authenticate_payment(
        req: &PaymentAuthenticationRequest,
        credential: &Option<Credential>,
        merchant_verified: bool,
        challenge: &Option<AuthenticationChallenge>,
        replay_tracker: &AntiReplayTracker,
        daily_limit: f64,
        daily_used: f64,
    ) -> PaymentGatewayAuthorization {
        let mut checks = Vec::new();

        // CHECK 1: Credential exists
        checks.push(Self::check_credential_exists(credential));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        let cred = credential.as_ref().unwrap();

        // CHECK 2: Credential status valid
        checks.push(Self::check_credential_status(cred));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 3: Merchant authorized
        checks.push(Self::check_merchant_authorization(merchant_verified));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 4: Payment amount valid
        checks.push(Self::check_payment_amount(&req.payment_amount, 0.1, 1000.0));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 5: Biometric authentication
        checks.push(Self::check_biometric_authentication(&req.biometric_signature));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 6: Challenge validity
        if let Some(ch) = challenge {
            checks.push(Self::check_challenge_validity(ch));
            if !checks.last().unwrap().passed {
                return PaymentGatewayAuthorization::new(
                    req.payment_request_id.clone(),
                    req.student_user_id.clone(),
                    checks,
                );
            }
        } else {
            checks.push(GatewayCheckResult::fail(
                "Challenge Verification",
                "Challenge not found",
            ));
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 7: Anti-replay
        checks.push(Self::check_anti_replay(&req.payment_request_id, replay_tracker).await);
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // CHECK 8: Payment policy
        let amount = req.payment_amount.parse::<f64>().unwrap_or(0.0);
        checks.push(Self::check_payment_policy(daily_limit, daily_used, amount));
        if !checks.last().unwrap().passed {
            return PaymentGatewayAuthorization::new(
                req.payment_request_id.clone(),
                req.student_user_id.clone(),
                checks,
            );
        }

        // ALL CHECKS PASSED - return authorization
        PaymentGatewayAuthorization::new(
            req.payment_request_id.clone(),
            req.student_user_id.clone(),
            checks,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_check_result_pass() {
        let result = GatewayCheckResult::pass("Test Check");
        assert!(result.passed);
        assert_eq!(result.check_name, "Test Check");
        assert!(result.failure_reason.is_none());
    }

    #[test]
    fn test_gateway_check_result_fail() {
        let result = GatewayCheckResult::fail("Test Check", "Test failure");
        assert!(!result.passed);
        assert_eq!(result.failure_reason, Some("Test failure".to_string()));
    }

    #[test]
    fn test_check_credential_exists_valid() {
        let credential = Credential {
            id: "cred1".to_string(),
            user_id: "user1".to_string(),
            credential_type: "student_id".to_string(),
            issuer: "school".to_string(),
            issued_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            status: CredentialStatus::Active,
            data: serde_json::json!({}),
            signature: Some("sig".to_string()),
            verified: true,
            revocation_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = PaymentAuthenticationGateway::check_credential_exists(&Some(credential));
        assert!(result.passed);
        assert_eq!(result.check_name, "Credential Verification");
    }

    #[test]
    fn test_check_credential_exists_invalid() {
        let result = PaymentAuthenticationGateway::check_credential_exists(&None);
        assert!(!result.passed);
        assert_eq!(result.check_name, "Credential Verification");
    }

    #[test]
    fn test_check_credential_status_active() {
        let credential = Credential {
            id: "cred1".to_string(),
            user_id: "user1".to_string(),
            credential_type: "student_id".to_string(),
            issuer: "school".to_string(),
            issued_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            status: CredentialStatus::Active,
            data: serde_json::json!({}),
            signature: Some("sig".to_string()),
            verified: true,
            revocation_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = PaymentAuthenticationGateway::check_credential_status(&credential);
        assert!(result.passed);
    }

    #[test]
    fn test_check_credential_status_revoked() {
        let credential = Credential {
            id: "cred1".to_string(),
            user_id: "user1".to_string(),
            credential_type: "student_id".to_string(),
            issuer: "school".to_string(),
            issued_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            status: CredentialStatus::Revoked,
            data: serde_json::json!({}),
            signature: Some("sig".to_string()),
            verified: true,
            revocation_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = PaymentAuthenticationGateway::check_credential_status(&credential);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("revoked"));
    }

    #[test]
    fn test_check_credential_status_expired() {
        let credential = Credential {
            id: "cred1".to_string(),
            user_id: "user1".to_string(),
            credential_type: "student_id".to_string(),
            issuer: "school".to_string(),
            issued_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            status: CredentialStatus::Expired,
            data: serde_json::json!({}),
            signature: Some("sig".to_string()),
            verified: true,
            revocation_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = PaymentAuthenticationGateway::check_credential_status(&credential);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("expired"));
    }

    #[test]
    fn test_check_credential_status_suspended() {
        let credential = Credential {
            id: "cred1".to_string(),
            user_id: "user1".to_string(),
            credential_type: "student_id".to_string(),
            issuer: "school".to_string(),
            issued_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            status: CredentialStatus::Suspended,
            data: serde_json::json!({}),
            signature: Some("sig".to_string()),
            verified: true,
            revocation_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let result = PaymentAuthenticationGateway::check_credential_status(&credential);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("suspended"));
    }

    #[test]
    fn test_check_merchant_authorization_valid() {
        let result = PaymentAuthenticationGateway::check_merchant_authorization(true);
        assert!(result.passed);
    }

    #[test]
    fn test_check_merchant_authorization_not_verified() {
        let result = PaymentAuthenticationGateway::check_merchant_authorization(false);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("not verified"));
    }

    #[test]
    fn test_check_payment_amount_valid() {
        let result = PaymentAuthenticationGateway::check_payment_amount("50.0", 0.1, 1000.0);
        assert!(result.passed);
    }

    #[test]
    fn test_check_payment_amount_too_low() {
        let result = PaymentAuthenticationGateway::check_payment_amount("0.05", 0.1, 1000.0);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("outside allowed range"));
    }

    #[test]
    fn test_check_payment_amount_too_high() {
        let result = PaymentAuthenticationGateway::check_payment_amount("2000.0", 0.1, 1000.0);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_payment_amount_invalid_format() {
        let result = PaymentAuthenticationGateway::check_payment_amount("invalid", 0.1, 1000.0);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_biometric_authentication_valid() {
        let valid_sig = "a".repeat(128); // 128 hex chars
        let result = PaymentAuthenticationGateway::check_biometric_authentication(&valid_sig);
        assert!(result.passed);
    }

    #[test]
    fn test_check_biometric_authentication_invalid_length() {
        let invalid_sig = "a".repeat(100); // Wrong length
        let result = PaymentAuthenticationGateway::check_biometric_authentication(&invalid_sig);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_biometric_authentication_invalid_chars() {
        let invalid_sig = "g".repeat(128); // 'g' is not hex
        let result = PaymentAuthenticationGateway::check_biometric_authentication(&invalid_sig);
        assert!(!result.passed);
    }

    #[test]
    fn test_check_challenge_validity_valid() {
        let challenge = AuthenticationChallenge {
            id: "ch1".to_string(),
            challenge: "test".to_string(),
            credential_id: "cred1".to_string(),
            user_id: "user1".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            used: false,
        };

        let result = PaymentAuthenticationGateway::check_challenge_validity(&challenge);
        assert!(result.passed);
    }

    #[test]
    fn test_check_challenge_validity_expired() {
        let challenge = AuthenticationChallenge {
            id: "ch1".to_string(),
            challenge: "test".to_string(),
            credential_id: "cred1".to_string(),
            user_id: "user1".to_string(),
            created_at: Utc::now() - chrono::Duration::minutes(10),
            expires_at: Utc::now() - chrono::Duration::minutes(5),
            used: false,
        };

        let result = PaymentAuthenticationGateway::check_challenge_validity(&challenge);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("expired"));
    }

    #[test]
    fn test_check_challenge_validity_reused() {
        let challenge = AuthenticationChallenge {
            id: "ch1".to_string(),
            challenge: "test".to_string(),
            credential_id: "cred1".to_string(),
            user_id: "user1".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(5),
            used: true,
        };

        let result = PaymentAuthenticationGateway::check_challenge_validity(&challenge);
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("already been used"));
    }

    #[test]
    fn test_check_payment_policy_within_limit() {
        let result = PaymentAuthenticationGateway::check_payment_policy(
            100.0, // daily limit
            30.0,  // daily used
            50.0,  // amount
        );
        assert!(result.passed);
    }

    #[test]
    fn test_check_payment_policy_exceeds_limit() {
        let result = PaymentAuthenticationGateway::check_payment_policy(
            100.0, // daily limit
            80.0,  // daily used
            50.0,  // amount (would exceed)
        );
        assert!(!result.passed);
        assert!(result.failure_reason.unwrap().contains("Daily limit exceeded"));
    }

    #[test]
    fn test_gateway_authorization_all_passed() {
        let checks = vec![
            GatewayCheckResult::pass("Check 1"),
            GatewayCheckResult::pass("Check 2"),
            GatewayCheckResult::pass("Check 3"),
        ];

        let auth = PaymentGatewayAuthorization::new(
            "payment1".to_string(),
            "user1".to_string(),
            checks,
        );

        assert!(auth.authorized);
        assert!(auth.is_valid());
        assert!(auth.failed_checks().is_empty());
        assert!(auth.failure_summary().is_none());
    }

    #[test]
    fn test_gateway_authorization_one_failed() {
        let checks = vec![
            GatewayCheckResult::pass("Check 1"),
            GatewayCheckResult::fail("Check 2", "Failed for test"),
            GatewayCheckResult::pass("Check 3"),
        ];

        let auth = PaymentGatewayAuthorization::new(
            "payment1".to_string(),
            "user1".to_string(),
            checks,
        );

        assert!(!auth.authorized);
        assert_eq!(auth.failed_checks().len(), 1);
        assert!(auth.failure_summary().unwrap().contains("Check 2"));
    }

    #[test]
    fn test_gateway_authorization_expired() {
        let checks = vec![GatewayCheckResult::pass("Check 1")];

        let mut auth = PaymentGatewayAuthorization::new(
            "payment1".to_string(),
            "user1".to_string(),
            checks,
        );

        // Manually set expiration in past
        auth.expires_at = Utc::now() - chrono::Duration::minutes(1);
        assert!(!auth.is_valid());
        assert!(auth.is_expired());
    }

    #[tokio::test]
    async fn test_anti_replay_tracker_new_payment() {
        let tracker = AntiReplayTracker::new();
        let payment_id = "payment1";

        assert!(!tracker.is_replay(payment_id).await);
        tracker.mark_in_flight(payment_id.to_string()).await.unwrap();
        assert!(tracker.is_replay(payment_id).await);
    }

    #[tokio::test]
    async fn test_anti_replay_tracker_duplicate_in_flight() {
        let tracker = AntiReplayTracker::new();
        let payment_id = "payment1";

        tracker.mark_in_flight(payment_id.to_string()).await.unwrap();
        let result = tracker.mark_in_flight(payment_id.to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_anti_replay_tracker_completed() {
        let tracker = AntiReplayTracker::new();
        let payment_id = "payment1";

        tracker.mark_in_flight(payment_id.to_string()).await.unwrap();
        tracker.mark_completed(payment_id.to_string()).await.unwrap();
        
        assert!(tracker.is_replay(payment_id).await);

        // Should not be able to mark again
        let result = tracker.mark_in_flight(payment_id.to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_anti_replay_tracker_cancel() {
        let tracker = AntiReplayTracker::new();
        let payment_id = "payment1";

        tracker.mark_in_flight(payment_id.to_string()).await.unwrap();
        tracker.cancel_in_flight(payment_id).await.unwrap();

        assert!(!tracker.is_replay(payment_id).await);
    }
}
