use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Database URL
    pub database_url: String,
    /// JWT secret
    pub jwt_secret: String,
    /// Soroban RPC endpoint
    pub soroban_rpc_endpoint: String,
    /// Stellar network passphrase
    pub stellar_network_passphrase: String,
}

impl Config {
    /// Load config from environment variables
    pub fn from_env() -> Self {
        Config {
            host: std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://data/id_card_system.db".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string()),
            soroban_rpc_endpoint: std::env::var("SOROBAN_RPC_ENDPOINT")
                .unwrap_or_else(|_| "https://soroban-testnet.stellar.org/".to_string()),
            stellar_network_passphrase: std::env::var("STELLAR_NETWORK_PASSPHRASE")
                .unwrap_or_else(|_| "Test SDF Network ; September 2015".to_string()),
        }
    }
}
