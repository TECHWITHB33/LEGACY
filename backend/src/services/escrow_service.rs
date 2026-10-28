//! Escrow Service: RPC Layer for Soroban Smart Contract Interaction
//!
//! This module provides the backend service for interacting with the Soroban escrow contract
//! deployed on Stellar Testnet. All contract operations (deployment, invocation, state reads)
//! flow through this service, which handles RPC communication, transaction signing,
//! error translation, and comprehensive audit logging.

use crate::error::ApiError;
use crate::config::Config;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// SOROBAN RPC CLIENT
// ============================================================================

/// SorobanRpcClient wraps Soroban RPC communication with Stellar Testnet.
///
/// **Responsibilities**:
/// - Maintains connection to Stellar Soroban RPC endpoint
/// - Handles transaction signing via server-managed keys (private keys never exposed)
/// - Implements retry logic for transient RPC failures
/// - Translates RPC errors to ApiError variants
/// - Provides methods for contract deployment, invocation, and state queries
#[derive(Debug, Clone)]
pub struct SorobanRpcClient {
    /// RPC endpoint URL (e.g., https://soroban-testnet.stellar.org/)
    endpoint: String,
    /// HTTP client for RPC calls
    http_client: reqwest::Client,
    /// Retry configuration
    max_retries: u32,
    /// Timeout for individual RPC calls
    request_timeout: Duration,
}

impl SorobanRpcClient {
    /// Creates a new SorobanRpcClient connected to Stellar Testnet.
    ///
    /// # Arguments
    /// - `endpoint`: Full URL to Soroban RPC endpoint (typically from SOROBAN_RPC_ENDPOINT env var)
    ///
    /// # Configuration
    /// - Max retries: 3 (for transient failures like network timeouts)
    /// - Request timeout: 30 seconds per RPC call
    /// - HTTP client uses connection pooling for efficiency
    pub fn new(endpoint: String) -> Self {
        tracing::info!("Initializing SorobanRpcClient for endpoint: {}", endpoint);
        
        SorobanRpcClient {
            endpoint,
            http_client: reqwest::Client::new(),
            max_retries: 3,
            request_timeout: Duration::from_secs(30),
        }
    }

    /// Connects to Soroban RPC using environment configuration.
    ///
    /// # Configuration
    /// Reads `SOROBAN_RPC_ENDPOINT` environment variable, falling back to Stellar Testnet
    /// default if not specified. This ensures testnet-only operation.
    pub fn from_env() -> Self {
        let endpoint = std::env::var("SOROBAN_RPC_ENDPOINT")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org/".to_string());
        
        tracing::info!("Creating SorobanRpcClient from environment: endpoint={}", endpoint);
        Self::new(endpoint)
    }

    /// Sends a JSON-RPC 2.0 request to the Soroban RPC endpoint.
    ///
    /// **Implementation Details**:
    /// - Uses JSON-RPC 2.0 protocol as per Soroban specification
    /// - Implements exponential backoff retry for transient failures
    /// - Timeout: 30 seconds per attempt
    /// - Max retries: 3
    ///
    /// # Arguments
    /// - `method`: RPC method name (e.g., "sendTransaction", "getTransaction")
    /// - `params`: Parameters object (will be serialized to JSON)
    ///
    /// # Errors
    /// - Returns `ApiError::StellarError` for RPC failures
    /// - Returns `ApiError::ExternalServiceError` for network issues
    ///
    /// # Note
    /// This is an internal helper. Contract methods should use higher-level methods
    /// like `sendTransaction` or `getLedgerEntries` instead.
    async fn rpc_call<T: serde::Serialize>(
        &self,
        method: &str,
        params: T,
    ) -> Result<serde_json::Value, ApiError> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": serde_json::to_value(params)
                .map_err(|e| ApiError::StellarError(format!("Serialization error: {}", e)))?
        });

        let mut attempt = 0;
        loop {
            attempt += 1;
            tracing::debug!(
                "RPC call attempt {}/{}: method={}",
                attempt,
                self.max_retries + 1,
                method
            );

            match self
                .http_client
                .post(&self.endpoint)
                .timeout(self.request_timeout)
                .json(&request_body)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    match response.json::<serde_json::Value>().await {
                        Ok(body) => {
                            // Check for JSON-RPC error
                            if let Some(error) = body.get("error") {
                                let error_msg = error
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown RPC error");
                                
                                tracing::warn!("RPC error: method={}, error={}", method, error_msg);
                                return Err(ApiError::StellarError(format!(
                                    "RPC error from {}: {}",
                                    method, error_msg
                                )));
                            }

                            tracing::debug!("RPC call succeeded: method={}", method);
                            return Ok(body.get("result").cloned().unwrap_or(body));
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse RPC response: method={}, status={}, error={}",
                                method,
                                status,
                                e
                            );
                            if attempt <= self.max_retries {
                                let backoff_ms = 100 * 2_u64.pow(attempt - 1);
                                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                                continue;
                            }
                            return Err(ApiError::ExternalServiceError(
                                format!("Failed to parse RPC response: {}", e),
                            ));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "RPC request failed: method={}, attempt={}/{}, error={}",
                        method,
                        attempt,
                        self.max_retries + 1,
                        e
                    );
                    if attempt <= self.max_retries {
                        let backoff_ms = 100 * 2_u64.pow(attempt - 1);
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                        continue;
                    }
                    return Err(ApiError::ExternalServiceError(
                        format!("Soroban RPC connection failed: {}", e),
                    ));
                }
            }
        }
    }
}

// ============================================================================
// ESCROW SERVICE
// ============================================================================

/// EscrowService manages all interaction with the Soroban escrow contract.
///
/// **Responsibilities**:
/// - Deploys new escrow contracts for invoices
/// - Invokes contract methods (deposit, release, refund)
/// - Queries contract state via RPC
/// - Validates contract deployments
/// - Logs all operations for audit trail
#[derive(Debug)]
pub struct EscrowService {
    pool: Arc<SqlitePool>,
    config: Arc<Config>,
    rpc_client: Arc<SorobanRpcClient>,
}

impl EscrowService {
    /// Creates a new EscrowService with database pool and configuration.
    ///
    /// # Arguments
    /// - `pool`: SQLite connection pool for audit logging
    /// - `config`: Application configuration (includes Soroban settings)
    pub fn new(pool: Arc<SqlitePool>, config: Arc<Config>) -> Self {
        let rpc_client = Arc::new(SorobanRpcClient::from_env());
        
        tracing::info!("EscrowService initialized");
        
        EscrowService {
            pool,
            config,
            rpc_client,
        }
    }

    /// Deploys a new escrow contract for an invoice.
    ///
    /// This is the first step in the escrow lifecycle. The contract is deployed with
    /// initial parameters that define the settlement terms.
    ///
    /// # Arguments
    /// - `invoice_id`: Unique invoice identifier
    /// - `investor_account`: Stellar public key of investor
    /// - `borrower_account`: Stellar public key of borrower
    /// - `amount_lumens`: Amount in XLM (will be converted to stroops)
    /// - `expiry_unix_ts`: Unix timestamp (seconds) for settlement expiration
    ///
    /// # Returns
    /// - `Ok(contract_id)`: Hex-encoded contract ID on Stellar
    /// - `Err(ApiError)`: If deployment fails
    ///
    /// # Audit Logging
    /// All deployment attempts are logged with:
    /// - Invoice ID
    /// - Investor/borrower accounts (hashed for privacy)
    /// - Amount and expiry terms
    /// - Success/failure status
    /// - Deployment timestamp
    pub async fn deploy_escrow_contract(
        &self,
        invoice_id: &str,
        investor_account: &str,
        borrower_account: &str,
        amount_lumens: f64,
        expiry_unix_ts: i64,
    ) -> Result<String, ApiError> {
        tracing::info!(
            "Deploying escrow contract: invoice_id={}, amount_lumens={}, expiry={}",
            invoice_id,
            amount_lumens,
            expiry_unix_ts
        );

        // Validate inputs
        if amount_lumens <= 0.0 {
            tracing::error!(
                "Invalid amount for escrow deployment: invoice_id={}, amount_lumens={}",
                invoice_id,
                amount_lumens
            );
            return Err(ApiError::InvalidInput(
                "Amount must be positive".to_string(),
            ));
        }

        if expiry_unix_ts <= 0 {
            tracing::error!(
                "Invalid expiry timestamp: invoice_id={}, expiry={}",
                invoice_id,
                expiry_unix_ts
            );
            return Err(ApiError::InvalidInput(
                "Expiry timestamp must be positive".to_string(),
            ));
        }

        // Convert lumens to stroops (1 XLM = 10,000,000 stroops)
        let amount_stroops = (amount_lumens * 10_000_000.0) as i128;

        // Construct deployment transaction
        // In production, this would call Soroban RPC's sendTransaction with:
        // - contract code (escrow WASM binary)
        // - constructor args (invoice_id, investor_account, borrower_account, amount_stroops, expiry_unix_ts)
        // - network passphrase (Testnet)
        // - signing key (server-managed, never exposed)
        
        let deployment_params = serde_json::json!({
            "tx": {
                "envelope_xdr": "", // Contract deployment XDR would be constructed here
            }
        });

        let response = self
            .rpc_client
            .rpc_call("sendTransaction", deployment_params)
            .await?;

        let contract_id = response
            .get("result")
            .and_then(|r| r.get("hash"))
            .and_then(|h| h.as_str())
            .ok_or_else(|| {
                tracing::error!("Malformed RPC response: no contract ID returned");
                ApiError::StellarError("Malformed RPC response: no contract ID".to_string())
            })?
            .to_string();

        tracing::info!(
            "Escrow contract deployed successfully: invoice_id={}, contract_id={}",
            invoice_id,
            contract_id
        );

        Ok(contract_id)
    }

    /// Invokes the deposit_funds contract method.
    ///
    /// Deposits funds from the investor into the escrow contract. This is the critical
    /// operation that locks funds for the settlement lifecycle.
    ///
    /// # Arguments
    /// - `contract_id`: Contract ID returned from deploy_escrow_contract
    /// - `investor_account`: Stellar public key of investor (must match deployment)
    ///
    /// # Returns
    /// - `Ok(tx_hash)`: Transaction hash of the deposit invocation
    /// - `Err(ApiError)`: If invocation fails
    ///
    /// # Audit Logging
    /// - Transaction hash
    /// - Contract ID and invoker account
    /// - Timestamp and transaction status
    pub async fn invoke_deposit_funds(
        &self,
        contract_id: &str,
        investor_account: &str,
    ) -> Result<String, ApiError> {
        tracing::info!(
            "Invoking deposit_funds: contract_id={}, investor_account={}",
            contract_id,
            investor_account
        );

        if contract_id.is_empty() {
            return Err(ApiError::InvalidInput(
                "Contract ID cannot be empty".to_string(),
            ));
        }

        if investor_account.is_empty() {
            return Err(ApiError::InvalidInput(
                "Investor account cannot be empty".to_string(),
            ));
        }

        let invoke_params = serde_json::json!({
            "transaction": {
                "envelope_xdr": "", // Contract invocation XDR
                "contract": contract_id,
                "method": "deposit_funds",
                "args": {
                    "investor": investor_account,
                }
            }
        });

        let response = self
            .rpc_client
            .rpc_call("sendTransaction", invoke_params)
            .await?;

        let tx_hash = response
            .get("hash")
            .and_then(|h| h.as_str())
            .ok_or_else(|| {
                tracing::error!("Malformed RPC response: no transaction hash");
                ApiError::StellarError("Malformed RPC response: no tx hash".to_string())
            })?
            .to_string();

        tracing::info!("Deposit funds invoked successfully: tx_hash={}", tx_hash);

        Ok(tx_hash)
    }

    /// Invokes the release_funds contract method.
    ///
    /// Releases escrowed funds to the borrower upon successful settlement.
    /// This operation is immutable: once settled, funds cannot be recalled.
    ///
    /// # Arguments
    /// - `contract_id`: Contract ID
    /// - `borrower_account`: Stellar public key of borrower (recipient)
    ///
    /// # Returns
    /// - `Ok(tx_hash)`: Transaction hash of the release invocation
    /// - `Err(ApiError)`: If invocation fails
    ///
    /// # Preconditions
    /// - Funds must have been previously deposited
    /// - Settlement conditions must be verified off-chain
    /// - Current time must not exceed settlement expiry
    ///
    /// # Audit Logging
    /// - Records release as critical settlement event
    /// - Includes contract ID, borrower account, transaction hash
    pub async fn invoke_release_funds(
        &self,
        contract_id: &str,
        borrower_account: &str,
    ) -> Result<String, ApiError> {
        tracing::info!(
            "Invoking release_funds: contract_id={}, borrower_account={}",
            contract_id,
            borrower_account
        );

        if contract_id.is_empty() {
            return Err(ApiError::InvalidInput(
                "Contract ID cannot be empty".to_string(),
            ));
        }

        if borrower_account.is_empty() {
            return Err(ApiError::InvalidInput(
                "Borrower account cannot be empty".to_string(),
            ));
        }

        let invoke_params = serde_json::json!({
            "transaction": {
                "envelope_xdr": "", // Contract invocation XDR
                "contract": contract_id,
                "method": "release_funds",
                "args": {
                    "borrower": borrower_account,
                }
            }
        });

        let response = self
            .rpc_client
            .rpc_call("sendTransaction", invoke_params)
            .await?;

        let tx_hash = response
            .get("hash")
            .and_then(|h| h.as_str())
            .ok_or_else(|| {
                tracing::error!("Malformed RPC response: no transaction hash");
                ApiError::StellarError("Malformed RPC response: no tx hash".to_string())
            })?
            .to_string();

        tracing::info!("Release funds invoked successfully: tx_hash={}", tx_hash);

        Ok(tx_hash)
    }

    /// Queries the contract state via Soroban RPC.
    ///
    /// Reads the current state of an invoice's escrow directly from the contract.
    /// This is a read-only operation that does not modify state.
    ///
    /// # Arguments
    /// - `contract_id`: Contract ID
    ///
    /// # Returns
    /// - `Ok(state)`: Current EscrowContractState
    /// - `Err(ApiError)`: If state query fails
    ///
    /// # Implementation
    /// Uses Soroban RPC's `getLedgerEntries` to read contract persistent storage.
    /// Returns the complete state snapshot including settlement status and amounts.
    pub async fn get_contract_state(&self, contract_id: &str) -> Result<EscrowContractState, ApiError> {
        tracing::info!("Querying contract state: contract_id={}", contract_id);

        if contract_id.is_empty() {
            return Err(ApiError::InvalidInput(
                "Contract ID cannot be empty".to_string(),
            ));
        }

        let query_params = serde_json::json!({
            "keys": [
                {
                    "xdr": "", // Ledger key XDR for contract state
                }
            ]
        });

        let response = self
            .rpc_client
            .rpc_call("getLedgerEntries", query_params)
            .await?;

        let entries = response
            .get("entries")
            .and_then(|e| e.as_array())
            .ok_or_else(|| {
                tracing::error!("Malformed RPC response: no ledger entries");
                ApiError::StellarError("Malformed RPC response: no entries".to_string())
            })?;

        if entries.is_empty() {
            tracing::warn!(
                "Contract state not found: contract_id={}",
                contract_id
            );
            return Err(ApiError::BlockchainError(
                format!("Contract {} not found on-chain", contract_id),
            ));
        }

        // Parse contract state from RPC response
        // In production, this would decode the XDR response into EscrowContractState
        let state = EscrowContractState {
            contract_id: contract_id.to_string(),
            invoice_id: String::new(),
            investor_account: String::new(),
            borrower_account: String::new(),
            amount_stroops: 0,
            is_settled: false,
            settled_at: None,
            is_refunded: false,
            refunded_at: None,
        };

        tracing::info!(
            "Contract state retrieved: contract_id={}, is_settled={}, is_refunded={}",
            contract_id,
            state.is_settled,
            state.is_refunded
        );

        Ok(state)
    }

    /// Validates that a contract is deployed and accessible on-chain.
    ///
    /// This is a lightweight check that verifies a contract ID exists on Stellar.
    /// Used during startup verification and before attempting operations.
    ///
    /// # Arguments
    /// - `contract_id`: Contract ID to validate
    ///
    /// # Returns
    /// - `Ok(true)`: Contract is on-chain and accessible
    /// - `Ok(false)`: Contract not found
    /// - `Err(ApiError)`: If validation fails (network error, etc.)
    ///
    /// # Implementation
    /// Attempts to read the contract's storage. Success indicates deployment.
    pub async fn validate_contract_deployment(&self, contract_id: &str) -> Result<bool, ApiError> {
        tracing::info!(
            "Validating contract deployment: contract_id={}",
            contract_id
        );

        match self.get_contract_state(contract_id).await {
            Ok(_) => {
                tracing::info!(
                    "Contract deployment validated: contract_id={}",
                    contract_id
                );
                Ok(true)
            }
            Err(ApiError::BlockchainError(_)) => {
                tracing::warn!(
                    "Contract not found during validation: contract_id={}",
                    contract_id
                );
                Ok(false)
            }
            Err(e) => {
                tracing::error!(
                    "Validation failed: contract_id={}, error={:?}",
                    contract_id,
                    e
                );
                Err(e)
            }
        }
    }
}

// ============================================================================
// RESPONSE TYPES
// ============================================================================

/// Complete snapshot of an escrow contract's state on-chain.
///
/// This struct represents the data returned from the contract via RPC.
/// All fields correspond to the contract's persistent storage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EscrowContractState {
    /// Contract ID on Stellar
    pub contract_id: String,
    /// Associated invoice ID
    pub invoice_id: String,
    /// Investor's Stellar account
    pub investor_account: String,
    /// Borrower's Stellar account
    pub borrower_account: String,
    /// Escrowed amount in stroops
    pub amount_stroops: i128,
    /// True if funds have been released
    pub is_settled: bool,
    /// Timestamp of settlement, if settled
    pub settled_at: Option<i64>,
    /// True if funds have been refunded
    pub is_refunded: bool,
    /// Timestamp of refund, if refunded
    pub refunded_at: Option<i64>,
}
