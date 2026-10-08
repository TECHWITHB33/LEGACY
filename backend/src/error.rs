use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use serde::Serialize;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Authorization failed: {0}")]
    AuthorizationError(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Credential not found")]
    CredentialNotFound,

    #[error("Invalid schema hash: expected 32 bytes")]
    InvalidSchemaHash,

    #[error("User not found")]
    UserNotFound,

    #[error("User already exists")]
    UserAlreadyExists,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("QR code error: {0}")]
    QrError(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Payment error: {0}")]
    PaymentError(String),

    #[error("Stellar error: {0}")]
    StellarError(String),

    #[error("WebAuthn error: {0}")]
    WebAuthnError(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Expired")]
    Expired,

    #[error("Revoked")]
    Revoked,

    #[error("Internal server error")]
    InternalError,

    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status: u16,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let (status_code, message) = match self {
            ApiError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            ApiError::AuthenticationError(_) => (StatusCode::UNAUTHORIZED, "Authentication failed"),
            ApiError::AuthorizationError(_) => (StatusCode::FORBIDDEN, "Not authorized"),
            ApiError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials"),
            ApiError::CredentialNotFound => (StatusCode::NOT_FOUND, "Credential not found"),
            ApiError::InvalidSchemaHash => (StatusCode::BAD_REQUEST, "Invalid schema hash"),
            ApiError::UserNotFound => (StatusCode::NOT_FOUND, "User not found"),
            ApiError::UserAlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            ApiError::InvalidInput(_) => (StatusCode::BAD_REQUEST, "Invalid input"),
            ApiError::ValidationError(_) => (StatusCode::BAD_REQUEST, "Validation failed"),
            ApiError::CryptoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Cryptographic error"),
            ApiError::QrError(_) => (StatusCode::BAD_REQUEST, "QR code error"),
            ApiError::BlockchainError(_) => (StatusCode::BAD_REQUEST, "Blockchain error"),
            ApiError::PaymentError(_) => (StatusCode::BAD_REQUEST, "Payment error"),
            ApiError::StellarError(_) => (StatusCode::BAD_REQUEST, "Stellar network error"),
            ApiError::WebAuthnError(_) => (StatusCode::BAD_REQUEST, "WebAuthn error"),
            ApiError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "Rate limited"),
            ApiError::Expired => (StatusCode::UNAUTHORIZED, "Expired"),
            ApiError::Revoked => (StatusCode::UNAUTHORIZED, "Revoked"),
            ApiError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
            ApiError::ExternalServiceError(_) => (StatusCode::BAD_GATEWAY, "External service error"),
        };

        let error_response = ErrorResponse {
            error: self.to_string(),
            message: message.to_string(),
            status: status_code.as_u16(),
        };

        HttpResponse::build(status_code).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            ApiError::AuthorizationError(_) => StatusCode::FORBIDDEN,
            ApiError::InvalidCredentials => StatusCode::UNAUTHORIZED,
            ApiError::CredentialNotFound => StatusCode::NOT_FOUND,
            ApiError::InvalidSchemaHash => StatusCode::BAD_REQUEST,
            ApiError::UserNotFound => StatusCode::NOT_FOUND,
            ApiError::UserAlreadyExists => StatusCode::CONFLICT,
            ApiError::InvalidInput(_) => StatusCode::BAD_REQUEST,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            ApiError::CryptoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::QrError(_) => StatusCode::BAD_REQUEST,
            ApiError::BlockchainError(_) => StatusCode::BAD_REQUEST,
            ApiError::PaymentError(_) => StatusCode::BAD_REQUEST,
            ApiError::StellarError(_) => StatusCode::BAD_REQUEST,
            ApiError::WebAuthnError(_) => StatusCode::BAD_REQUEST,
            ApiError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            ApiError::Expired => StatusCode::UNAUTHORIZED,
            ApiError::Revoked => StatusCode::UNAUTHORIZED,
            ApiError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::ExternalServiceError(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ApiError::DatabaseError(err.to_string())
    }
}
