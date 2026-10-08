use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::ApiError;
use crate::config::Config;

/// JWT claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,              // Subject (user_id)
    pub exp: u64,                 // Expiration time (seconds since epoch)
    pub iat: u64,                 // Issued at (seconds since epoch)
    pub email: String,            // User email
}

/// Generate a JWT token
pub fn generate_token(user_id: &str, email: &str, config: &Config) -> Result<String, ApiError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ApiError::InternalError)?
        .as_secs();

    let expiry = now + config.jwt.expiry;

    let claims = JwtClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: expiry,
    };

    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| ApiError::CryptoError(format!("JWT encoding failed: {}", e)))
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, config: &Config) -> Result<JwtClaims, ApiError> {
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    let token_data = jsonwebtoken::decode::<JwtClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &validation,
    )
    .map_err(|e| {
        tracing::warn!("JWT validation failed: {}", e);
        ApiError::AuthenticationError("Invalid or expired token".to_string())
    })?;

    Ok(token_data.claims)
}

/// Extract token from Authorization header
pub fn extract_token_from_header(auth_header: &str) -> Result<String, ApiError> {
    if !auth_header.starts_with("Bearer ") {
        return Err(ApiError::AuthenticationError(
            "Invalid Authorization header format".to_string(),
        ));
    }

    Ok(auth_header[7..].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            server: crate::config::ServerConfig {
                host: "localhost".to_string(),
                port: 8080,
            },
            database: crate::config::DatabaseConfig {
                url: "sqlite::memory:".to_string(),
            },
            jwt: crate::config::JwtConfig {
                secret: "test-secret-at-least-32-characters-long-1234".to_string(),
                expiry: 3600,
            },
            crypto: crate::config::CryptoConfig {
                algorithm: "HS256".to_string(),
            },
            stellar: crate::config::StellarConfig {
                network: "testnet".to_string(),
                horizon_url: "https://horizon-testnet.stellar.org".to_string(),
            },
            features: crate::config::FeaturesConfig {
                enable_biometric: true,
                enable_payments: true,
            },
        }
    }

    #[test]
    fn test_generate_and_validate_token() {
        let config = create_test_config();
        let user_id = "STU-2026-00001";
        let email = "test@example.com";

        // Generate token
        let token = generate_token(user_id, email, &config).expect("Failed to generate token");
        assert!(!token.is_empty());

        // Validate token
        let claims = validate_token(&token, &config).expect("Failed to validate token");
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.email, email);
    }

    #[test]
    fn test_invalid_token_rejected() {
        let config = create_test_config();
        let result = validate_token("invalid.token.here", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_token_from_header() {
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let token = extract_token_from_header(valid_header).expect("Failed to extract token");
        assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
    }

    #[test]
    fn test_invalid_header_format() {
        let invalid_header = "Basic dXNlcjpwYXNz";
        let result = extract_token_from_header(invalid_header);
        assert!(result.is_err());
    }
}
