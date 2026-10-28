use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use sqlx::FromRow;

// ============================================================================
// INVOICE LIFECYCLE MODELS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvoiceState {
    Draft,
    Submitted,
    Funded,
    Processing,
    Settled,
    Closed,
}

impl InvoiceState {
    pub fn as_str(&self) -> &str {
        match self {
            InvoiceState::Draft => "Draft",
            InvoiceState::Submitted => "Submitted",
            InvoiceState::Funded => "Funded",
            InvoiceState::Processing => "Processing",
            InvoiceState::Settled => "Settled",
            InvoiceState::Closed => "Closed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Draft" => Some(InvoiceState::Draft),
            "Submitted" => Some(InvoiceState::Submitted),
            "Funded" => Some(InvoiceState::Funded),
            "Processing" => Some(InvoiceState::Processing),
            "Settled" => Some(InvoiceState::Settled),
            "Closed" => Some(InvoiceState::Closed),
            _ => None,
        }
    }

    /// Validates if a state transition is allowed
    pub fn can_transition_to(&self, next_state: &InvoiceState) -> bool {
        match (self, next_state) {
            (InvoiceState::Draft, InvoiceState::Submitted) => true,
            (InvoiceState::Submitted, InvoiceState::Funded) => true,
            (InvoiceState::Funded, InvoiceState::Processing) => true,
            (InvoiceState::Processing, InvoiceState::Settled) => true,
            (InvoiceState::Settled, InvoiceState::Closed) => true,
            // Allow refund from various states
            (InvoiceState::Funded, InvoiceState::Draft) => true, // Refund back to draft
            (InvoiceState::Processing, InvoiceState::Draft) => true, // Refund from processing
            _ => false,
        }
    }
}

/// Invoice: Core entity for invoice liquidity network
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub borrower_id: String,
    pub amount: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Invoice request: Create a new invoice
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInvoiceRequest {
    pub amount: String,
}

/// Invoice response: Invoice details
#[derive(Debug, Clone, Serialize)]
pub struct InvoiceResponse {
    pub id: String,
    pub borrower_id: String,
    pub amount: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create invoice response
#[derive(Debug, Clone, Serialize)]
pub struct CreateInvoiceResponse {
    pub invoice_id: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
}

/// Update invoice state request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateInvoiceStateRequest {
    pub state: String,
}

/// Fund invoice request
#[derive(Debug, Clone, Deserialize)]
pub struct FundInvoiceRequest {
    pub invoice_id: String,
    pub settlement_expiry_unix_timestamp: u64,
}

/// Settle invoice request
#[derive(Debug, Clone, Deserialize)]
pub struct SettleInvoiceRequest {
    pub invoice_id: String,
    pub settlement_details_json: Option<String>,
}

/// Settlement summary response
#[derive(Debug, Clone, Serialize)]
pub struct SettlementSummaryResponse {
    pub invoice_id: String,
    pub state: String,
    pub amount: String,
    pub contract_id: Option<String>,
    pub settled_at: DateTime<Utc>,
}

/// Escrow response
#[derive(Debug, Clone, Serialize)]
pub struct EscrowResponse {
    pub id: String,
    pub invoice_id: String,
    pub investor_account: String,
    pub amount: String,
    pub state: String,
    pub soroban_contract_id: Option<String>,
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    pub page: u64,
    pub per_page: u64,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

// ============================================================================
// ESCROW & SETTLEMENT MODELS
// ============================================================================

/// Escrow contract record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Escrow {
    pub id: String,
    pub invoice_id: String,
    pub investor_account: String,
    pub borrower_account: String,
    pub amount: String,
    pub settlement_expiry: i64,
    pub soroban_contract_id: String,
    pub created_at: DateTime<Utc>,
}

/// Escrow state history entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EscrowStateHistory {
    pub id: String,
    pub invoice_id: String,
    pub state: String,
    pub recorded_at: DateTime<Utc>,
}

/// Settlement fee
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SettlementFee {
    pub id: String,
    pub invoice_id: String,
    pub fee_type: String, // "platform", "network", "conversion"
    pub amount: String,
    pub currency: String,
    pub recorded_at: DateTime<Utc>,
}

/// Fee breakdown for settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub platform_fee: String,
    pub network_fee: String,
    pub conversion_fee: String,
    pub total_fees: String,
}

// ============================================================================
// AUDIT MODELS
// ============================================================================

/// Audit log entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: String,
    pub event_type: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// USER & CREDENTIAL MODELS
// ============================================================================

/// User record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub password_salt: String,
    pub created_at: DateTime<Utc>,
}

/// Register request
#[derive(Debug, Clone, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// Login request
#[derive(Debug, Clone, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Credential model
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub user_id: String,
    pub credential_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Credential status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CredentialStatus {
    Active,
    Expired,
    Revoked,
    Suspended,
}

impl CredentialStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CredentialStatus::Active => "Active",
            CredentialStatus::Expired => "Expired",
            CredentialStatus::Revoked => "Revoked",
            CredentialStatus::Suspended => "Suspended",
        }
    }
}

/// Create credential request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCredentialRequest {
    pub credential_type: String,
    pub expiration_days: i64,
}

/// Create credential response
#[derive(Debug, Clone, Serialize)]
pub struct CreateCredentialResponse {
    pub id: String,
    pub user_id: String,
    pub credential_type: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

/// Verify credential response
#[derive(Debug, Clone, Serialize)]
pub struct VerifyCredentialResponse {
    pub id: String,
    pub status: String,
    pub valid: bool,
}

// ============================================================================
// PAYMENT MODELS
// ============================================================================

/// Payment record
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub user_id: String,
    pub amount: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// BLOCKCHAIN MODELS
// ============================================================================

/// Block in blockchain
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub transaction_hash: String,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}
