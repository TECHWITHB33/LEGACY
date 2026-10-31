//! Soroban Escrow Smart Contract Specification
//!
//! This module defines the contract interface, events, and stored data structures
//! for the Legacy escrow contract deployed on Stellar's Soroban platform.
//!
//! ## Contract Overview
//!
//! The escrow contract enforces the following guarantees:
//! - **Investor Fund Locking**: Investor funds are locked until settlement conditions are met
//! - **Immutable Transaction Records**: All transaction records are immutable; funds cannot be
//!   recalled or refunded after settlement unless explicitly permitted by contract logic
//! - **Event Emission**: All significant state transitions emit events for audit and settlement tracking
//! - **Token Interface Compliance**: The contract complies with Soroban's token interface (SEP-0041)
//!   for deposit/release operations

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Soroban Escrow Contract Specification
///
/// This struct documents the complete interface and behavior of the escrow contract.
/// The contract is deployed on Stellar Testnet and manages investor funds for tokenized invoices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SorobanEscrowSpec {
    /// Contract ID on Stellar (hex-encoded)
    pub contract_id: String,
    /// Network where contract is deployed (testnet)
    pub network: String,
    /// Contract deployment timestamp
    pub deployed_at: DateTime<Utc>,
}

impl SorobanEscrowSpec {
    pub fn new(contract_id: String) -> Self {
        SorobanEscrowSpec {
            contract_id,
            network: "testnet".to_string(),
            deployed_at: Utc::now(),
        }
    }
}

// ============================================================================
// CONTRACT METHODS
// ============================================================================

/// Contract method: deposit_funds
///
/// Deposits funds from an investor into the escrow contract. This is the initial
/// funding step for a tokenized invoice settlement.
///
/// **Parameters**:
/// - `invoice_id` (string): Unique identifier for the invoice
/// - `investor_account` (string): Stellar public key of the investor funding the escrow
/// - `amount_stroops` (i128): Amount in stroops (1 XLM = 10,000,000 stroops)
///
/// **Preconditions**:
/// - Investor must have sufficient XLM balance
/// - Invoice must not already have funds deposited
///
/// **Postconditions**:
/// - Funds are transferred from investor to contract
/// - Contract state is updated with deposit details
/// - `FundsDeposited` event is emitted
///
/// **Returns**: transaction hash (hex string)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositFundsMethod {
    pub invoice_id: String,
    pub investor_account: String,
    pub amount_stroops: i128,
}

/// Contract method: release_funds
///
/// Releases escrowed funds to the borrower upon settlement. This is called after
/// settlement conditions have been verified and settlement expiry has not passed.
///
/// **Parameters**:
/// - `invoice_id` (string): Invoice being settled
/// - `borrower_account` (string): Stellar public key receiving the funds
///
/// **Preconditions**:
/// - Funds must have been previously deposited
/// - Settlement conditions must be met (verified off-chain)
/// - Current timestamp must not exceed settlement_expiry_timestamp
/// - Funds must not have been previously released
///
/// **Postconditions**:
/// - Funds are transferred from contract to borrower
/// - `is_settled` flag is set to true (immutable thereafter)
/// - `FundsReleased` event is emitted
///
/// **Returns**: transaction hash (hex string)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseFundsMethod {
    pub invoice_id: String,
    pub borrower_account: String,
}

/// Contract method: refund_funds
///
/// Refunds escrowed funds back to the investor if settlement conditions fail or
/// expiry is reached without settlement.
///
/// **Parameters**:
/// - `invoice_id` (string): Invoice to refund
/// - `investor_account` (string): Stellar public key receiving refunded funds
///
/// **Preconditions**:
/// - Funds must have been previously deposited
/// - Either: (a) settlement has expired without being settled, or
///   (b) settlement conditions explicitly failed (verified off-chain)
/// - Funds must not have been previously released
///
/// **Postconditions**:
/// - Funds are transferred from contract back to investor
/// - `is_refunded` flag is set to true (immutable thereafter)
/// - `FundsRefunded` event is emitted
///
/// **Returns**: transaction hash (hex string)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundFundsMethod {
    pub invoice_id: String,
    pub investor_account: String,
}

/// Contract method: get_state
///
/// Reads the current state of an invoice's escrow. This is a read-only operation
/// that queries the contract's persistent storage on Stellar.
///
/// **Parameters**:
/// - `invoice_id` (string): Invoice ID to query
///
/// **Returns**: `EscrowState` struct with all stored data, or error if not found
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStateMethod {
    pub invoice_id: String,
}

// ============================================================================
// CONTRACT EVENTS
// ============================================================================

/// Event: FundsDeposited
///
/// Emitted when funds are initially deposited into escrow for an invoice.
/// Marks the start of the settlement lifecycle.
///
/// **Fields**:
/// - `invoice_id` (string): The invoice ID
/// - `investor_account` (string): Investor's Stellar account
/// - `amount_stroops` (i128): Amount deposited
/// - `timestamp` (i64): Unix timestamp of deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsDepositedEvent {
    pub invoice_id: String,
    pub investor_account: String,
    pub amount_stroops: i128,
    pub timestamp: i64,
}

/// Event: FundsReleased
///
/// Emitted when escrowed funds are successfully released to the borrower.
/// Marks successful invoice settlement.
///
/// **Fields**:
/// - `invoice_id` (string): The invoice ID
/// - `borrower_account` (string): Borrower's Stellar account (recipient)
/// - `amount_stroops` (i128): Amount released
/// - `timestamp` (i64): Unix timestamp of release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsReleasedEvent {
    pub invoice_id: String,
    pub borrower_account: String,
    pub amount_stroops: i128,
    pub timestamp: i64,
}

/// Event: FundsRefunded
///
/// Emitted when escrowed funds are refunded back to the investor.
/// Marks failed or expired settlement.
///
/// **Fields**:
/// - `invoice_id` (string): The invoice ID
/// - `investor_account` (string): Investor's Stellar account (recipient)
/// - `amount_stroops` (i128): Amount refunded
/// - `reason` (string): Reason for refund (e.g., "settlement_expired", "settlement_failed")
/// - `timestamp` (i64): Unix timestamp of refund
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsRefundedEvent {
    pub invoice_id: String,
    pub investor_account: String,
    pub amount_stroops: i128,
    pub reason: String,
    pub timestamp: i64,
}

/// Event: StateChanged
///
/// Emitted on any significant state transition. Provides a general audit trail
/// for all contract operations.
///
/// **Fields**:
/// - `invoice_id` (string): The invoice ID
/// - `old_state` (string): Previous state (serialized)
/// - `new_state` (string): New state (serialized)
/// - `timestamp` (i64): Unix timestamp of change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChangedEvent {
    pub invoice_id: String,
    pub old_state: String,
    pub new_state: String,
    pub timestamp: i64,
}

// ============================================================================
// CONTRACT STORED DATA
// ============================================================================

/// Escrow State: Complete snapshot of an invoice's escrow data
///
/// This struct represents the complete persistent storage of a single invoice's
/// escrow state on the Soroban contract. All fields are immutable after creation
/// of the corresponding record (enforced by contract logic).
///
/// **Immutability Guarantees**:
/// - `invoice_id`, `investor_account`, `borrower_account`, `amount_in_stroops`,
///   `settlement_expiry_timestamp` are set once at deposit and never change
/// - `is_settled` transitions from false → true exactly once, never back to false
/// - `is_refunded` transitions from false → true exactly once, never back to false
/// - `settled_at` and `refunded_at` are set once when the corresponding transition occurs
/// - These immutability guarantees ensure audit trail integrity and prevent fund manipulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscrowState {
    /// Unique identifier for the invoice (correlates to off-chain invoice database)
    pub invoice_id: String,

    /// Stellar public key of the investor depositing funds (immutable)
    pub investor_account: String,

    /// Stellar public key of the borrower receiving funds on settlement (immutable)
    pub borrower_account: String,

    /// Amount of XLM held in escrow, expressed in stroops (immutable)
    /// Note: 1 XLM = 10,000,000 stroops
    pub amount_in_stroops: i128,

    /// Unix timestamp (seconds since epoch) after which settlement cannot occur (immutable)
    /// If not settled by this time, refund may be processed
    pub settlement_expiry_timestamp: i64,

    /// Flag indicating whether funds have been released to borrower (immutable once true)
    pub is_settled: bool,

    /// Timestamp when funds were released, or None if not yet settled (immutable once set)
    pub settled_at: Option<i64>,

    /// Flag indicating whether funds have been refunded to investor (immutable once true)
    pub is_refunded: bool,

    /// Timestamp when funds were refunded, or None if not yet refunded (immutable once set)
    pub refunded_at: Option<i64>,
}

impl EscrowState {
    pub fn new(
        invoice_id: String,
        investor_account: String,
        borrower_account: String,
        amount_in_stroops: i128,
        settlement_expiry_timestamp: i64,
    ) -> Self {
        EscrowState {
            invoice_id,
            investor_account,
            borrower_account,
            amount_in_stroops,
            settlement_expiry_timestamp,
            is_settled: false,
            settled_at: None,
            is_refunded: false,
            refunded_at: None,
        }
    }
}

// ============================================================================
// TOKEN INTERFACE COMPLIANCE
// ============================================================================
//
// The contract complies with Soroban's token interface as defined in SEP-0041.
//
// Key compliance points:
// - Transfer operations use standard token semantics (from, to, amount in stroops)
// - All amounts are expressed in stroops (the native unit of XLM)
// - Transaction signing follows Stellar's standard envelope format
// - Errors are propagated using Soroban's standard error codes
//
// References:
// - Soroban SDK: https://github.com/stellar/rs-soroban-sdk
// - Token Interface: https://stellar.org/docs/learn/glossary#token-contract
