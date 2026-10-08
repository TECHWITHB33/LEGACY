use actix_web::{web, HttpResponse, HttpRequest};
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::error::ApiError;
use crate::models::Credential;
use crate::qr_verification::{SecureQrData, QrVerificationEngine};
use crate::services::QrService;
use crate::middleware::extract_user_id;

/// Request to generate QR code
#[derive(serde::Deserialize)]
pub struct GenerateQrRequest {
    pub credential_id: String,
    pub include_verification_url: Option<bool>,
    pub base_verification_url: Option<String>,
}

/// Response with QR code image
#[derive(serde::Serialize)]
pub struct GenerateQrResponse {
    pub credential_id: String,
    pub qr_code_svg: String,
    pub qr_data: String,
    pub generated_at: String,
}

/// Request to verify QR code
#[derive(serde::Deserialize)]
pub struct VerifyQrRequest {
    pub qr_data: String,
}

/// Response with verification result
#[derive(serde::Serialize)]
pub struct VerifyQrResponse {
    pub status: String,
    pub credential_id: Option<String>,
    pub message: String,
    pub credential_info: Option<serde_json::Value>,
    pub verified_at: String,
}

/// Generate QR code for credential
/// POST /api/v1/qr/generate
pub async fn generate_qr(
    req: HttpRequest,
    body: web::Json<GenerateQrRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user ID from JWT
    let _user_id = extract_user_id(&req)?;

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
    .bind(&body.credential_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch credential: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?
    .ok_or_else(|| ApiError::CredentialNotFound)?;

    // Create QR verification engine
    let engine = QrVerificationEngine::new(
        "ID Card System".to_string(),
        credential.public_key_hash.clone(),
    );

    // Add base URL if provided
    let engine = if let Some(url) = &body.base_verification_url {
        engine.with_base_url(url.clone())
    } else if body.include_verification_url.unwrap_or(false) {
        engine.with_base_url("https://api.idcardsystem.local/api/v1/qr".to_string())
    } else {
        engine
    };

    // Generate secure QR data (non-sensitive information only)
    let qr_data = engine.generate_qr_data(credential.id.clone())?;

    // Convert to JSON
    let qr_json = qr_data.to_json()
        .map_err(|e| ApiError::QrError(format!("Failed to serialize QR data: {}", e)))?;

    // Generate QR code image using QrService
    let qr_service = QrService::new();
    let qr_svg = qr_service.generate(&qr_json)
        .map_err(|e| ApiError::QrError(format!("Failed to generate QR code: {:?}", e)))?;

    let qr_svg_string = String::from_utf8(qr_svg)
        .map_err(|_| ApiError::QrError("Failed to convert QR to string".to_string()))?;

    // Log QR generation
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&credential.user_id)
    .bind("generate_qr")
    .bind("credential")
    .bind(&credential.id)
    .bind(json!({
        "qr_generated": true,
        "include_url": body.include_verification_url,
    }).to_string())
    .bind(chrono::Utc::now())
    .execute(pool_ref)
    .await
    .ok();

    let response = GenerateQrResponse {
        credential_id: credential.id,
        qr_code_svg: qr_svg_string,
        qr_data: qr_json,
        generated_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Verify QR code and credential
/// POST /api/v1/qr/verify
pub async fn verify_qr(
    body: web::Json<VerifyQrRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Parse QR data
    let qr_data = SecureQrData::from_json(&body.qr_data)
        .map_err(|e| ApiError::QrError(format!("Invalid QR data format: {}", e)))?;

    // Fetch credential from database
    let pool_ref = pool.get_ref().as_ref();

    let credential: Option<Credential> = sqlx::query_as(
        r#"
        SELECT id, user_id, public_key_hash, qr_code, qr_hash, status,
               version, schema_hash, credential_hash, signature,
               signed_at, expires_at, revoked_at, created_at, updated_at
        FROM credentials
        WHERE id = ?
        "#,
    )
    .bind(&qr_data.credential_id)
    .fetch_optional(pool_ref)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch credential: {:?}", e);
        ApiError::DatabaseError(e.to_string())
    })?;

    // Create verification engine
    let engine = QrVerificationEngine::new(
        "ID Card System".to_string(),
        // Use issuer public key from credential if available
        qr_data.credential_id.clone() + "hash",
    );

    // Verify
    let verification_result = if let Some(cred) = credential {
        engine.verify_credential(&cred, &qr_data)?
    } else {
        use crate::qr_verification::QrVerificationResult;
        use crate::qr_verification::QrVerificationStatus;
        
        QrVerificationResult {
            status: QrVerificationStatus::NotFound,
            credential_id: Some(qr_data.credential_id.clone()),
            message: "Credential not found".to_string(),
            credential_data: None,
            verified_at: chrono::Utc::now().to_rfc3339(),
        }
    };

    // Log verification
    sqlx::query(
        r#"
        INSERT INTO audit_logs (
            id, user_id, action, resource_type, resource_id,
            details, timestamp
        ) VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&qr_data.credential_id)
    .bind("verify_qr")
    .bind("credential")
    .bind(&qr_data.credential_id)
    .bind(json!({
        "verification_status": verification_result.status.as_str(),
    }).to_string())
    .bind(chrono::Utc::now())
    .execute(pool_ref)
    .await
    .ok();

    // Build response
    let response = VerifyQrResponse {
        status: verification_result.status.as_str().to_string(),
        credential_id: verification_result.credential_id,
        message: verification_result.message,
        credential_info: verification_result.credential_data.map(|data| {
            serde_json::to_value(data).unwrap_or_else(|_| json!({}))
        }),
        verified_at: verification_result.verified_at,
    };

    Ok(HttpResponse::Ok().json(response))
}
