use crate::error::ApiError;
use serde::{Deserialize, Serialize};

/// RSA Key Pair wrapper for 4096-bit RSA keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RSAKeyPair {
    pub public_key_pem: String,
    pub private_key_pem: String,
    pub key_id: String,
}

/// Generate a new RSA 4096-bit keypair
pub fn generate_rsa_keypair() -> Result<RSAKeyPair, ApiError> {
    use rsa::{RsaPrivateKey, RsaPublicKey};
    use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

    let mut rng = rand::thread_rng();
    
    // Generate 4096-bit RSA keypair
    let bits = 4096;
    let private_key = RsaPrivateKey::new(&mut rng, bits)
        .map_err(|e| ApiError::CryptoError(format!("Failed to generate RSA key: {}", e)))?;
    
    let public_key = RsaPublicKey::from(&private_key);

    // Encode to PEM format
    let private_key_pem = private_key
        .to_pkcs8_pem(Default::default())
        .map_err(|e| ApiError::CryptoError(format!("Failed to encode private key: {}", e)))?
        .to_string();

    let public_key_pem = public_key
        .to_public_key_pem(Default::default())
        .map_err(|e| ApiError::CryptoError(format!("Failed to encode public key: {}", e)))?;

    // Generate a unique key ID (hash of public key)
    let key_id = crate::crypto::hash::sha256_hex(&public_key_pem);

    Ok(RSAKeyPair {
        public_key_pem,
        private_key_pem,
        key_id,
    })
}

/// Load RSA private key from PEM string
pub fn load_private_key(pem: &str) -> Result<rsa::RsaPrivateKey, ApiError> {
    use rsa::pkcs8::DecodePrivateKey;
    
    rsa::RsaPrivateKey::from_pkcs8_pem(pem)
        .map_err(|e| ApiError::CryptoError(format!("Failed to load private key: {}", e)))
}

/// Load RSA public key from PEM string
pub fn load_public_key(pem: &str) -> Result<rsa::RsaPublicKey, ApiError> {
    use rsa::pkcs8::DecodePublicKey;
    
    rsa::RsaPublicKey::from_public_key_pem(pem)
        .map_err(|e| ApiError::CryptoError(format!("Failed to load public key: {}", e)))
}

/// Encrypt data with RSA public key
pub fn encrypt(public_key: &rsa::RsaPublicKey, data: &[u8]) -> Result<Vec<u8>, ApiError> {
    use rsa::Pkcs1v15Encrypt;
    
    public_key
        .encrypt(&mut rand::thread_rng(), Pkcs1v15Encrypt, data)
        .map_err(|e| ApiError::CryptoError(format!("Encryption failed: {}", e)))
}

/// Decrypt data with RSA private key
pub fn decrypt(private_key: &rsa::RsaPrivateKey, encrypted_data: &[u8]) -> Result<Vec<u8>, ApiError> {
    use rsa::Pkcs1v15Encrypt;
    
    private_key
        .decrypt(Pkcs1v15Encrypt, encrypted_data)
        .map_err(|e| ApiError::CryptoError(format!("Decryption failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = generate_rsa_keypair().expect("Failed to generate keypair");
        assert!(!keypair.public_key_pem.is_empty());
        assert!(!keypair.private_key_pem.is_empty());
        assert!(!keypair.key_id.is_empty());
    }

    #[test]
    fn test_encrypt_decrypt() {
        let keypair = generate_rsa_keypair().expect("Failed to generate keypair");
        let public_key = load_public_key(&keypair.public_key_pem).expect("Failed to load public key");
        let private_key = load_private_key(&keypair.private_key_pem).expect("Failed to load private key");

        let original_data = b"Test encryption message";
        let encrypted = encrypt(&public_key, original_data).expect("Encryption failed");
        let decrypted = decrypt(&private_key, &encrypted).expect("Decryption failed");

        assert_eq!(original_data.to_vec(), decrypted);
    }
}
