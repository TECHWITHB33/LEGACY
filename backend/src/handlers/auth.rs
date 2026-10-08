use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::models::*;
use crate::error::ApiError;
use crate::config::Config;

pub async fn register(
    pool: web::Data<Arc<SqlitePool>>,
    config: web::Data<Arc<Config>>,
    req: web::Json<RegisterRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("User registration request: {}", req.email);
    
    // 1. Validate email format
    if !req.email.contains('@') || !req.email.contains('.') {
        return Err(ApiError::ValidationError("Invalid email format".to_string()));
    }

    // 2. Check if user already exists
    let existing_user = sqlx::query!("SELECT id FROM users WHERE email = ?")
        .bind(&req.email)
        .fetch_optional(pool.as_ref().as_ref())
        .await?;

    if existing_user.is_some() {
        return Err(ApiError::UserAlreadyExists);
    }

    // 3. Hash password using PBKDF2
    let (password_hash, password_salt) = crate::crypto::hash_password(&req.password)?;

    // 4. Generate RSA 4096 keypair for user
    let rsa_keypair = crate::crypto::generate_rsa_keypair()?;

    // 5. Generate unique user ID
    let user_id = format!("STU-{}-{:05}", 
        chrono::Utc::now().format("%Y"),
        rand::random::<u16>()
    );

    // FIX VULN-1: Encrypt private key with AES-256-GCM
    let encryption_key = std::env::var("CREDENTIAL_ENCRYPTION_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("CREDENTIAL_ENCRYPTION_KEY not set, using default (DEVELOPMENT ONLY)");
            "development_key_must_be_changed_in_production_123456".to_string()
        });

    let encrypted_private_key = crate::crypto::encrypt_private_key(
        &rsa_keypair.private_key_pem,
        &encryption_key,
    )?;

    // 6. Insert user into database with ENCRYPTED private key
    sqlx::query!(
        r#"INSERT INTO users 
           (id, email, name, institution, matric_number, age, gender, department, phone_contact,
            password_hash, public_key, private_key_encrypted, status, created_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
        user_id,
        req.email,
        req.name,
        req.institution,
        req.matric_number,
        req.age,
        req.gender,
        req.department,
        req.phone_contact,
        format!("{}:{}", password_hash, password_salt),
        rsa_keypair.public_key_pem,
        encrypted_private_key,  // ✅ NOW ENCRYPTED
    )
    .execute(pool.as_ref().as_ref())
    .await?;

    // 7. Generate JWT token
    let token = crate::jwt::generate_token(&user_id, &req.email, &config)?;

    // 8. Log audit event
    tracing::info!("User registered successfully: {}", user_id);
    sqlx::query!(
        "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp) 
         VALUES (?, ?, 'STUDENT_CREATED', 'user', ?, ?, CURRENT_TIMESTAMP)",
        format!("AUD-{}", uuid::Uuid::new_v4()),
        user_id,
        user_id,
        format!("Student {} registered", req.email),
    )
    .execute(pool.as_ref().as_ref())
    .await?;

    // 9. Generate initial credential ID
    let credential_id = format!("CRED-{}-{:05}", 
        chrono::Utc::now().format("%Y"),
        rand::random::<u16>()
    );

    Ok(HttpResponse::Created().json(RegisterResponse {
        user_id,
        public_key: rsa_keypair.public_key_pem,
        credential_id,
        created_at: chrono::Utc::now(),
    }))
}

pub async fn login(
    pool: web::Data<Arc<SqlitePool>>,
    config: web::Data<Arc<Config>>,
    req: web::Json<LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("User login request: {}", req.email);
    
    // 1. Look up user by email
    let user = sqlx::query!(
        "SELECT id, password_hash, status FROM users WHERE email = ?",
        req.email
    )
    .fetch_optional(pool.as_ref().as_ref())
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    // 2. Check user status
    if user.status != "active" {
        tracing::warn!("Login attempt for non-active user: {}", req.email);
        return Err(ApiError::AuthenticationError("User account is not active".to_string()));
    }

    // 3. Verify password against stored hash
    let password_parts: Vec<&str> = user.password_hash.split(':').collect();
    if password_parts.len() != 2 {
        return Err(ApiError::InternalError);
    }

    let is_valid = crate::crypto::verify_password(
        &req.password,
        password_parts[0],
        password_parts[1],
    )?;

    if !is_valid {
        tracing::warn!("Invalid password for user: {}", req.email);
        sqlx::query!(
            "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp) 
             VALUES (?, ?, 'LOGIN_FAILURE', 'user', ?, ?, CURRENT_TIMESTAMP)",
            format!("AUD-{}", uuid::Uuid::new_v4()),
            user.id,
            user.id,
            "Invalid password",
        )
        .execute(pool.as_ref().as_ref())
        .await?;
        
        return Err(ApiError::InvalidCredentials);
    }

    // 4. Generate JWT token
    let token = crate::jwt::generate_token(&user.id, &req.email, &config)?;

    // 5. Calculate expiry
    let expiry = config.jwt.expiry;

    // 6. Log successful login
    tracing::info!("User logged in successfully: {}", user.id);
    sqlx::query!(
        "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp) 
         VALUES (?, ?, 'LOGIN_SUCCESS', 'user', ?, ?, CURRENT_TIMESTAMP)",
        format!("AUD-{}", uuid::Uuid::new_v4()),
        user.id,
        user.id,
        "Successful login",
    )
    .execute(pool.as_ref().as_ref())
    .await?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        user_id: user.id,
        token,
        refresh_token: "not_implemented".to_string(),
        expires_in: expiry as u64,
    }))
}

pub async fn verify_signature(
    pool: web::Data<Arc<SqlitePool>>,
    req: web::Json<VerifySignatureRequest>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Signature verification request");
    
    // FIX VULN-3: Implement proper signature verification
    
    // 1. Validate signature format (128 hex chars = 64 bytes for Ed25519)
    if req.signature.len() != 128 || !req.signature.chars().all(|c| c.is_ascii_hexdigit()) {
        tracing::warn!("Invalid signature format");
        return Err(ApiError::ValidationError(
            "Signature must be 128 hex characters".to_string()
        ));
    }

    // 2. Validate message is not empty
    if req.message.is_empty() {
        return Err(ApiError::ValidationError(
            "Message cannot be empty".to_string()
        ));
    }

    // 3. Validate credential_id format
    if req.credential_id.is_empty() {
        return Err(ApiError::ValidationError(
            "Credential ID cannot be empty".to_string()
        ));
    }

    // 4. Look up credential and public key
    let pool_ref = pool.get_ref().as_ref();
    
    let credential = sqlx::query!(
        "SELECT public_key_hash FROM credentials WHERE id = ?",
        req.credential_id
    )
    .fetch_optional(pool_ref)
    .await?
    .ok_or_else(|| {
        tracing::warn!("Credential not found: {}", req.credential_id);
        ApiError::CredentialNotFound
    })?;

    // 5. Decode signature and public key
    let signature_bytes = hex::decode(&req.signature)
        .map_err(|_| {
            tracing::warn!("Invalid signature hex encoding");
            ApiError::ValidationError("Invalid signature encoding".to_string())
        })?;

    let public_key_bytes = hex::decode(&credential.public_key_hash)
        .map_err(|_| {
            tracing::warn!("Invalid public key hex encoding");
            ApiError::ValidationError("Invalid public key format".to_string())
        })?;

    // 6. Verify signature using Ed25519
    let message_bytes = req.message.as_bytes();
    
    let signature_valid = crate::crypto::verify_ed25519_signature(
        message_bytes,
        &signature_bytes,
        &public_key_bytes,
    ).unwrap_or(false);

    if !signature_valid {
        tracing::warn!("Signature verification failed for credential: {}", req.credential_id);
        return Err(ApiError::AuthenticationError(
            "Signature verification failed".to_string()
        ));
    }

    // 7. Log successful verification
    tracing::info!("Signature verified successfully for credential: {}", req.credential_id);
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "valid": true,
        "message": "Signature verified successfully",
        "credential_id": req.credential_id,
        "verified_at": chrono::Utc::now()
    })))
}

pub async fn refresh_token(
    pool: web::Data<Arc<SqlitePool>>,
    config: web::Data<Arc<Config>>,
    req: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    tracing::info!("Token refresh request");
    
    // FIX VULN-4: Implement token refresh with proper validation
    
    // 1. Extract refresh token from request
    let refresh_token = req.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::ValidationError(
            "refresh_token is required".to_string()
        ))?;

    // 2. Validate refresh token (similar validation as access token)
    let claims = crate::jwt::validate_token(refresh_token, &config)
        .map_err(|_| ApiError::AuthenticationError(
            "Invalid or expired refresh token".to_string()
        ))?;

    // 3. Verify user still exists and is active
    let pool_ref = pool.get_ref().as_ref();
    
    let user = sqlx::query!(
        "SELECT status FROM users WHERE id = ?",
        claims.sub
    )
    .fetch_optional(pool_ref)
    .await?
    .ok_or(ApiError::InvalidCredentials)?;

    if user.status != "active" {
        tracing::warn!("Refresh token attempted for non-active user: {}", claims.sub);
        return Err(ApiError::AuthenticationError(
            "User account is not active".to_string()
        ));
    }

    // 4. Generate new access token
    let new_token = crate::jwt::generate_token(&claims.sub, &claims.email, &config)?;

    // 5. Log token refresh
    sqlx::query!(
        "INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, details, timestamp) 
         VALUES (?, ?, 'TOKEN_REFRESH', 'token', ?, ?, CURRENT_TIMESTAMP)",
        format!("AUD-{}", uuid::Uuid::new_v4()),
        claims.sub,
        claims.sub,
        "Token refreshed successfully",
    )
    .execute(pool_ref)
    .await
    .ok();

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "token": new_token,
        "token_type": "Bearer",
        "expires_in": config.jwt.expiry,
        "refreshed_at": chrono::Utc::now()
    })))
}
