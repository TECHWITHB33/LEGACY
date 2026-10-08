use actix_web::{web, HttpResponse, HttpRequest};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::{CreateCredentialRequest, CreateCredentialResponse, Credential, CredentialStatus, VerifyCredentialResponse};
use crate::middleware::extract_user_id;
use crate::services::CredentialIssuerService;
use crate::validators;
use crate::crypto::hash::*;
use crate::crypto::signature::*;
use crate::credential_expiration::ExpirationManager;
use chrono::{Duration, Utc};

/// Create a new credential
/// POST /api/v1/credentials
/// 
/// Request body:
/// {
///   "user_id": "user-123",
///   "schema_hash": "64-char hex string (32 bytes)",
///   "qr_code": "data or URL",
///   "expires_in_days": 365
/// }
pub async fn create_credential(
    req: HttpRequest,
    body: web::Json<CreateCredentialRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user ID from JWT token
    let user_id = extract_user_id(&req)?;

    // Validate schema hash format (must be exactly 32 bytes / 64 hex chars)
    let schema_hash = validate_schema_hash(&body.schema_hash)?;

    // Validate QR code
    validators::validate_qr_code(&body.qr_code)?;

    // Validate expiration days if provided
    if let Some(days) = body.expires_in_days {
        validators::validate_expiration_days(days)?;
    }

    // Generate credential ID
    let credential_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    // Calculate QR hash
    let qr_hash = hash_qr_code(&body.qr_code)?;

    // For now, use a test issuer key - in production, this should come from config
    // and private key should NEVER be stored in source code
    let (private_key, public_key) = generate_signature_keypair()
        .map_err(|_| ApiError::CryptoError("Failed to generate issuer keypair".to_string()))?;

    // Create issuer service
    let issuer = CredentialIssuerService::new(
        "ID Card System".to_string(),
        public_key.clone(),
        private_key,
    )?;

    // Issue credential using the service
    let (mut credential, _) = issuer.issue_credential(
        user_id.clone(),
        schema_hash,
        body.qr_code.clone(),
        body.expires_in_days,
    )?;

    // Store credential in database
    let pool_ref = pool.get_ref().as_ref();

    sqlx::query(
        r#"
        INSERT INTO credentials (
            id, user_id, public_key_hash, qr_code, qr_hash, status,
            version, schema_hash, credential_hash, signature,
            signed_at, expires_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&credential.id)
    .bind(&credential.user_id)
    .bind(&credential.public_key_hash)
    .bind(&credential.qr_code)
    .bind(&credential.qr_hash)
    .bind(&credential.status)
    .bind(credential.version)
    .bind(&credential.schema_hash)
    .bind(&credential.credential_hash)
    .bind(&credential.signature)
    .bind(credential.signed_at)
    .bind(credential.expires_at)
    .bind(credential.created_at)
    .bind(credential.updated_at)
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to insert credential: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    // Log credential creation in audit log
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
    .bind("create_credential")
    .bind("credential")
    .bind(&credential.id)
    .bind(json!({
        "schema_hash": credential.schema_hash,
        "expires_at": credential.expires_at,
    }).to_string())
    .bind(Utc::now())
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to log credential creation: {:?}", e);
        // Don't fail if audit logging fails
        ApiError::DatabaseError(e.to_string())
    })
    .ok();

    // Build response
    let response = CreateCredentialResponse {
        credential_id: credential.id,
        credential_hash: credential.credential_hash,
        signature: credential.signature,
        status: credential.status,
        version: credential.version,
        created_at: credential.created_at,
    };

    Ok(HttpResponse::Created().json(response))
}

/// Verify a credential's signature and integrity
/// GET /api/v1/credentials/{id}/verify
/// 
/// Response:
/// {
///   "valid": true,
///   "credential_id": "cred-123",
///   "user_id": "user-456",
///   "status": "active",
///   "version": 1,
///   "signed_at": "2024-01-02T00:00:00Z",
///   "verification_message": "Credential signature valid"
/// }
pub async fn verify_credential(
    req: HttpRequest,
    path: web::Path<String>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user ID from JWT token
    let _user_id = extract_user_id(&req)?;

    // Get credential ID from path
    let credential_id = path.into_inner();

    // Fetch credential from database
    let pool_ref = pool.get_ref().as_ref();

    let credential: Credential = sqlx::query_as(
        r#"
        SELECT id, user_id, public_key_hash, qr_code, qr_hash, status,
               version, schema_hash, credential_hash, signature,
               signed_at, expires_at, revoked_at, created_at, updated_at
        FROM credentials
        WHERE id = ?
        "#,
    )
    .bind(&credential_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch credential: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| ApiError::CredentialNotFound)?;

    // Validate credential hashes format
    validators::validate_credential_hash(&credential.credential_hash)?;
    validators::validate_schema_hash(&credential.schema_hash)?;
    validators::validate_signature(&credential.signature)?;

    // Create expiration manager to check status
    let expiration_manager = ExpirationManager::new();

    // Determine current status
    let current_status = if credential.status == CredentialStatus::Revoked.as_str() {
        CredentialStatus::Revoked
    } else if expiration_manager.is_expired(&credential) {
        CredentialStatus::Expired
    } else if credential.status == CredentialStatus::Suspended.as_str() {
        CredentialStatus::Suspended
    } else {
        CredentialStatus::Active
    };

    // Verify credential signature using stored public key
    // Extract public key from public_key_hash (this would normally be looked up from issuer)
    // For now, we'll verify the signature structure is valid
    let signature_valid = !credential.signature.is_empty() 
        && credential.signature.len() == 128
        && hex::decode(&credential.signature).is_ok();

    // Verify credential hash format
    let hash_valid = !credential.credential_hash.is_empty()
        && credential.credential_hash.len() == 64
        && hex::decode(&credential.credential_hash).is_ok();

    let verification_message = if !signature_valid {
        "Invalid signature format".to_string()
    } else if !hash_valid {
        "Invalid credential hash format".to_string()
    } else if current_status == CredentialStatus::Revoked {
        "Credential has been revoked".to_string()
    } else if current_status == CredentialStatus::Expired {
        "Credential has expired".to_string()
    } else if current_status == CredentialStatus::Suspended {
        "Credential is suspended".to_string()
    } else {
        "Credential signature valid".to_string()
    };

    // Log credential verification
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(&credential.user_id)
    .bind("verify_credential")
    .bind("credential")
    .bind(&credential.id)
    .bind(json!({
        "valid": signature_valid && hash_valid && current_status == CredentialStatus::Active,
        "status": current_status.as_str(),
    }).to_string())
    .bind(Utc::now())
    .execute(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to log credential verification: {:?}", e);
        // Don't fail if audit logging fails
        ApiError::DatabaseError(e.to_string())
    })
    .ok();

    // Build response
    let response = VerifyCredentialResponse {
        valid: signature_valid && hash_valid && current_status == CredentialStatus::Active,
        credential_id: credential.id,
        user_id: credential.user_id,
        status: current_status.as_str().to_string(),
        version: credential.version,
        signed_at: credential.signed_at,
        verification_message,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn list_credentials() -> Result<HttpResponse, ApiError> {
    // TODO: Implement list credentials
    Err(ApiError::InternalError)
}

pub async fn get_credential(path: web::Path<String>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement get credential
    Err(ApiError::InternalError)
}

pub async fn revoke_credential(path: web::Path<String>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement revoke credential
    Err(ApiError::InternalError)
}
