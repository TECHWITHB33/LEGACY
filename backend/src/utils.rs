use uuid::Uuid;

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

pub fn generate_short_id() -> String {
    Uuid::new_v4().to_string()[..8].to_string()
}

pub fn validate_email(email: &str) -> bool {
    email.contains('@') && email.contains('.')
}

pub fn validate_password(password: &str) -> bool {
    password.len() >= 8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id = generate_id();
        assert_eq!(id.len(), 36); // UUID format
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com"));
        assert!(!validate_email("invalid-email"));
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("ValidPass123"));
        assert!(!validate_password("short"));
    }
}
