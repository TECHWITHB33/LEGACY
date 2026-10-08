use crate::error::ApiError;

pub struct CryptographyService;

impl CryptographyService {
    pub fn new() -> Self {
        CryptographyService
    }

    /// Generate RSA keypair
    pub fn generate_rsa_keypair(&self) -> Result<(String, String), ApiError> {
        let keypair = crate::crypto::generate_rsa_keypair()?;
        Ok((keypair.public_key_pem, keypair.private_key_pem))
    }

    /// Hash data using SHA-256
    pub fn hash_sha256(&self, data: &[u8]) -> String {
        crate::crypto::sha256_hex(data)
    }

    /// Hash password using PBKDF2
    pub fn hash_password(&self, password: &str) -> Result<(String, String), ApiError> {
        crate::crypto::hash_password(password)
    }

    /// Verify password
    pub fn verify_password(
        &self,
        password: &str,
        hash: &str,
        salt: &str,
    ) -> Result<bool, ApiError> {
        crate::crypto::verify_password(password, hash, salt)
    }

    /// Sign data using EdDSA
    pub fn sign_data(&self, message: &[u8], private_key_hex: &str) -> Result<String, ApiError> {
        crate::crypto::sign_data(message, private_key_hex)
    }

    /// Verify signature using EdDSA
    pub fn verify_signature(
        &self,
        message: &[u8],
        signature_hex: &str,
        public_key_hex: &str,
    ) -> Result<bool, ApiError> {
        crate::crypto::verify_signature(message, signature_hex, public_key_hex)
    }
}
