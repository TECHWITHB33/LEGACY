use crate::error::ApiError;
use crate::crypto::key_derivation;

/// Hash a password and return (password_hash, salt)
/// Returns both hex-encoded for storage in database
pub fn hash_password(password: &str) -> Result<(String, String), ApiError> {
    // Validate password strength
    validate_password_strength(password)?;

    // Derive key using PBKDF2-SHA256
    let (password_hash, salt) = key_derivation::derive_key(password, None)?;

    Ok((password_hash, salt))
}

/// Verify a password against stored hash and salt
pub fn verify_password(password: &str, password_hash: &str, salt: &str) -> Result<bool, ApiError> {
    key_derivation::verify_key(password, password_hash, salt)
}

/// Validate password strength requirements
fn validate_password_strength(password: &str) -> Result<(), ApiError> {
    if password.len() < 8 {
        return Err(ApiError::ValidationError(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one digit".to_string(),
        ));
    }

    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ApiError::ValidationError(
            "Password must contain at least one special character".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SecurePass123!";
        
        let (hash, salt) = hash_password(password).expect("Failed to hash password");
        assert!(!hash.is_empty());
        assert!(!salt.is_empty());

        let is_valid = verify_password(password, &hash, &salt)
            .expect("Failed to verify password");
        assert!(is_valid);

        let is_invalid = verify_password("WrongPassword123!", &hash, &salt)
            .expect("Failed to verify password");
        assert!(!is_invalid);
    }

    #[test]
    fn test_password_validation_too_short() {
        let result = validate_password_strength("Short1!");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_uppercase() {
        let result = validate_password_strength("lowercase123!");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_lowercase() {
        let result = validate_password_strength("UPPERCASE123!");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_digit() {
        let result = validate_password_strength("NoDigits!");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_no_special() {
        let result = validate_password_strength("NoSpecial123");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_validation_valid() {
        let result = validate_password_strength("ValidPass123!");
        assert!(result.is_ok());
    }
}
