use actix_web::{web, HttpResponse, HttpRequest};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::error::ApiError;
use crate::biometric_auth::*;
use crate::middleware::extract_user_id;
use chrono::Utc;
use uuid::Uuid;

/// Request for registration challenge
#[derive(serde::Deserialize)]
pub struct RegistrationBeginRequest {
    pub device_id: String,
    pub device_name: String,
}

/// Response with registration challenge
#[derive(serde::Serialize)]
pub struct RegistrationBeginResponse {
    pub challenge_id: String,
    pub challenge: String,
    pub user_id: String,
}

/// Request to complete registration
#[derive(serde::Deserialize)]
pub struct RegistrationCompleteRequest {
    pub challenge_id: String,
    pub public_key: String,
    pub attestation_object: String,
    pub credential_type: String,
    pub device_id: String,
    pub device_name: String,
}

/// Response after registration
#[derive(serde::Serialize)]
pub struct RegistrationCompleteResponse {
    pub success: bool,
    pub credential_id: Option<String>,
    pub message: String,
}

/// Request for authentication challenge
#[derive(serde::Deserialize)]
pub struct AuthenticationBeginRequest {
    pub credential_id: String,
}

/// Response with authentication challenge
#[derive(serde::Serialize)]
pub struct AuthenticationBeginResponse {
    pub challenge_id: String,
    pub challenge: String,
}

/// Request to complete authentication
#[derive(serde::Deserialize)]
pub struct AuthenticationCompleteRequest {
    pub challenge_id: String,
    pub credential_id: String,
    pub signature: String,
    pub sign_count: u32,
}

/// Response after authentication
#[derive(serde::Serialize)]
pub struct AuthenticationCompleteResponse {
    pub success: bool,
    pub result: String,
    pub message: String,
}

/// Begin biometric registration
/// POST /api/v1/biometric/register/begin
/// Generates a challenge for the device to use during biometric enrollment
pub async fn register_begin(
    req: HttpRequest,
    body: web::Json<RegistrationBeginRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(&req)?;

    // Generate registration challenge
    let challenge = RegistrationChallenge::new(user_id.clone());

    // Store challenge in database
    let pool_ref = pool.get_ref().as_ref();
    sqlx::query(
        r#"
        INSERT INTO biometric_challenges (
            id, user_id, challenge, challenge_type, expires_at, used, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&challenge.id)
    .bind(&challenge.user_id)
    .bind(&challenge.challenge)
    .bind("registration")
    .bind(challenge.expires_at)
    .bind(false)
    .bind(challenge.created_at)
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to store registration challenge: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    // Log the operation
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&user_id)
    .bind("biometric_register_begin")
    .bind("biometric_credential")
    .bind(&challenge.id)
    .bind(json!({
        "device_id": body.device_id,
        "device_name": body.device_name,
    }).to_string())
    .bind(Utc::now())
    .execute(pool_ref)
    .await
    .ok();

    Ok(HttpResponse::Ok().json(RegistrationBeginResponse {
        challenge_id: challenge.id,
        challenge: challenge.challenge,
        user_id: challenge.user_id,
    }))
}

/// Complete biometric registration
/// POST /api/v1/biometric/register/complete
/// Device sends public key and attestation after biometric enrollment
pub async fn register_complete(
    req: HttpRequest,
    body: web::Json<RegistrationCompleteRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let user_id = extract_user_id(&req)?;
    let pool_ref = pool.get_ref().as_ref();

    // Verify challenge exists and is valid
    let stored_challenge: Option<(String, bool)> = sqlx::query_as(
        "SELECT challenge, used FROM biometric_challenges WHERE id = ?",
    )
    .bind(&body.challenge_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch challenge: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    let (_, challenge_used) = stored_challenge.ok_or(ApiError::ValidationError(
        "Challenge not found".to_string(),
    ))?;

    if challenge_used {
        return Err(ApiError::ValidationError(
            "Challenge already used".to_string(),
        ));
    }

    // Verify attestation (in production, use full WebAuthn verification)
    let engine = BiometricAuthEngine::new("ID Card System".to_string());
    engine.verify_registration_attestation(&body.attestation_object, &body.public_key)?;

    // Parse credential type
    let credential_type = BiometricType::from_str(&body.credential_type)?;

    // Create credential
    let credential_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + chrono::Duration::days(730); // 2 years

    sqlx::query(
        r#"
        INSERT INTO biometric_credentials (
            id, user_id, public_key, credential_type, device_id, device_name,
            sign_count, status, created_at, expires_at, last_used_at, revoked_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&credential_id)
    .bind(&user_id)
    .bind(&body.public_key)
    .bind(credential_type.as_str())
    .bind(&body.device_id)
    .bind(&body.device_name)
    .bind(0i32)
    .bind("active")
    .bind(now)
    .bind(expires_at)
    .bind::<Option<_>>(None)
    .bind::<Option<_>>(None)
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create biometric credential: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    // Mark challenge as used
    sqlx::query("UPDATE biometric_challenges SET used = 1 WHERE id = ?")
        .bind(&body.challenge_id)
        .execute(pool_ref)
        .await
        .ok();

    // Log the operation
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&user_id)
    .bind("biometric_register_complete")
    .bind("biometric_credential")
    .bind(&credential_id)
    .bind(json!({
        "device_id": body.device_id,
        "device_name": body.device_name,
        "credential_type": body.credential_type,
    }).to_string())
    .bind(Utc::now())
    .execute(pool_ref)
    .await
    .ok();

    Ok(HttpResponse::Ok().json(RegistrationCompleteResponse {
        success: true,
        credential_id: Some(credential_id),
        message: "Biometric credential registered successfully".to_string(),
    }))
}

/// Begin biometric authentication
/// POST /api/v1/biometric/authenticate/begin
/// Generates a challenge that the device will sign with biometric
pub async fn authenticate_begin(
    body: web::Json<AuthenticationBeginRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let pool_ref = pool.get_ref().as_ref();

    // Verify credential exists
    let user_id: String = sqlx::query_scalar("SELECT user_id FROM biometric_credentials WHERE id = ?")
        .bind(&body.credential_id)
        .fetch_optional(pool_ref)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch credential: {:?}", e);
            ApiError::DatabaseError(e.to_string())
        })?
        .ok_or(ApiError::ValidationError(
            "Credential not found".to_string(),
        ))?;

    // Generate authentication challenge
    let challenge = AuthenticationChallenge::new(body.credential_id.clone(), user_id);

    // Store challenge in database
    sqlx::query(
        r#"
        INSERT INTO biometric_challenges (
            id, user_id, challenge, challenge_type, credential_id, expires_at, used, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&challenge.id)
    .bind(&challenge.user_id)
    .bind(&challenge.challenge)
    .bind("authentication")
    .bind(&challenge.credential_id)
    .bind(challenge.expires_at)
    .bind(false)
    .bind(challenge.created_at)
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to store authentication challenge: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    Ok(HttpResponse::Ok().json(AuthenticationBeginResponse {
        challenge_id: challenge.id,
        challenge: challenge.challenge,
    }))
}

/// Complete biometric authentication
/// POST /api/v1/biometric/authenticate/complete
/// Device sends signature after biometric verification
pub async fn authenticate_complete(
    body: web::Json<AuthenticationCompleteRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    let pool_ref = pool.get_ref().as_ref();

    // Fetch and verify challenge
    let challenge_data: Option<(String, bool, String)> = sqlx::query_as(
        "SELECT challenge, used, user_id FROM biometric_challenges WHERE id = ?",
    )
    .bind(&body.challenge_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (challenge, challenge_used, user_id) = challenge_data.ok_or(ApiError::ValidationError(
        "Challenge not found".to_string(),
    ))?;

    // Check if challenge already used (replay protection)
    if challenge_used {
        return Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
            success: false,
            result: AuthenticationResult::ReplayedChallenge.to_string(),
            message: "Challenge has already been used".to_string(),
        }));
    }

    // Fetch credential
    let credential_data: Option<(String, String, i32, String, Option<String>)> = sqlx::query_as(
        "SELECT public_key, status, sign_count, user_id, expires_at FROM biometric_credentials WHERE id = ?",
    )
    .bind(&body.credential_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let (public_key, status, old_sign_count, cred_user_id, expires_at) =
        credential_data.ok_or(ApiError::ValidationError(
            "Credential not found".to_string(),
        ))?;

    // Verify credential status
    let engine = BiometricAuthEngine::new("ID Card System".to_string());

    // Check replay protection (sign_count must increase)
    if !BiometricAuthEngine::check_replay_protection(old_sign_count as u32, body.sign_count)? {
        return Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
            success: false,
            result: AuthenticationResult::ReplayedChallenge.to_string(),
            message: "Sign count did not increase - possible replay attack".to_string(),
        }));
    }

    // Verify signature
    if !engine.verify_signature(&public_key, &challenge, &body.signature)? {
        return Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
            success: false,
            result: AuthenticationResult::InvalidSignature.to_string(),
            message: "Signature verification failed".to_string(),
        }));
    }

    // Check credential status
    if status == "revoked" {
        return Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
            success: false,
            result: AuthenticationResult::RevokedCredential.to_string(),
            message: "Credential has been revoked".to_string(),
        }));
    }

    // Check expiration
    if let Some(exp) = expires_at {
        if let Ok(exp_time) = chrono::DateTime::parse_from_rfc3339(&exp) {
            if Utc::now() > exp_time.with_timezone(&Utc) {
                return Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
                    success: false,
                    result: AuthenticationResult::ExpiredCredential.to_string(),
                    message: "Credential has expired".to_string(),
                }));
            }
        }
    }

    // Update credential: increment sign_count and last_used_at
    sqlx::query(
        "UPDATE biometric_credentials SET sign_count = ?, last_used_at = ? WHERE id = ?",
    )
    .bind(body.sign_count as i32)
    .bind(Utc::now())
    .bind(&body.credential_id)
    .execute(pool_ref)
    .await
    .ok();

    // Mark challenge as used
    sqlx::query("UPDATE biometric_challenges SET used = 1 WHERE id = ?")
        .bind(&body.challenge_id)
        .execute(pool_ref)
        .await
        .ok();

    // Log the operation
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&cred_user_id)
    .bind("biometric_authenticate")
    .bind("biometric_credential")
    .bind(&body.credential_id)
    .bind(json!({
        "result": "success",
        "sign_count": body.sign_count,
    }).to_string())
    .bind(Utc::now())
    .execute(pool_ref)
    .await
    .ok();

    Ok(HttpResponse::Ok().json(AuthenticationCompleteResponse {
        success: true,
        result: AuthenticationResult::Success.to_string(),
        message: "Authentication successful".to_string(),
    }))
}
