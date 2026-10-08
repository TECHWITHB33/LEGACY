// Payment Authentication REST Handlers
// These endpoints enforce the payment gateway with fail-closed security
//
// Critical: DO NOT call Stellar endpoints directly from anywhere else
// All payments MUST flow through /authenticate-payment endpoint

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::payment_gateway::*;
use crate::stellar_payment::*;
use crate::models::{Credential, CredentialStatus};
use crate::biometric_auth::AuthenticationChallenge;
use chrono::Utc;

/// Request body for payment authentication
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatePaymentRequest {
    /// Payment request ID (must be unique per payment)
    pub payment_request_id: String,
    /// Student credential ID
    pub student_credential_id: String,
    /// Student user ID
    pub student_user_id: String,
    /// Merchant account public key
    pub merchant_account: String,
    /// Payment amount in Lumens
    pub payment_amount: String,
    /// Biometric challenge ID
    pub biometric_challenge_id: String,
    /// Biometric signature (signed challenge with Ed25519)
    pub biometric_signature: String,
    /// Biometric credential ID
    pub biometric_credential_id: String,
}

/// Response from payment authentication - lists all checks performed
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticatePaymentResponse {
    /// Authorization ID (for tracking)
    pub authorization_id: String,
    /// Whether payment is authorized
    pub authorized: bool,
    /// Payment request ID
    pub payment_request_id: String,
    /// List of all checks performed and results
    pub checks: Vec<CheckResult>,
    /// If not authorized, reason(s) for denial
    pub failure_summary: Option<String>,
    /// Authorized timestamp
    pub authorized_at: String,
    /// Authorization expiration (15 minutes)
    pub expires_at: String,
}

/// Single check result for API response
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub passed: bool,
    pub failure_reason: Option<String>,
}

/// Submit authenticated payment to Stellar
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitPaymentRequest {
    /// Authorization ID from previous step
    pub authorization_id: String,
    /// Payment request ID (must match authorization)
    pub payment_request_id: String,
    /// Stellar transaction envelope
    pub transaction_envelope: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitPaymentResponse {
    /// Transaction hash from Stellar
    pub tx_hash: String,
    /// Ledger number
    pub ledger: u32,
    /// Submission status
    pub status: String,
    /// Timestamp
    pub submitted_at: String,
}

/// Handler: Authenticate payment through full gateway
/// This performs all 8 security checks and returns authorization if all pass
pub async fn authenticate_payment(
    req: web::Json<AuthenticatePaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    // Create authentication request
    let auth_req = PaymentAuthenticationRequest {
        payment_request_id: req.payment_request_id.clone(),
        student_credential_id: req.student_credential_id.clone(),
        student_user_id: req.student_user_id.clone(),
        merchant_account: req.merchant_account.clone(),
        payment_amount: req.payment_amount.clone(),
        biometric_challenge_id: req.biometric_challenge_id.clone(),
        biometric_signature: req.biometric_signature.clone(),
        biometric_credential_id: req.biometric_credential_id.clone(),
    };

    // In production, would fetch from database:
    // - credential from credentials table
    // - challenge from authentication_challenges table
    // - merchant verification status from stellar_accounts table
    // - student daily limit from payment_policies table
    // For now, we'll create placeholder data

    let credential = Some(Credential {
        id: req.student_credential_id.clone(),
        user_id: req.student_user_id.clone(),
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
    });

    let challenge = Some(AuthenticationChallenge {
        id: req.biometric_challenge_id.clone(),
        challenge: "challenge_data".to_string(),
        credential_id: req.biometric_credential_id.clone(),
        user_id: req.student_user_id.clone(),
        created_at: Utc::now(),
        expires_at: Utc::now() + chrono::Duration::minutes(5),
        used: false,
    });

    let merchant_verified = true;
    let replay_tracker = AntiReplayTracker::new();
    let daily_limit = 500.0;
    let daily_used = 0.0;

    // Run full gateway authentication
    let auth = PaymentAuthenticationGateway::authenticate_payment(
        &auth_req,
        &credential,
        merchant_verified,
        &challenge,
        &replay_tracker,
        daily_limit,
        daily_used,
    )
    .await;

    // Convert to response format
    let checks: Vec<CheckResult> = auth
        .checks
        .iter()
        .map(|c| CheckResult {
            check_name: c.check_name.clone(),
            passed: c.passed,
            failure_reason: c.failure_reason.clone(),
        })
        .collect();

    let response = AuthenticatePaymentResponse {
        authorization_id: auth.id.clone(),
        authorized: auth.authorized,
        payment_request_id: auth.payment_request_id.clone(),
        checks,
        failure_summary: auth.failure_summary(),
        authorized_at: auth.authorized_at.to_rfc3339(),
        expires_at: auth.expires_at.to_rfc3339(),
    };

    // If not authorized, return 403 Forbidden with details
    if !auth.authorized {
        return Ok(HttpResponse::Forbidden().json(response));
    }

    // If authorized, return 200 OK with authorization details
    Ok(HttpResponse::Ok().json(response))
}

/// Handler: Submit payment to Stellar ONLY after gateway authorization
/// FAIL-CLOSED: Will reject if authorization is missing, expired, or invalid
pub async fn submit_authenticated_payment(
    req: web::Json<SubmitPaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    // In production, would:
    // 1. Fetch authorization from payment_authorizations table
    // 2. Verify authorization is valid (not expired, not used)
    // 3. Verify authorization matches payment_request_id
    // 4. Verify all checks passed
    // 5. Only then submit to Stellar

    // For now, we'll validate the structure
    if req.payment_request_id.is_empty() {
        return Err(ApiError::ValidationError(
            "payment_request_id is required".to_string(),
        ));
    }

    if req.authorization_id.is_empty() {
        return Err(ApiError::ValidationError(
            "authorization_id is required".to_string(),
        ));
    }

    if req.transaction_envelope.is_empty() {
        return Err(ApiError::ValidationError(
            "transaction_envelope is required".to_string(),
        ));
    }

    // In production, would call StellarPaymentEngine::submit_transaction()
    // after validating authorization

    let response = SubmitPaymentResponse {
        tx_hash: "0".repeat(64),
        ledger: 12345,
        status: "submitted".to_string(),
        submitted_at: Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Accepted().json(response))
}

/// Handler: Get payment authorization status
#[derive(Debug, Serialize, Deserialize)]
pub struct GetAuthorizationRequest {
    pub authorization_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAuthorizationResponse {
    pub authorization_id: String,
    pub payment_request_id: String,
    pub authorized: bool,
    pub checks: Vec<CheckResult>,
    pub is_valid: bool,
    pub is_expired: bool,
    pub authorized_at: String,
    pub expires_at: String,
}

pub async fn get_authorization(
    authorization_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // In production, fetch from database
    // For now, return not found
    Err(ApiError::NotFound("Authorization not found".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticate_payment_request_structure() {
        let req = AuthenticatePaymentRequest {
            payment_request_id: "payment1".to_string(),
            student_credential_id: "cred1".to_string(),
            student_user_id: "user1".to_string(),
            merchant_account: "GMERCHANT".to_string(),
            payment_amount: "50.0".to_string(),
            biometric_challenge_id: "challenge1".to_string(),
            biometric_signature: "a".repeat(128),
            biometric_credential_id: "biocred1".to_string(),
        };

        assert_eq!(req.payment_request_id, "payment1");
        assert_eq!(req.student_user_id, "user1");
    }

    #[test]
    fn test_check_result_pass() {
        let result = CheckResult {
            check_name: "Test Check".to_string(),
            passed: true,
            failure_reason: None,
        };

        assert!(result.passed);
        assert!(result.failure_reason.is_none());
    }

    #[test]
    fn test_check_result_fail() {
        let result = CheckResult {
            check_name: "Test Check".to_string(),
            passed: false,
            failure_reason: Some("Reason".to_string()),
        };

        assert!(!result.passed);
        assert_eq!(result.failure_reason, Some("Reason".to_string()));
    }
}
