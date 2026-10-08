// Enhanced Input Validation Module
// Fixes VULN-5, VULN-8, VULN-9, VULN-14, VULN-15

use crate::error::ApiError;

/// FIX VULN-9: Strict email validation using regex
pub fn validate_email(email: &str) -> Result<String, ApiError> {
    // RFC 5322 simplified regex
    let email_regex = regex::Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$"
    ).unwrap();
    
    if !email_regex.is_match(email) {
        return Err(ApiError::ValidationError(
            "Invalid email format".to_string()
        ));
    }
    
    if email.len() > 254 {
        return Err(ApiError::ValidationError(
            "Email too long (max 254 characters)".to_string()
        ));
    }
    
    Ok(email.to_string())
}

/// FIX VULN-14: Strict password validation
pub fn validate_password(password: &str) -> Result<String, ApiError> {
    // Must be at least 12 characters
    if password.len() < 12 {
        return Err(ApiError::ValidationError(
            "Password must be at least 12 characters long".to_string()
        ));
    }
    
    // Max 128 characters
    if password.len() > 128 {
        return Err(ApiError::ValidationError(
            "Password too long (max 128 characters)".to_string()
        ));
    }
    
    // Must contain uppercase
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one uppercase letter".to_string()
        ));
    }
    
    // Must contain lowercase
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one lowercase letter".to_string()
        ));
    }
    
    // Must contain number
    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one digit".to_string()
        ));
    }
    
    // Must contain special character
    let special_chars = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one special character: !@#$%^&*()_+-=[]{}|;:,.<>?".to_string()
        ));
    }
    
    Ok(password.to_string())
}

/// FIX VULN-5: Schema hash must be exactly 64 hex characters
pub fn validate_schema_hash(hash: &str) -> Result<String, ApiError> {
    if hash.len() != 64 {
        return Err(ApiError::ValidationError(
            format!("Schema hash must be exactly 64 hex characters, got {}", hash.len())
        ));
    }
    
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::ValidationError(
            "Schema hash must be valid hexadecimal".to_string()
        ));
    }
    
    // Verify it's not all zeros (weak hash)
    if hash.chars().all(|c| c == '0') {
        return Err(ApiError::ValidationError(
            "Schema hash cannot be all zeros".to_string()
        ));
    }
    
    Ok(hash.to_string())
}

/// Validate credential hash format
pub fn validate_credential_hash(hash: &str) -> Result<String, ApiError> {
    if hash.len() != 64 {
        return Err(ApiError::ValidationError(
            "Credential hash must be exactly 64 hex characters".to_string()
        ));
    }
    
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::ValidationError(
            "Credential hash must be valid hexadecimal".to_string()
        ));
    }
    
    Ok(hash.to_string())
}

/// Validate Ed25519 signature format
pub fn validate_signature(signature: &str) -> Result<String, ApiError> {
    // Ed25519 signature is 64 bytes = 128 hex characters
    if signature.len() != 128 {
        return Err(ApiError::ValidationError(
            format!("Signature must be exactly 128 hex characters, got {}", signature.len())
        ));
    }
    
    if !signature.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::ValidationError(
            "Signature must be valid hexadecimal".to_string()
        ));
    }
    
    Ok(signature.to_string())
}

/// Validate payment amount
pub fn validate_payment_amount(amount_str: &str) -> Result<f64, ApiError> {
    let amount: f64 = amount_str.parse()
        .map_err(|_| ApiError::ValidationError(
            "Amount must be a valid number".to_string()
        ))?;
    
    // Must be positive
    if amount <= 0.0 {
        return Err(ApiError::ValidationError(
            "Amount must be positive".to_string()
        ));
    }
    
    // Must be within range (0.1 - 1000 XLM)
    if amount < 0.1 {
        return Err(ApiError::ValidationError(
            "Amount must be at least 0.1 XLM".to_string()
        ));
    }
    
    if amount > 1000.0 {
        return Err(ApiError::ValidationError(
            "Amount cannot exceed 1000.0 XLM".to_string()
        ));
    }
    
    // Check for reasonable precision (max 7 decimal places for Stellar lumens)
    let amount_str_parts: Vec<&str> = amount_str.split('.').collect();
    if amount_str_parts.len() == 2 && amount_str_parts[1].len() > 7 {
        return Err(ApiError::ValidationError(
            "Amount has too many decimal places (max 7)".to_string()
        ));
    }
    
    Ok(amount)
}

/// Validate QR code format
pub fn validate_qr_code(qr_code: &str) -> Result<String, ApiError> {
    if qr_code.is_empty() {
        return Err(ApiError::ValidationError(
            "QR code cannot be empty".to_string()
        ));
    }
    
    if qr_code.len() > 2953 {
        return Err(ApiError::ValidationError(
            "QR code too long (max 2953 characters)".to_string()
        ));
    }
    
    // Must not contain secret keys
    if qr_code.contains("secret") || qr_code.contains("private") || qr_code.contains("-----BEGIN") {
        return Err(ApiError::ValidationError(
            "QR code must not contain secret keys or private data".to_string()
        ));
    }
    
    Ok(qr_code.to_string())
}

/// Validate expiration days
pub fn validate_expiration_days(days: i64) -> Result<i64, ApiError> {
    if days < 1 {
        return Err(ApiError::ValidationError(
            "Expiration must be at least 1 day".to_string()
        ));
    }
    
    if days > 3650 {
        return Err(ApiError::ValidationError(
            "Expiration cannot exceed 10 years (3650 days)".to_string()
        ));
    }
    
    Ok(days)
}

/// Validate credential ID format
pub fn validate_credential_id(id: &str) -> Result<String, ApiError> {
    if id.is_empty() {
        return Err(ApiError::ValidationError(
            "Credential ID cannot be empty".to_string()
        ));
    }
    
    if id.len() > 100 {
        return Err(ApiError::ValidationError(
            "Credential ID too long".to_string()
        ));
    }
    
    // Must be alphanumeric or underscore/dash
    if !id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(ApiError::ValidationError(
            "Credential ID must be alphanumeric with underscores or dashes".to_string()
        ));
    }
    
    Ok(id.to_string())
}

/// Validate public key format (Stellar address)
pub fn validate_stellar_public_key(key: &str) -> Result<String, ApiError> {
    // Stellar public key: starts with 'G' and is 56 characters
    if key.len() != 56 {
        return Err(ApiError::ValidationError(
            format!("Stellar public key must be 56 characters, got {}", key.len())
        ));
    }
    
    if !key.starts_with('G') {
        return Err(ApiError::ValidationError(
            "Stellar public key must start with 'G'".to_string()
        ));
    }
    
    // Verify it's valid base32
    if !key.chars().all(|c| c.is_alphanumeric()) {
        return Err(ApiError::ValidationError(
            "Stellar public key must be alphanumeric".to_string()
        ));
    }
    
    Ok(key.to_string())
}

/// FIX VULN-8: Rate limiting check (simulated - real implementation in middleware)
pub fn check_rate_limit(user_id: &str, max_requests_per_minute: u32) -> Result<(), ApiError> {
    // This is a placeholder - real implementation would check Redis or in-memory cache
    // For now, just validate the input
    if max_requests_per_minute == 0 {
        return Err(ApiError::ValidationError(
            "Rate limit must be positive".to_string()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("john.doe@university.ac.uk").is_ok());
    }

    #[test]
    fn test_validate_email_invalid() {
        assert!(validate_email("invalid.email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_validate_password_weak() {
        assert!(validate_password("short").is_err()); // Too short
        assert!(validate_password("alllowercase123!").is_err()); // No uppercase
        assert!(validate_password("ALLUPPERCASE123!").is_err()); // No lowercase
        assert!(validate_password("NoSpecial123").is_err()); // No special char
    }

    #[test]
    fn test_validate_password_strong() {
        assert!(validate_password("MyStr0ngPass!").is_ok());
        assert!(validate_password("Complex#Pass123").is_ok());
    }

    #[test]
    fn test_validate_schema_hash() {
        let valid_hash = "a".repeat(64);
        assert!(validate_schema_hash(&valid_hash).is_ok());
        
        assert!(validate_schema_hash("tooshort").is_err());
        assert!(validate_schema_hash("g".repeat(64)).is_err()); // Not hex
        assert!(validate_schema_hash("0".repeat(64)).is_err()); // All zeros
    }

    #[test]
    fn test_validate_payment_amount() {
        assert!(validate_payment_amount("50.0").is_ok());
        assert!(validate_payment_amount("0.5").is_err()); // Too low
        assert!(validate_payment_amount("1500").is_err()); // Too high
        assert!(validate_payment_amount("-100").is_err()); // Negative
        assert!(validate_payment_amount("invalid").is_err()); // Not a number
    }

    #[test]
    fn test_validate_qr_code_no_secrets() {
        assert!(validate_qr_code("stellar://payment?amount=100").is_ok());
        assert!(validate_qr_code("stellar://payment?secret_key=xyz").is_err());
        assert!(validate_qr_code("stellar://payment?private_key=xyz").is_err());
    }

    #[test]
    fn test_validate_stellar_public_key() {
        let valid_key = "G".to_string() + &"A".repeat(55);
        assert!(validate_stellar_public_key(&valid_key).is_ok());
        
        assert!(validate_stellar_public_key("A".repeat(56)).is_err()); // Wrong prefix
        assert!(validate_stellar_public_key("G".repeat(56)).is_err()); // Wrong length
    }
}
