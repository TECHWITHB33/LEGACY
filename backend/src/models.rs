use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// ENUMS
// ============================================================================

// FIX VULN-2: Role-Based Access Control
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum UserRole {
    #[serde(rename = "student")]
    Student,
    #[serde(rename = "merchant")]
    Merchant,
    #[serde(rename = "issuer")]
    Issuer,
    #[serde(rename = "administrator")]
    Administrator,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Student => "student",
            UserRole::Merchant => "merchant",
            UserRole::Issuer => "issuer",
            UserRole::Administrator => "administrator",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "student" => Ok(UserRole::Student),
            "merchant" => Ok(UserRole::Merchant),
            "issuer" => Ok(UserRole::Issuer),
            "administrator" => Ok(UserRole::Administrator),
            _ => Err(format!("Invalid role: {}", s)),
        }
    }

    pub fn can_issue_credentials(&self) -> bool {
        matches!(self, UserRole::Issuer | UserRole::Administrator)
    }

    pub fn can_revoke_credentials(&self) -> bool {
        matches!(self, UserRole::Administrator)
    }

    pub fn can_modify_payment_limits(&self) -> bool {
        matches!(self, UserRole::Administrator)
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum CredentialStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "revoked")]
    Revoked,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "suspended")]
    Suspended,
    #[serde(rename = "pending")]
    Pending,
}

impl CredentialStatus {
    pub fn as_str(&self) -> &str {
        match self {
            CredentialStatus::Active => "active",
            CredentialStatus::Revoked => "revoked",
            CredentialStatus::Expired => "expired",
            CredentialStatus::Suspended => "suspended",
            CredentialStatus::Pending => "pending",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(CredentialStatus::Active),
            "revoked" => Ok(CredentialStatus::Revoked),
            "expired" => Ok(CredentialStatus::Expired),
            "suspended" => Ok(CredentialStatus::Suspended),
            "pending" => Ok(CredentialStatus::Pending),
            _ => Err(format!("Invalid credential status: {}", s)),
        }
    }
}

impl std::fmt::Display for CredentialStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterRequest {
    pub email: String,
    pub name: String,
    pub institution: String,
    pub matric_number: String,
    pub age: u8,
    pub gender: String,
    pub department: String,
    pub phone_contact: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub public_key: String,
    pub credential_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifySignatureRequest {
    pub public_key: String,
    pub message: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifySignatureResponse {
    pub valid: bool,
    pub user_id: Option<String>,
}

// ============================================================================
// CREDENTIAL REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialRequest {
    pub user_id: String,
    pub schema_hash: String, // Must be exactly 32 bytes (64 chars hex)
    pub qr_code: String,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCredentialResponse {
    pub credential_id: String,
    pub credential_hash: String,
    pub signature: String,
    pub status: String,
    pub version: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyCredentialRequest {
    pub credential_id: String,
    pub credential_data: String, // Canonical JSON representation
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyCredentialResponse {
    pub valid: bool,
    pub credential_id: String,
    pub user_id: String,
    pub status: String,
    pub version: i32,
    pub signed_at: Option<DateTime<Utc>>,
    pub verification_message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeCredentialRequest {
    pub credential_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevokeCredentialResponse {
    pub credential_id: String,
    pub status: String,
    pub revoked_at: DateTime<Utc>,
}

// ============================================================================
// DATABASE MODELS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub institution: String,
    pub matric_number: String,
    pub age: u8,
    pub gender: String,
    pub department: String,
    pub phone_contact: String,
    pub password_hash: String,
    pub public_key: String,
    pub private_key_encrypted: String,
    pub status: String, // active, suspended, revoked
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Credential {
    pub id: String,
    pub user_id: String,
    pub public_key_hash: String,
    pub qr_code: String,
    pub qr_hash: String,
    pub status: String, // active, expired, revoked, suspended
    pub version: i32,
    pub schema_hash: String, // SHA-256 hash of credential schema (32 bytes hex encoded)
    pub credential_hash: String, // SHA-256 hash of canonical credential JSON
    pub signature: String, // EdDSA signature of credential_hash
    pub signed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub hash: String,
    pub timestamp: DateTime<Utc>,
    pub data: String, // JSON serialized data
    pub nonce: u64,
    pub difficulty: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainRecord {
    pub id: String,
    pub credential_id: String,
    pub block_index: u64,
    pub block_hash: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    pub id: String,
    pub user_id: String,
    pub amount: String, // Stellar asset amount
    pub asset_code: String,
    pub recipient_address: String,
    pub merchant_id: Option<String>,
    pub status: String, // pending, completed, failed, revoked
    pub transaction_hash: Option<String>,
    pub qr_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: String, // JSON
    pub ip_address: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ============================================================================
// INVOICE AND ESCROW MODELS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum InvoiceState {
    #[serde(rename = "Draft")]
    Draft,
    #[serde(rename = "Submitted")]
    Submitted,
    #[serde(rename = "Funded")]
    Funded,
    #[serde(rename = "Processing")]
    Processing,
    #[serde(rename = "Settled")]
    Settled,
    #[serde(rename = "Closed")]
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

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "Draft" => Ok(InvoiceState::Draft),
            "Submitted" => Ok(InvoiceState::Submitted),
            "Funded" => Ok(InvoiceState::Funded),
            "Processing" => Ok(InvoiceState::Processing),
            "Settled" => Ok(InvoiceState::Settled),
            "Closed" => Ok(InvoiceState::Closed),
            _ => Err(format!("Invalid invoice state: {}", s)),
        }
    }
}

impl std::fmt::Display for InvoiceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invoice {
    pub id: String,
    pub borrower_id: String,
    pub amount: String,
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Escrow {
    pub id: String,
    pub invoice_id: String,
    pub investor_account: String,
    pub borrower_account: String,
    pub amount: String,
    pub settlement_expiry: DateTime<Utc>,
    pub soroban_contract_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EscrowStateHistory {
    pub id: String,
    pub escrow_id: String,
    pub previous_state: String,
    pub new_state: String,
    pub timestamp: DateTime<Utc>,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SettlementFee {
    pub id: String,
    pub invoice_id: String,
    pub fee_type: String,
    pub amount: String,
    pub currency: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub platform_fee: String,
    pub network_fee: String,
    pub currency_conversion_fee: String,
    pub total_fees: String,
}

// ============================================================================
// INVOICE AND ESCROW REQUEST/RESPONSE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateInvoiceRequest {
    pub borrower_id: String,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInvoiceStateRequest {
    pub invoice_id: String,
    pub new_state: String,
    pub details_json: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateEscrowRequest {
    pub invoice_id: String,
    pub investor_account: String,
    pub settlement_expiry_unix_timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EscrowResponse {
    pub id: String,
    pub invoice_id: String,
    pub investor_account: String,
    pub amount: String,
    pub state: String,
    pub soroban_contract_id: Option<String>,
}

// ============================================================================
// BLOCKCHAIN-SPECIFIC MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockchainBlockData {
    pub credential_id: String,
    pub user_id: String,
    pub user_email: String,
    pub public_key_hash: String,
    pub qr_hash: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainValidationResult {
    pub is_valid: bool,
    pub block_index: u64,
    pub error: Option<String>,
}

// ============================================================================
// QR CODE MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct QrGenerateRequest {
    pub credential_id: String,
    pub include_private_data: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrGenerateResponse {
    pub qr_code_data: String,
    pub qr_code_image: String,
    pub encoding: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrVerifyRequest {
    pub qr_code_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QrVerifyResponse {
    pub valid: bool,
    pub credential_id: String,
    pub user_id: String,
    pub user_email: String,
}

// ============================================================================
// PAYMENT MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: String,
    pub asset_code: String,
    pub recipient_address: String,
    pub merchant_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub id: String,
    pub status: String,
    pub amount: String,
    pub transaction_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantQrRequest {
    pub amount: String,
    pub merchant_name: String,
    pub merchant_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MerchantQrResponse {
    pub qr_code: String,
    pub merchant_id: String,
    pub amount: String,
}

// ============================================================================
// BIOMETRIC/WEBAUTHN MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricRegisterStartRequest {
    pub user_id: String,
    pub device_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricRegisterStartResponse {
    pub registration_id: String,
    pub challenge: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricRegisterCompleteRequest {
    pub registration_id: String,
    pub credential_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricAuthenticateStartRequest {
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricAuthenticateStartResponse {
    pub authentication_id: String,
    pub challenge: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricAuthenticateCompleteRequest {
    pub authentication_id: String,
    pub assertion_data: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BiometricAuthenticateResponse {
    pub authenticated: bool,
    pub token: Option<String>,
}

// ============================================================================
// PAGINATION MODELS
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

// ============================================================================
// HEALTH CHECK
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}
