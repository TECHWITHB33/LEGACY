use crate::error::ApiError;
use crate::models::{RegisterRequest, LoginRequest};
use crate::jwt::JwtClaims;
use sqlx::SqlitePool;
use std::sync::Arc;
use crate::config::Config;

pub struct AuthenticationService {
    pool: Arc<SqlitePool>,
    config: Arc<Config>,
}

impl AuthenticationService {
    pub fn new(pool: Arc<SqlitePool>, config: Arc<Config>) -> Self {
        AuthenticationService { pool, config }
    }

    /// Register a new user
    pub async fn register(&self, req: &RegisterRequest) -> Result<String, ApiError> {
        // Validate email format
        if !req.email.contains('@') || !req.email.contains('.') {
            return Err(ApiError::ValidationError("Invalid email format".to_string()));
        }

        // Check if user already exists
        let existing = sqlx::query!("SELECT id FROM users WHERE email = ?")
            .bind(&req.email)
            .fetch_optional(self.pool.as_ref())
            .await?;

        if existing.is_some() {
            return Err(ApiError::UserAlreadyExists);
        }

        // Hash password
        let (password_hash, password_salt) = crate::crypto::hash_password(&req.password)?;

        // Generate RSA keypair
        let rsa_keypair = crate::crypto::generate_rsa_keypair()?;

        // Generate user ID
        let user_id = format!(
            "STU-{}-{:05}",
            chrono::Utc::now().format("%Y"),
            rand::random::<u16>()
        );

        // Store user in database
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
            rsa_keypair.private_key_pem,
        )
        .execute(self.pool.as_ref())
        .await?;

        // Log audit event
        crate::audit::log_event_no_user(
            &self.pool,
            crate::audit::AuditEvent::StudentCreated,
            "user",
            Some(&user_id),
            Some(&format!("Student {} registered", req.email)),
        )
        .await
        .ok();

        Ok(user_id)
    }

    /// Authenticate a user and return JWT token
    pub async fn login(&self, req: &LoginRequest) -> Result<(String, String), ApiError> {
        // Look up user
        let user = sqlx::query!(
            "SELECT id, password_hash, status FROM users WHERE email = ?",
            req.email
        )
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or(ApiError::InvalidCredentials)?;

        // Check status
        if user.status != "active" {
            return Err(ApiError::AuthenticationError(
                "User account is not active".to_string(),
            ));
        }

        // Verify password
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
            // Log failed login
            crate::audit::log_event_no_user(
                &self.pool,
                crate::audit::AuditEvent::LoginFailure,
                "user",
                None,
                Some(&format!("Failed login attempt for {}", req.email)),
            )
            .await
            .ok();

            return Err(ApiError::InvalidCredentials);
        }

        // Generate JWT token
        let token = crate::jwt::generate_token(&user.id, &req.email, &self.config)?;

        // Log successful login
        crate::audit::log_event(
            &self.pool,
            &user.id,
            crate::audit::AuditEvent::LoginSuccess,
            "user",
            Some(&user.id),
            Some("Successful login"),
        )
        .await
        .ok();

        Ok((user.id, token))
    }

    /// Validate and extract claims from JWT token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, ApiError> {
        crate::jwt::validate_token(token, &self.config)
    }
}
