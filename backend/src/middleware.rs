//! Middleware for request processing

use crate::error::ApiError;
use actix_web::HttpRequest;

/// Extract user ID from JWT token in request header
pub fn extract_user_id(req: &HttpRequest) -> Result<String, ApiError> {
    // Try to get Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!("Missing Authorization header");
            ApiError::AuthenticationError("Missing Authorization header".to_string())
        })?;

    // Extract Bearer token
    if !auth_header.starts_with("Bearer ") {
        tracing::warn!("Invalid Authorization header format");
        return Err(ApiError::AuthenticationError(
            "Invalid Authorization header format".to_string(),
        ));
    }

    let _token = &auth_header[7..];
    
    // For now, extract a dummy user ID from a custom header
    // In production, decode JWT and extract sub claim
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("default-user")
        .to_string();

    Ok(user_id)
}
