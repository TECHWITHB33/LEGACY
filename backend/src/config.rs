use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub crypto: CryptoConfig,
    pub stellar: StellarConfig,
    pub features: FeaturesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_allowed_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiry_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub pbkdf2_iterations: u32,
    pub key_derivation_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarConfig {
    pub network: String,
    pub horizon_url: String,
    pub network_passphrase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub enable_biometric_auth: bool,
    pub enable_stellar_payments: bool,
    pub enable_qr_verification: bool,
    pub enable_audit_logging: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let server = ServerConfig {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|_| "SERVER_PORT must be a valid u16")?,
            cors_allowed_origins: std::env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:8000".to_string())
                .split(',')
                .map(|s| s.to_string())
                .collect(),
        };

        let database = DatabaseConfig {
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/id_card_system.db".to_string()),
            max_connections: 10,
        };

        let jwt = JwtConfig {
            secret: std::env::var("JWT_SECRET")
                .map_err(|_| "JWT_SECRET must be set")?,
            expiry_seconds: std::env::var("JWT_EXPIRY")
                .unwrap_or_else(|_| "3600".to_string())
                .parse()
                .map_err(|_| "JWT_EXPIRY must be a valid u64")?,
        };

        let crypto = CryptoConfig {
            pbkdf2_iterations: std::env::var("PBKDF2_ITERATIONS")
                .unwrap_or_else(|_| "100000".to_string())
                .parse()
                .map_err(|_| "PBKDF2_ITERATIONS must be a valid u32")?,
            key_derivation_algorithm: "PBKDF2-SHA256".to_string(),
        };

        let stellar = StellarConfig {
            network: std::env::var("STELLAR_NETWORK")
                .unwrap_or_else(|_| "testnet".to_string()),
            horizon_url: std::env::var("STELLAR_HORIZON_URL")
                .unwrap_or_else(|_| "https://horizon-testnet.stellar.org".to_string()),
            network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
        };

        let features = FeaturesConfig {
            enable_biometric_auth: std::env::var("ENABLE_BIOMETRIC_AUTH")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_stellar_payments: std::env::var("ENABLE_STELLAR_PAYMENTS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_qr_verification: std::env::var("ENABLE_QR_VERIFICATION")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            enable_audit_logging: std::env::var("ENABLE_AUDIT_LOGGING")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        };

        Ok(Config {
            server,
            database,
            jwt,
            crypto,
            stellar,
            features,
        })
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Config {{ server: {}:{}, database: {}, features: {:?} }}",
            self.server.host, self.server.port, self.database.url, self.features
        )
    }
}
