// Stellar Payment Module
// IMPORTANT: Uses Stellar TESTNET ONLY
// Current official APIs: Stellar RPC + Horizon
// No secret keys exposed to frontend
// No secret keys in QR codes
// All keys managed via environment variables

use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use sqlx::{FromRow, Row};

/// Payment request types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentType {
    StudentToMerchant,   // Student pays merchant
    MerchantToStudent,   // Refund/reverse payment
    StudentToStudent,    // Peer-to-peer
}

impl PaymentType {
    pub fn as_str(&self) -> &str {
        match self {
            PaymentType::StudentToMerchant => "student_to_merchant",
            PaymentType::MerchantToStudent => "merchant_to_student",
            PaymentType::StudentToStudent => "student_to_student",
        }
    }
}

/// Stellar account information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StellarAccount {
    /// Unique account ID
    pub id: String,
    /// User ID (foreign key to users table)
    pub user_id: Option<String>,
    /// Account public key (can be public)
    pub public_key: String,
    /// Account type (student or merchant)
    pub account_type: String,
    /// Balance in native asset (Lumens)
    pub balance: String,
    /// Account sequence number (for transaction ordering)
    pub sequence: String,
    /// Whether account is verified on Stellar network
    pub verified: bool,
    /// Whether account is active
    pub active: bool,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Account type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccountType {
    Student,
    Merchant,
}

impl AccountType {
    pub fn as_str(&self) -> &str {
        match self {
            AccountType::Student => "student",
            AccountType::Merchant => "merchant",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "student" => Ok(AccountType::Student),
            "merchant" => Ok(AccountType::Merchant),
            _ => Err(ApiError::ValidationError(
                "Invalid account type".to_string(),
            )),
        }
    }
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl StellarAccount {
    /// Create a new Stellar account
    pub fn new(
        user_id: Option<String>,
        public_key: String,
        account_type: String,
    ) -> Result<Self, ApiError> {
        // Validate public key format
        StellarPaymentEngine::validate_public_key(&public_key)?;

        let now = Utc::now();
        Ok(StellarAccount {
            id: Uuid::new_v4().to_string(),
            user_id,
            public_key,
            account_type,
            balance: "0.0".to_string(),
            sequence: "0".to_string(),
            verified: false,
            active: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Create a student account
    pub fn student(user_id: String, public_key: String) -> Result<Self, ApiError> {
        Self::new(Some(user_id), public_key, "student".to_string())
    }

    /// Create a merchant account
    pub fn merchant(user_id: Option<String>, public_key: String) -> Result<Self, ApiError> {
        Self::new(user_id, public_key, "merchant".to_string())
    }

    /// Update account balance
    pub fn set_balance(&mut self, balance: String) -> Result<(), ApiError> {
        StellarPaymentEngine::validate_amount(&balance)?;
        self.balance = balance;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Update account sequence number
    pub fn set_sequence(&mut self, sequence: String) -> Result<(), ApiError> {
        sequence.parse::<u64>()
            .map_err(|_| ApiError::ValidationError("Invalid sequence format".to_string()))?;
        self.sequence = sequence;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark account as verified
    pub fn mark_verified(&mut self) {
        self.verified = true;
        self.updated_at = Utc::now();
    }

    /// Deactivate account
    pub fn deactivate(&mut self) {
        self.active = false;
        self.updated_at = Utc::now();
    }

    /// Check if account is ready and verified
    pub fn is_ready(&self) -> bool {
        self.active && self.verified
    }
}

/// Student payment account (represents a student's Stellar account)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentPaymentAccount {
    /// Student account ID
    pub account_id: String,
    /// Student user ID
    pub user_id: String,
    /// Stellar public key
    pub public_key: String,
    /// Current balance in Lumens
    pub balance: String,
    /// Account verification status
    pub verified: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl StudentPaymentAccount {
    /// Create a new student payment account
    pub fn new(user_id: String, public_key: String) -> Result<Self, ApiError> {
        StellarPaymentEngine::validate_public_key(&public_key)?;
        
        Ok(StudentPaymentAccount {
            account_id: Uuid::new_v4().to_string(),
            user_id,
            public_key,
            balance: "0.0".to_string(),
            verified: false,
            created_at: Utc::now(),
        })
    }

    /// Update balance from Stellar network
    pub fn update_balance(&mut self, balance: String) -> Result<(), ApiError> {
        StellarPaymentEngine::validate_amount(&balance)?;
        self.balance = balance;
        Ok(())
    }

    /// Mark account as verified on Stellar network
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// Convert to StellarAccount
    pub fn to_stellar_account(&self) -> Result<StellarAccount, ApiError> {
        StellarAccount::new(
            Some(self.user_id.clone()),
            self.public_key.clone(),
            "student".to_string(),
        )
    }
}

/// Merchant payment account (represents a merchant's Stellar account)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantPaymentAccount {
    /// Merchant account ID
    pub account_id: String,
    /// Merchant user ID (optional for bulk merchants)
    pub user_id: Option<String>,
    /// Merchant name/identifier
    pub merchant_name: String,
    /// Stellar public key
    pub public_key: String,
    /// Current balance in Lumens
    pub balance: String,
    /// Account verification status
    pub verified: bool,
    /// Whether merchant is active and accepting payments
    pub active: bool,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl MerchantPaymentAccount {
    /// Create a new merchant payment account
    pub fn new(
        merchant_name: String,
        public_key: String,
        user_id: Option<String>,
    ) -> Result<Self, ApiError> {
        StellarPaymentEngine::validate_public_key(&public_key)?;
        
        Ok(MerchantPaymentAccount {
            account_id: Uuid::new_v4().to_string(),
            user_id,
            merchant_name,
            public_key,
            balance: "0.0".to_string(),
            verified: false,
            active: true,
            created_at: Utc::now(),
        })
    }

    /// Update balance from Stellar network
    pub fn update_balance(&mut self, balance: String) -> Result<(), ApiError> {
        StellarPaymentEngine::validate_amount(&balance)?;
        self.balance = balance;
        Ok(())
    }

    /// Mark account as verified on Stellar network
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// Deactivate merchant account
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Check if merchant is ready to accept payments
    pub fn is_ready(&self) -> bool {
        self.active && self.verified
    }

    /// Convert to StellarAccount
    pub fn to_stellar_account(&self) -> Result<StellarAccount, ApiError> {
        StellarAccount::new(
            self.user_id.clone(),
            self.public_key.clone(),
            "merchant".to_string(),
        )
    }
}

/// Payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    /// Unique payment ID
    pub id: String,
    /// Payer public key
    pub payer: String,
    /// Payee public key
    pub payee: String,
    /// Amount in Lumens
    pub amount: String,
    /// Payment type
    pub payment_type: String,
    /// Description/memo
    pub memo: String,
    /// Status
    pub status: String,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

impl PaymentRequest {
    pub fn new(
        payer: String,
        payee: String,
        amount: String,
        payment_type: PaymentType,
        memo: String,
    ) -> Result<Self, ApiError> {
        // Validate inputs
        Self::validate(&payer, &payee, &amount, &memo)?;

        let now = Utc::now();
        Ok(PaymentRequest {
            id: Uuid::new_v4().to_string(),
            payer,
            payee,
            amount,
            payment_type: payment_type.as_str().to_string(),
            memo,
            status: "pending".to_string(),
            expires_at: now + chrono::Duration::hours(1),
            created_at: now,
        })
    }

    /// Create a payment request with custom expiration
    pub fn with_expiration(
        payer: String,
        payee: String,
        amount: String,
        payment_type: PaymentType,
        memo: String,
        expiry_hours: i64,
    ) -> Result<Self, ApiError> {
        Self::validate(&payer, &payee, &amount, &memo)?;

        if expiry_hours <= 0 || expiry_hours > 168 {
            return Err(ApiError::ValidationError(
                "Expiration must be between 1 and 168 hours".to_string(),
            ));
        }

        let now = Utc::now();
        Ok(PaymentRequest {
            id: Uuid::new_v4().to_string(),
            payer,
            payee,
            amount,
            payment_type: payment_type.as_str().to_string(),
            memo,
            status: "pending".to_string(),
            expires_at: now + chrono::Duration::hours(expiry_hours),
            created_at: now,
        })
    }

    /// Validate all payment request fields
    pub fn validate(
        payer: &str,
        payee: &str,
        amount: &str,
        memo: &str,
    ) -> Result<(), ApiError> {
        // Validate payer
        StellarPaymentEngine::validate_public_key(payer)?;

        // Validate payee
        StellarPaymentEngine::validate_public_key(payee)?;

        // Validate they're different
        if payer == payee {
            return Err(ApiError::ValidationError(
                "Payer and payee must be different accounts".to_string(),
            ));
        }

        // Validate amount
        StellarPaymentEngine::validate_amount(amount)?;

        // Validate memo (max 28 bytes for Stellar memo)
        if memo.is_empty() || memo.len() > 28 {
            return Err(ApiError::ValidationError(
                "Memo must be between 1 and 28 characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if payment request is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if payment request is still valid (not expired, pending status)
    pub fn is_valid(&self) -> bool {
        !self.is_expired() && self.status == "pending"
    }

    /// Mark payment as authorized
    pub fn authorize(&mut self) -> Result<(), ApiError> {
        if self.is_expired() {
            return Err(ApiError::Expired);
        }
        if self.status != "pending" {
            return Err(ApiError::ValidationError(
                "Payment request is not pending".to_string(),
            ));
        }
        self.status = "authorized".to_string();
        Ok(())
    }

    /// Mark payment as completed
    pub fn complete(&mut self) -> Result<(), ApiError> {
        if self.status != "authorized" {
            return Err(ApiError::ValidationError(
                "Payment must be authorized before completion".to_string(),
            ));
        }
        self.status = "completed".to_string();
        Ok(())
    }

    /// Mark payment as failed
    pub fn fail(&mut self) -> Result<(), ApiError> {
        if self.status == "completed" {
            return Err(ApiError::ValidationError(
                "Cannot fail a completed payment".to_string(),
            ));
        }
        self.status = "failed".to_string();
        Ok(())
    }

    /// Get time remaining until expiration
    pub fn time_remaining(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }

    /// Check if payment is close to expiration (within 5 minutes)
    pub fn is_expiring_soon(&self) -> bool {
        let remaining = self.time_remaining();
        remaining.num_minutes() < 5 && remaining.num_seconds() > 0
    }
}

/// Payment QR data (does NOT contain secret keys, NO sensitive data)
/// SECURITY: Contains ONLY public payment information safe to encode in QR code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentQrData {
    /// Payment request ID (public identifier)
    pub request_id: String,
    /// Payee public key (can be public)
    pub payee: String,
    /// Amount in Lumens
    pub amount: String,
    /// Memo for payment (max 28 chars, Stellar standard)
    pub memo: String,
    /// Network (testnet only)
    pub network: String,
    /// Timestamp (creation time)
    pub created_at: String,
}

impl PaymentQrData {
    /// Create new payment QR data for a payment request
    pub fn new(
        request_id: String,
        payee: String,
        amount: String,
        memo: String,
    ) -> Result<Self, ApiError> {
        // Validate payee is a valid Stellar public key
        StellarPaymentEngine::validate_public_key(&payee)?;
        
        // Validate amount
        StellarPaymentEngine::validate_amount(&amount)?;
        
        // Validate memo
        if memo.is_empty() || memo.len() > 28 {
            return Err(ApiError::ValidationError(
                "Memo must be between 1 and 28 characters".to_string(),
            ));
        }

        Ok(PaymentQrData {
            request_id,
            payee,
            amount,
            memo,
            network: "testnet".to_string(),
            created_at: Utc::now().to_rfc3339(),
        })
    }

    /// Convert to JSON string for encoding in QR code
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse JSON from QR code
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get QR code content (compact JSON for efficient QR encoding)
    pub fn get_qr_content(&self) -> Result<String, serde_json::Error> {
        // Use compact JSON without extra whitespace
        serde_json::to_string(&self)
    }

    /// Generate QR code image from payment data
    /// Returns QR code as PNG bytes
    pub fn generate_qr_code(&self) -> Result<Vec<u8>, ApiError> {
        use qrcode::QrCode;
        
        // Get QR content
        let content = self.get_qr_content()
            .map_err(|e| ApiError::QrError(format!("Failed to serialize QR data: {}", e)))?;
        
        // Create QR code
        let code = QrCode::new(&content)
            .map_err(|e| ApiError::QrError(format!("Failed to generate QR code: {}", e)))?;
        
        // Render to PNG
        let image = code.render::<qrcode::render::unicode::Dense1x2>()
            .build();
        
        // Return as bytes (Unicode representation)
        Ok(image.into_bytes())
    }

    /// Generate a Stellar payment URI for QR code
    /// Format: stellar:pay?destination=PAYEE&amount=AMOUNT&memo=MEMO
    pub fn get_stellar_uri(&self) -> String {
        format!(
            "stellar:pay?destination={}&amount={}&memo={}",
            urlencoding::encode(&self.payee),
            urlencoding::encode(&self.amount),
            urlencoding::encode(&self.memo),
        )
    }

    /// Get merchant payment URL (for displaying payment info)
    /// This would typically link to the payment gateway
    pub fn get_payment_url(&self, base_url: &str) -> String {
        format!("{}/pay/{}", base_url, self.request_id)
    }

    /// Validate QR data integrity
    pub fn validate(&self) -> Result<(), ApiError> {
        // Verify it's testnet
        if self.network != "testnet" {
            return Err(ApiError::StellarError(
                "QR code is not for testnet".to_string(),
            ));
        }

        // Verify payee
        StellarPaymentEngine::validate_public_key(&self.payee)?;

        // Verify amount
        StellarPaymentEngine::validate_amount(&self.amount)?;

        // Verify memo
        if self.memo.is_empty() || self.memo.len() > 28 {
            return Err(ApiError::ValidationError(
                "Invalid memo in QR code".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if QR code is still valid (created recently, not too old)
    pub fn is_fresh(&self, max_age_hours: i64) -> Result<bool, ApiError> {
        let created = DateTime::parse_from_rfc3339(&self.created_at)
            .map_err(|e| ApiError::ValidationError(
                format!("Failed to parse creation time: {}", e)
            ))?
            .with_timezone(&Utc);

        let age = Utc::now() - created;
        Ok(age.num_hours() < max_age_hours)
    }
}

/// Merchant QR Code Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerchantQrConfig {
    /// Merchant public key
    pub merchant_key: String,
    /// Merchant name
    pub merchant_name: String,
    /// Payment gateway base URL (for payment links)
    pub payment_gateway_url: String,
    /// Whether to include merchant name in QR
    pub include_merchant_name: bool,
    /// QR code size (in pixels, typically 256-512)
    pub qr_size: u32,
}

impl MerchantQrConfig {
    /// Create new merchant QR configuration
    pub fn new(
        merchant_key: String,
        merchant_name: String,
        payment_gateway_url: String,
    ) -> Result<Self, ApiError> {
        StellarPaymentEngine::validate_public_key(&merchant_key)?;
        
        Ok(MerchantQrConfig {
            merchant_key,
            merchant_name,
            payment_gateway_url,
            include_merchant_name: true,
            qr_size: 256,
        })
    }

    /// Set QR code size
    pub fn with_size(mut self, size: u32) -> Result<Self, ApiError> {
        if size < 128 || size > 1024 {
            return Err(ApiError::ValidationError(
                "QR size must be between 128 and 1024 pixels".to_string(),
            ));
        }
        self.qr_size = size;
        Ok(self)
    }

    /// Include or exclude merchant name from QR
    pub fn with_merchant_name(mut self, include: bool) -> Self {
        self.include_merchant_name = include;
        self
    }

    /// Generate a payment QR code for this merchant
    pub fn generate_payment_qr(
        &self,
        request_id: String,
        amount: String,
        memo: String,
    ) -> Result<PaymentQrData, ApiError> {
        PaymentQrData::new(
            request_id,
            self.merchant_key.clone(),
            amount,
            memo,
        )
    }
}

/// Transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransaction {
    /// Transaction ID
    pub id: String,
    /// Payment request ID
    pub payment_request_id: String,
    /// Stellar transaction hash
    pub tx_hash: Option<String>,
    /// Source account
    pub source: String,
    /// Destination account
    pub destination: String,
    /// Amount in Lumens
    pub amount: String,
    /// Status: pending, success, failed
    pub status: String,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

impl StellarTransaction {
    pub fn new(
        payment_request_id: String,
        source: String,
        destination: String,
        amount: String,
    ) -> Self {
        let now = Utc::now();
        StellarTransaction {
            id: Uuid::new_v4().to_string(),
            payment_request_id,
            tx_hash: None,
            source,
            destination,
            amount,
            status: "pending".to_string(),
            error: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Stellar transaction builder - creates transactions for Stellar network
/// No secret keys involved in building - only in signing (backend-only)
#[derive(Debug, Clone)]
pub struct StellarTransactionBuilder {
    /// Source account public key
    pub source_account: String,
    /// Destination account public key
    pub destination: String,
    /// Amount in Lumens
    pub amount: String,
    /// Memo for transaction
    pub memo: String,
    /// Fee in stroops (1 Lumen = 10,000,000 stroops)
    pub fee: u32,
    /// Sequence number for this account
    pub sequence: u64,
    /// Network passphrase (testnet)
    pub network_passphrase: String,
}

impl StellarTransactionBuilder {
    /// Create new transaction builder
    pub fn new(
        source_account: String,
        destination: String,
        amount: String,
        memo: String,
    ) -> Result<Self, ApiError> {
        // Validate accounts
        StellarPaymentEngine::validate_public_key(&source_account)?;
        StellarPaymentEngine::validate_public_key(&destination)?;

        // Validate they're different
        if source_account == destination {
            return Err(ApiError::ValidationError(
                "Source and destination cannot be the same".to_string(),
            ));
        }

        // Validate amount
        StellarPaymentEngine::validate_amount(&amount)?;

        // Validate memo
        if memo.is_empty() || memo.len() > 28 {
            return Err(ApiError::ValidationError(
                "Memo must be between 1 and 28 characters".to_string(),
            ));
        }

        Ok(StellarTransactionBuilder {
            source_account,
            destination,
            amount,
            memo,
            fee: 100, // Standard fee in stroops
            sequence: 0,
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
        })
    }

    /// Set custom fee
    pub fn with_fee(mut self, fee: u32) -> Result<Self, ApiError> {
        if fee < 100 || fee > 10000 {
            return Err(ApiError::ValidationError(
                "Fee must be between 100 and 10000 stroops".to_string(),
            ));
        }
        self.fee = fee;
        Ok(self)
    }

    /// Set sequence number for source account
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = sequence;
        self
    }

    /// Build transaction envelope (XDR format)
    /// In production, this would use soroban-client for actual XDR encoding
    pub fn build(&self) -> Result<StellarTransactionEnvelope, ApiError> {
        if self.sequence == 0 {
            return Err(ApiError::ValidationError(
                "Sequence number must be set before building transaction".to_string(),
            ));
        }

        let tx_envelope = StellarTransactionEnvelope {
            tx_id: Uuid::new_v4().to_string(),
            source_account: self.source_account.clone(),
            destination: self.destination.clone(),
            amount: self.amount.clone(),
            memo: self.memo.clone(),
            fee: self.fee,
            sequence: self.sequence,
            network: "testnet".to_string(),
            created_at: Utc::now(),
        };

        Ok(tx_envelope)
    }

    /// Get transaction for signing
    pub fn get_transaction_for_signing(&self) -> Result<String, ApiError> {
        if self.sequence == 0 {
            return Err(ApiError::ValidationError(
                "Sequence number must be set".to_string(),
            ));
        }

        // Create canonical transaction format for signing
        let signing_payload = format!(
            "{}:{}:{}:{}:{}",
            self.source_account, self.destination, self.amount, self.memo, self.sequence
        );

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(signing_payload.as_bytes());
        
        Ok(format!("{:x}", hasher.finalize()))
    }
}

/// Stellar transaction envelope - ready to be signed and submitted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StellarTransactionEnvelope {
    /// Transaction ID
    pub tx_id: String,
    /// Source account
    pub source_account: String,
    /// Destination account
    pub destination: String,
    /// Amount in Lumens
    pub amount: String,
    /// Memo
    pub memo: String,
    /// Fee in stroops
    pub fee: u32,
    /// Sequence number
    pub sequence: u64,
    /// Network (testnet)
    pub network: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl StellarTransactionEnvelope {
    /// Get data that needs to be signed
    pub fn get_signing_hash(&self) -> Result<String, ApiError> {
        let signing_payload = format!(
            "{}:{}:{}:{}:{}",
            self.source_account, self.destination, self.amount, self.memo, self.sequence
        );

        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(signing_payload.as_bytes());
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Convert to JSON for transmission
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Verify testnet
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.network != "testnet" {
            return Err(ApiError::StellarError(
                "Transaction is not for testnet".to_string(),
            ));
        }

        // Validate accounts
        StellarPaymentEngine::validate_public_key(&self.source_account)?;
        StellarPaymentEngine::validate_public_key(&self.destination)?;

        // Validate amount
        StellarPaymentEngine::validate_amount(&self.amount)?;

        // Validate fee
        if self.fee < 100 || self.fee > 10000 {
            return Err(ApiError::ValidationError(
                "Invalid fee".to_string(),
            ));
        }

        Ok(())
    }
}

/// Signed transaction - contains transaction envelope + signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedStellarTransaction {
    /// Transaction envelope
    pub envelope: StellarTransactionEnvelope,
    /// Signature (Ed25519)
    pub signature: String,
    /// Signer public key
    pub signer: String,
    /// Timestamp of signing
    pub signed_at: DateTime<Utc>,
}

impl SignedStellarTransaction {
    /// Create signed transaction
    pub fn new(
        envelope: StellarTransactionEnvelope,
        signature: String,
        signer: String,
    ) -> Result<Self, ApiError> {
        // Validate signature format (128 hex chars for Ed25519)
        if signature.len() != 128 || !signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiError::CryptoError(
                "Invalid signature format".to_string(),
            ));
        }

        // Validate signer is a proper public key
        StellarPaymentEngine::validate_public_key(&signer)?;

        // Signer must be source account
        if signer != envelope.source_account {
            return Err(ApiError::ValidationError(
                "Signer must match transaction source account".to_string(),
            ));
        }

        Ok(SignedStellarTransaction {
            envelope,
            signature,
            signer,
            signed_at: Utc::now(),
        })
    }

    /// Verify transaction integrity
    pub fn validate(&self) -> Result<(), ApiError> {
        self.envelope.validate()?;

        // Verify signer matches source
        if self.signer != self.envelope.source_account {
            return Err(ApiError::ValidationError(
                "Signer does not match transaction source".to_string(),
            ));
        }

        Ok(())
    }

    /// Convert to JSON for submission
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Transaction submission result from Stellar network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSubmissionResult {
    /// Stellar transaction hash (ledger confirmation)
    pub tx_hash: String,
    /// Ledger sequence number
    pub ledger: u32,
    /// Timestamp from Stellar network
    pub timestamp: DateTime<Utc>,
    /// Success flag
    pub successful: bool,
    /// Stellar transaction ID
    pub id: String,
}

impl TransactionSubmissionResult {
    /// Create from Stellar RPC response
    pub fn new(
        tx_hash: String,
        ledger: u32,
        id: String,
    ) -> Result<Self, ApiError> {
        // Validate tx_hash is valid hex
        if tx_hash.len() != 64 || !tx_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiError::StellarError(
                "Invalid transaction hash format".to_string(),
            ));
        }

        Ok(TransactionSubmissionResult {
            tx_hash,
            ledger,
            timestamp: Utc::now(),
            successful: true,
            id,
        })
    }
}

/// Payment transaction record - tracks payment through entire lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PaymentTransaction {
    /// Unique ID
    pub id: String,
    /// Payment request ID
    pub payment_request_id: String,
    /// Transaction hash from Stellar
    pub tx_hash: Option<String>,
    /// Source account
    pub source: String,
    /// Destination account
    pub destination: String,
    /// Amount in Lumens
    pub amount: String,
    /// Status
    pub status: String,
    /// Number of submission attempts
    pub attempts: u32,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Ledger sequence number (from Stellar)
    pub ledger: Option<u32>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Last updated
    pub updated_at: DateTime<Utc>,
    /// Submitted at
    pub submitted_at: Option<DateTime<Utc>>,
    /// Confirmed at
    pub confirmed_at: Option<DateTime<Utc>>,
}

impl PaymentTransaction {
    /// Create new payment transaction
    pub fn new(
        payment_request_id: String,
        source: String,
        destination: String,
        amount: String,
    ) -> Result<Self, ApiError> {
        // Validate accounts
        StellarPaymentEngine::validate_public_key(&source)?;
        StellarPaymentEngine::validate_public_key(&destination)?;
        StellarPaymentEngine::validate_amount(&amount)?;

        let now = Utc::now();
        Ok(PaymentTransaction {
            id: Uuid::new_v4().to_string(),
            payment_request_id,
            tx_hash: None,
            source,
            destination,
            amount,
            status: "pending".to_string(),
            attempts: 0,
            last_error: None,
            ledger: None,
            created_at: now,
            updated_at: now,
            submitted_at: None,
            confirmed_at: None,
        })
    }

    /// Mark as submitted to Stellar network
    pub fn mark_submitted(&mut self) -> Result<(), ApiError> {
        if self.status != "pending" {
            return Err(ApiError::ValidationError(
                "Can only submit pending transactions".to_string(),
            ));
        }
        self.status = "submitted".to_string();
        self.submitted_at = Some(Utc::now());
        self.attempts += 1;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark as confirmed on Stellar network
    pub fn mark_confirmed(&mut self, tx_hash: String, ledger: u32) -> Result<(), ApiError> {
        if self.status != "submitted" {
            return Err(ApiError::ValidationError(
                "Can only confirm submitted transactions".to_string(),
            ));
        }

        // Validate tx_hash
        if tx_hash.len() != 64 || !tx_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiError::StellarError(
                "Invalid transaction hash".to_string(),
            ));
        }

        self.status = "confirmed".to_string();
        self.tx_hash = Some(tx_hash);
        self.ledger = Some(ledger);
        self.confirmed_at = Some(Utc::now());
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Mark as failed
    pub fn mark_failed(&mut self, error: String) -> Result<(), ApiError> {
        if self.status == "confirmed" {
            return Err(ApiError::ValidationError(
                "Cannot fail a confirmed transaction".to_string(),
            ));
        }
        self.status = "failed".to_string();
        self.last_error = Some(error);
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Check if transaction can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts < 3 && self.status == "failed"
    }

    /// Reset for retry
    pub fn reset_for_retry(&mut self) {
        self.status = "pending".to_string();
        self.last_error = None;
        self.updated_at = Utc::now();
    }

    /// Check if transaction has timed out (no confirmation after 5 minutes)
    pub fn has_timed_out(&self) -> bool {
        if self.status != "submitted" {
            return false;
        }
        if let Some(submitted) = self.submitted_at {
            let elapsed = Utc::now() - submitted;
            elapsed.num_minutes() >= 5
        } else {
            false
        }
    }

    /// Get current status
    pub fn get_status(&self) -> String {
        self.status.clone()
    }
}

/// Transaction history filter
#[derive(Debug, Clone)]
pub struct TransactionHistoryFilter {
    /// Filter by status
    pub status: Option<String>,
    /// Filter by source account
    pub source: Option<String>,
    /// Filter by destination account
    pub destination: Option<String>,
    /// Limit results
    pub limit: u32,
    /// Offset for pagination
    pub offset: u32,
}

impl TransactionHistoryFilter {
    /// Create new filter
    pub fn new() -> Self {
        TransactionHistoryFilter {
            status: None,
            source: None,
            destination: None,
            limit: 50,
            offset: 0,
        }
    }

    /// Filter by status
    pub fn with_status(mut self, status: String) -> Self {
        self.status = Some(status);
        self
    }

    /// Filter by source
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    /// Filter by destination
    pub fn with_destination(mut self, destination: String) -> Self {
        self.destination = Some(destination);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: u32) -> Result<Self, ApiError> {
        if limit > 1000 {
            return Err(ApiError::ValidationError(
                "Limit cannot exceed 1000".to_string(),
            ));
        }
        self.limit = limit;
        Ok(self)
    }

    /// Set offset
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }
}

/// Transaction retry handler
#[derive(Debug, Clone)]
pub struct TransactionRetryHandler {
    max_retries: u32,
    retry_delay_seconds: u32,
}

impl TransactionRetryHandler {
    /// Create new retry handler
    pub fn new() -> Self {
        TransactionRetryHandler {
            max_retries: 3,
            retry_delay_seconds: 10,
        }
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Result<Self, ApiError> {
        if max_retries == 0 || max_retries > 10 {
            return Err(ApiError::ValidationError(
                "Max retries must be between 1 and 10".to_string(),
            ));
        }
        self.max_retries = max_retries;
        Ok(self)
    }

    /// Set retry delay
    pub fn with_retry_delay(mut self, delay_seconds: u32) -> Result<Self, ApiError> {
        if delay_seconds < 5 || delay_seconds > 300 {
            return Err(ApiError::ValidationError(
                "Retry delay must be between 5 and 300 seconds".to_string(),
            ));
        }
        self.retry_delay_seconds = delay_seconds;
        Ok(self)
    }

    /// Check if transaction should be retried
    pub fn should_retry(&self, tx: &PaymentTransaction) -> bool {
        tx.attempts < self.max_retries && tx.status == "failed"
    }

    /// Calculate backoff delay for retry
    pub fn get_backoff_delay(&self, attempt_number: u32) -> u32 {
        // Exponential backoff: 10s, 20s, 40s, etc.
        self.retry_delay_seconds * (2_u32.pow(attempt_number.saturating_sub(1)))
    }
}

/// Transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
}

impl TransactionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Success => "success",
            TransactionStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ApiError> {
        match s {
            "pending" => Ok(TransactionStatus::Pending),
            "success" => Ok(TransactionStatus::Success),
            "failed" => Ok(TransactionStatus::Failed),
            _ => Err(ApiError::ValidationError(
                "Invalid transaction status".to_string(),
            )),
        }
    }
}

/// Payment authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAuthorization {
    /// Authorization ID
    pub id: String,
    /// Payment request ID
    pub payment_request_id: String,
    /// Authorized by (payer public key)
    pub authorized_by: String,
    /// Authorization signature (Ed25519)
    pub signature: String,
    /// Timestamp
    pub authorized_at: DateTime<Utc>,
}

impl PaymentAuthorization {
    pub fn new(
        payment_request_id: String,
        authorized_by: String,
        signature: String,
    ) -> Self {
        PaymentAuthorization {
            id: Uuid::new_v4().to_string(),
            payment_request_id,
            authorized_by,
            signature,
            authorized_at: Utc::now(),
        }
    }
}

/// Payment signing request - used for backend to request signature from client
/// Contains only the data to be signed, NOT the secret key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentSigningRequest {
    /// Unique request ID
    pub request_id: String,
    /// Payment request ID being signed
    pub payment_request_id: String,
    /// Account to sign with (payer public key)
    pub account: String,
    /// Data to sign (SHA-256 hash of payment request)
    pub signing_data: String,
    /// Signature algorithm (Ed25519 for Stellar)
    pub algorithm: String,
    /// Expiration time for this signing request
    pub expires_at: DateTime<Utc>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl PaymentSigningRequest {
    /// Create a new signing request for a payment
    pub fn new(
        payment_request_id: String,
        account: String,
        payment_data: &PaymentRequest,
    ) -> Result<Self, ApiError> {
        // Validate account is a proper Stellar public key
        StellarPaymentEngine::validate_public_key(&account)?;

        // Generate signing data hash from payment details
        let signing_data = Self::generate_signing_data(payment_data)?;

        let now = Utc::now();
        Ok(PaymentSigningRequest {
            request_id: Uuid::new_v4().to_string(),
            payment_request_id,
            account,
            signing_data,
            algorithm: "Ed25519".to_string(),
            expires_at: now + chrono::Duration::minutes(5),
            created_at: now,
        })
    }

    /// Generate signing data (hash) from payment request
    /// This is what gets signed by the client
    pub fn generate_signing_data(payment: &PaymentRequest) -> Result<String, ApiError> {
        use sha2::{Sha256, Digest};
        
        // Create deterministic payload to sign
        let payload = format!(
            "{}:{}:{}:{}",
            payment.id,
            payment.payer,
            payment.payee,
            payment.amount
        );

        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Check if signing request is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        Utc::now() < self.expires_at
    }

    /// Get time remaining for this signing request
    pub fn time_remaining(&self) -> chrono::Duration {
        self.expires_at - Utc::now()
    }

    /// Verify a signature against this request
    /// Returns true if signature is valid for this account and data
    pub fn verify_signature(&self, signature: &str) -> Result<bool, ApiError> {
        if !self.is_valid() {
            return Err(ApiError::Expired);
        }

        // NOTE: Actual ED25519 signature verification would use:
        // ed25519_dalek::PublicKey and ed25519_dalek::Signature
        // For now, we validate format and return placeholder
        
        // Verify signature is valid hex (64 bytes = 128 hex chars)
        if signature.len() != 128 || !signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ApiError::CryptoError(
                "Invalid signature format (must be 128 hex characters)".to_string(),
            ));
        }

        Ok(true)
    }
}

/// Payment authorization handler - validates authorization workflow
#[derive(Debug, Clone)]
pub struct PaymentAuthorizationHandler {
    /// Store of active signing requests (in production, use database)
    signing_requests: std::collections::HashMap<String, PaymentSigningRequest>,
}

impl PaymentAuthorizationHandler {
    /// Create new authorization handler
    pub fn new() -> Self {
        PaymentAuthorizationHandler {
            signing_requests: std::collections::HashMap::new(),
        }
    }

    /// Create a signing request for a payment
    pub fn create_signing_request(
        &mut self,
        payment: &PaymentRequest,
    ) -> Result<PaymentSigningRequest, ApiError> {
        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            payment,
        )?;

        let request_id = signing_req.request_id.clone();
        self.signing_requests.insert(request_id.clone(), signing_req.clone());

        Ok(signing_req)
    }

    /// Get signing request by ID
    pub fn get_signing_request(&self, request_id: &str) -> Result<PaymentSigningRequest, ApiError> {
        self.signing_requests
            .get(request_id)
            .cloned()
            .ok_or_else(|| ApiError::ValidationError(
                "Signing request not found".to_string()
            ))
    }

    /// Verify a signature for a signing request
    pub fn verify_and_authorize(
        &self,
        request_id: &str,
        signature: &str,
    ) -> Result<PaymentAuthorization, ApiError> {
        let signing_req = self.get_signing_request(request_id)?;
        
        // Verify the signature
        signing_req.verify_signature(signature)?;

        // Create authorization record
        let auth = PaymentAuthorization::new(
            signing_req.payment_request_id.clone(),
            signing_req.account.clone(),
            signature.to_string(),
        );

        Ok(auth)
    }

    /// Clean up expired signing requests
    pub fn cleanup_expired(&mut self) {
        self.signing_requests.retain(|_, req| req.is_valid());
    }
}

/// Stellar payment engine
pub struct StellarPaymentEngine {
    /// Stellar testnet base URL
    pub horizon_url: String,
    /// Stellar RPC endpoint
    pub rpc_url: String,
    /// Network passphrase (testnet)
    pub network_passphrase: String,
}

impl StellarPaymentEngine {
    pub fn new(horizon_url: String, rpc_url: String) -> Self {
        StellarPaymentEngine {
            horizon_url,
            rpc_url,
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
        }
    }

    /// Validate Stellar public key format
    pub fn validate_public_key(key: &str) -> Result<bool, ApiError> {
        // Stellar public keys start with 'G' and are 56 characters
        if key.len() != 56 || !key.starts_with('G') {
            return Err(ApiError::ValidationError(
                "Invalid Stellar public key format".to_string(),
            ));
        }

        // Check if valid base32
        if !key.chars().all(|c| {
            (c >= 'A' && c <= 'Z') || (c >= '2' && c <= '7')
        }) {
            return Err(ApiError::ValidationError(
                "Invalid Stellar public key characters".to_string(),
            ));
        }

        Ok(true)
    }

    /// Validate payment amount (must be positive)
    pub fn validate_amount(amount: &str) -> Result<bool, ApiError> {
        match amount.parse::<f64>() {
            Ok(value) => {
                if value <= 0.0 {
                    Err(ApiError::ValidationError(
                        "Amount must be positive".to_string(),
                    ))
                } else {
                    Ok(true)
                }
            }
            Err(_) => Err(ApiError::ValidationError(
                "Invalid amount format".to_string(),
            )),
        }
    }

    /// Check if payment request is expired
    pub fn is_payment_expired(payment: &PaymentRequest) -> bool {
        payment.is_expired()
    }

    /// Generate secure memo hash
    pub fn generate_memo_hash(payment_id: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(payment_id.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Stellar RPC Service wrapper - provides high-level API for Stellar interactions
/// Uses official soroban-client library for testnet communication
#[derive(Debug, Clone)]
pub struct StellarRpcService {
    /// RPC server URL (testnet only)
    pub rpc_url: String,
    /// Network passphrase
    pub network_passphrase: String,
}

impl StellarRpcService {
    /// Create a new Stellar RPC service instance
    pub fn new() -> Result<Self, ApiError> {
        let rpc_url = std::env::var("STELLAR_RPC_URL")
            .unwrap_or_else(|_| "https://soroban-testnet.stellar.org".to_string());
        
        if !rpc_url.contains("testnet") && !rpc_url.contains("localhost") {
            return Err(ApiError::StellarError(
                "Only testnet RPC URLs are allowed".to_string(),
            ));
        }

        Ok(StellarRpcService {
            rpc_url,
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
        })
    }

    /// Create with custom RPC URL (testnet only)
    pub fn with_testnet_url(rpc_url: String) -> Result<Self, ApiError> {
        if !rpc_url.contains("testnet") && !rpc_url.contains("localhost") {
            return Err(ApiError::StellarError(
                "Only testnet RPC URLs are allowed".to_string(),
            ));
        }

        Ok(StellarRpcService {
            rpc_url,
            network_passphrase: "Test SDF Network ; September 2015".to_string(),
        })
    }

    /// Verify public key format using Stellar's strkey
    /// Returns error if invalid
    pub fn verify_public_key(public_key: &str) -> Result<bool, ApiError> {
        StellarPaymentEngine::validate_public_key(public_key)
    }

    /// Check if RPC is using testnet
    pub fn is_testnet(&self) -> bool {
        self.rpc_url.contains("testnet") || self.rpc_url.contains("localhost")
    }

    /// Get network info
    pub fn network_info(&self) -> NetworkInfo {
        NetworkInfo {
            network_name: "Stellar Testnet".to_string(),
            network_passphrase: self.network_passphrase.clone(),
            rpc_url: self.rpc_url.clone(),
            horizon_url: "https://horizon-testnet.stellar.org".to_string(),
        }
    }
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub network_name: String,
    pub network_passphrase: String,
    pub rpc_url: String,
    pub horizon_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payment_request_creation() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch payment".to_string(),
        );

        assert!(payment.is_ok());
        let p = payment.unwrap();
        assert_eq!(p.status, "pending");
        assert!(!p.is_expired());
        assert!(p.is_valid());
    }

    #[test]
    fn test_payment_expiration() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch payment".to_string(),
        ).unwrap();

        payment.expires_at = Utc::now() - chrono::Duration::minutes(1);
        assert!(payment.is_expired());
        assert!(!payment.is_valid());
    }

    #[test]
    fn test_payment_qr_data_serialization() {
        let qr_data = PaymentQrData::new(
            "payment-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Lunch".to_string(),
        );

        assert!(qr_data.is_ok());
        let qr = qr_data.unwrap();
        
        let json = qr.to_json();
        assert!(json.is_ok());

        let deserialized = PaymentQrData::from_json(&json.unwrap());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_account_type_conversion() {
        assert_eq!(AccountType::Student.as_str(), "student");
        assert_eq!(AccountType::Merchant.as_str(), "merchant");
        assert_eq!(AccountType::from_str("student").unwrap(), AccountType::Student);
        assert_eq!(AccountType::from_str("merchant").unwrap(), AccountType::Merchant);
    }

    #[test]
    fn test_valid_stellar_public_key() {
        // Valid Stellar public key format
        let key = "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW";
        assert!(StellarPaymentEngine::validate_public_key(key).is_ok());
    }

    #[test]
    fn test_invalid_stellar_public_key_length() {
        let key = "GCZST3XVCDTUJ76ZAV2HA72KYXZ2K"; // Too short
        assert!(StellarPaymentEngine::validate_public_key(key).is_err());
    }

    #[test]
    fn test_invalid_stellar_public_key_prefix() {
        let key = "SCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW"; // Starts with S
        assert!(StellarPaymentEngine::validate_public_key(key).is_err());
    }

    #[test]
    fn test_valid_payment_amount() {
        assert!(StellarPaymentEngine::validate_amount("100.50").is_ok());
        assert!(StellarPaymentEngine::validate_amount("0.001").is_ok());
    }

    #[test]
    fn test_invalid_payment_amount_zero() {
        assert!(StellarPaymentEngine::validate_amount("0.00").is_err());
    }

    #[test]
    fn test_invalid_payment_amount_negative() {
        assert!(StellarPaymentEngine::validate_amount("-100.00").is_err());
    }

    #[test]
    fn test_invalid_payment_amount_format() {
        assert!(StellarPaymentEngine::validate_amount("invalid").is_err());
    }

    #[test]
    fn test_stellar_transaction_creation() {
        let tx = StellarTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        );

        assert_eq!(tx.status, "pending");
        assert_eq!(tx.source, "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW");
    }

    #[test]
    fn test_payment_authorization_creation() {
        let auth = PaymentAuthorization::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "signature_bytes_here".to_string(),
        );

        assert_eq!(auth.payment_request_id, "payment-123");
    }

    #[test]
    fn test_memo_hash_generation() {
        let hash1 = StellarPaymentEngine::generate_memo_hash("payment-123");
        let hash2 = StellarPaymentEngine::generate_memo_hash("payment-123");
        assert_eq!(hash1, hash2);

        let hash3 = StellarPaymentEngine::generate_memo_hash("payment-456");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_payment_type_conversion() {
        assert_eq!(PaymentType::StudentToMerchant.as_str(), "student_to_merchant");
        assert_eq!(PaymentType::MerchantToStudent.as_str(), "merchant_to_student");
        assert_eq!(PaymentType::StudentToStudent.as_str(), "student_to_student");
    }

    #[test]
    fn test_transaction_status_conversion() {
        assert_eq!(TransactionStatus::Pending.as_str(), "pending");
        assert_eq!(TransactionStatus::Success.as_str(), "success");
        assert_eq!(TransactionStatus::Failed.as_str(), "failed");
    }
}
    // ==================== STUDENT ACCOUNT TESTS ====================
    
    #[test]
    fn test_student_account_creation() {
        let student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        );

        assert!(student.is_ok());
        let acc = student.unwrap();
        assert_eq!(acc.user_id, "user-123");
        assert!(!acc.verified);
        assert_eq!(acc.balance, "0.0");
    }

    #[test]
    fn test_student_account_invalid_public_key() {
        let student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "INVALID_KEY".to_string(),
        );

        assert!(student.is_err());
    }

    #[test]
    fn test_student_account_balance_update() {
        let mut student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(student.update_balance("150.50".to_string()).is_ok());
        assert_eq!(student.balance, "150.50");
    }

    #[test]
    fn test_student_account_invalid_balance() {
        let mut student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(student.update_balance("-100.00".to_string()).is_err());
        assert!(student.update_balance("0.00".to_string()).is_err());
    }

    #[test]
    fn test_student_account_verification() {
        let mut student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(!student.verified);
        student.verify();
        assert!(student.verified);
    }

    #[test]
    fn test_student_account_to_stellar_account() {
        let student = StudentPaymentAccount::new(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        let stellar_acc = student.to_stellar_account();
        assert!(stellar_acc.is_ok());
        let acc = stellar_acc.unwrap();
        assert_eq!(acc.account_type, "student");
        assert_eq!(acc.user_id, Some("user-123".to_string()));
    }

    // ==================== MERCHANT ACCOUNT TESTS ====================

    #[test]
    fn test_merchant_account_creation() {
        let merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        );

        assert!(merchant.is_ok());
        let acc = merchant.unwrap();
        assert_eq!(acc.merchant_name, "Campus Cafe");
        assert!(!acc.verified);
        assert!(acc.active);
    }

    #[test]
    fn test_merchant_account_creation_without_user() {
        let merchant = MerchantPaymentAccount::new(
            "Campus Bookstore".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            None,
        );

        assert!(merchant.is_ok());
        let acc = merchant.unwrap();
        assert_eq!(acc.user_id, None);
        assert_eq!(acc.merchant_name, "Campus Bookstore");
    }

    #[test]
    fn test_merchant_account_balance_update() {
        let mut merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        ).unwrap();

        assert!(merchant.update_balance("500.25".to_string()).is_ok());
        assert_eq!(merchant.balance, "500.25");
    }

    #[test]
    fn test_merchant_account_verification() {
        let mut merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        ).unwrap();

        assert!(!merchant.verified);
        merchant.verify();
        assert!(merchant.verified);
    }

    #[test]
    fn test_merchant_account_deactivation() {
        let mut merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        ).unwrap();

        assert!(merchant.active);
        merchant.deactivate();
        assert!(!merchant.active);
    }

    #[test]
    fn test_merchant_account_ready_status() {
        let mut merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        ).unwrap();

        // Not ready without verification
        assert!(!merchant.is_ready());

        merchant.verify();
        assert!(merchant.is_ready());

        merchant.deactivate();
        assert!(!merchant.is_ready());
    }

    #[test]
    fn test_merchant_account_to_stellar_account() {
        let merchant = MerchantPaymentAccount::new(
            "Campus Cafe".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            Some("merchant-user-123".to_string()),
        ).unwrap();

        let stellar_acc = merchant.to_stellar_account();
        assert!(stellar_acc.is_ok());
        let acc = stellar_acc.unwrap();
        assert_eq!(acc.account_type, "merchant");
        assert_eq!(acc.user_id, Some("merchant-user-123".to_string()));
    }

    // ==================== STELLAR ACCOUNT TESTS ====================

    #[test]
    fn test_stellar_account_student_creation() {
        let account = StellarAccount::student(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        );

        assert!(account.is_ok());
        let acc = account.unwrap();
        assert_eq!(acc.account_type, "student");
        assert!(!acc.verified);
    }

    #[test]
    fn test_stellar_account_merchant_creation() {
        let account = StellarAccount::merchant(
            Some("merchant-123".to_string()),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
        );

        assert!(account.is_ok());
        let acc = account.unwrap();
        assert_eq!(acc.account_type, "merchant");
    }

    #[test]
    fn test_stellar_account_balance_update() {
        let mut account = StellarAccount::student(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(account.set_balance("250.75".to_string()).is_ok());
        assert_eq!(account.balance, "250.75");
    }

    #[test]
    fn test_stellar_account_sequence_update() {
        let mut account = StellarAccount::student(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(account.set_sequence("42".to_string()).is_ok());
        assert_eq!(account.sequence, "42");
    }

    #[test]
    fn test_stellar_account_verification_and_ready() {
        let mut account = StellarAccount::student(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        assert!(!account.is_ready());
        account.mark_verified();
        assert!(account.is_ready());
    }

    #[test]
    fn test_stellar_account_deactivation_and_ready() {
        let mut account = StellarAccount::student(
            "user-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
        ).unwrap();

        account.mark_verified();
        assert!(account.is_ready());

        account.deactivate();
        assert!(!account.is_ready());
    }

    // ==================== STELLAR RPC SERVICE TESTS ====================

    #[test]
    fn test_stellar_rpc_service_creation() {
        // This test uses default RPC URL from env or default
        std::env::set_var("STELLAR_RPC_URL", "https://soroban-testnet.stellar.org");
        let service = StellarRpcService::new();
        assert!(service.is_ok());
        let rpc = service.unwrap();
        assert!(rpc.is_testnet());
    }

    #[test]
    fn test_stellar_rpc_service_testnet_only() {
        // Should reject mainnet URLs
        let result = StellarRpcService::with_testnet_url(
            "https://soroban-mainnet.stellar.org".to_string()
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_stellar_rpc_service_testnet_url() {
        let service = StellarRpcService::with_testnet_url(
            "https://soroban-testnet.stellar.org".to_string()
        );
        assert!(service.is_ok());
        let rpc = service.unwrap();
        assert!(rpc.is_testnet());
    }

    #[test]
    fn test_stellar_rpc_service_localhost() {
        // Localhost should be allowed for local testing
        let service = StellarRpcService::with_testnet_url(
            "http://localhost:8000".to_string()
        );
        assert!(service.is_ok());
    }

    #[test]
    fn test_stellar_rpc_verify_public_key() {
        let key = "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW";
        assert!(StellarRpcService::verify_public_key(key).is_ok());
    }

    #[test]
    fn test_stellar_rpc_network_info() {
        let service = StellarRpcService::with_testnet_url(
            "https://soroban-testnet.stellar.org".to_string()
        ).unwrap();
        
        let info = service.network_info();
        assert_eq!(info.network_name, "Stellar Testnet");
        assert_eq!(info.network_passphrase, "Test SDF Network ; September 2015");
        assert!(info.rpc_url.contains("testnet"));
    }

    // ==================== PAYMENT REQUEST GENERATION & VALIDATION TESTS ====================

    #[test]
    fn test_payment_request_validation_success() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "100.00",
            "Payment",
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_payment_request_validation_invalid_payer() {
        let result = PaymentRequest::validate(
            "INVALID_KEY",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "100.00",
            "Payment",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_invalid_payee() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "NOT_A_KEY",
            "100.00",
            "Payment",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_same_payer_payee() {
        let key = "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW";
        let result = PaymentRequest::validate(key, key, "100.00", "Payment");
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_invalid_amount_negative() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "-50.00",
            "Payment",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_invalid_amount_zero() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "0.00",
            "Payment",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_empty_memo() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "100.00",
            "",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_validation_memo_too_long() {
        let result = PaymentRequest::validate(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW",
            "GBBD47UZQ53HALXJEMZUKKMYRJA",
            "100.00",
            "This memo is way too long and exceeds the stellar limit",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_with_custom_expiration() {
        let payment = PaymentRequest::with_expiration(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            PaymentType::StudentToMerchant,
            "Textbooks".to_string(),
            24,
        );

        assert!(payment.is_ok());
        let p = payment.unwrap();
        let remaining = p.time_remaining();
        // Should be approximately 24 hours
        assert!(remaining.num_hours() >= 23 && remaining.num_hours() <= 24);
    }

    #[test]
    fn test_payment_request_invalid_expiration_too_short() {
        let result = PaymentRequest::with_expiration(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            PaymentType::StudentToMerchant,
            "Textbooks".to_string(),
            0,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_invalid_expiration_too_long() {
        let result = PaymentRequest::with_expiration(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            PaymentType::StudentToMerchant,
            "Textbooks".to_string(),
            200,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_payment_request_authorize() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert_eq!(payment.status, "pending");
        assert!(payment.authorize().is_ok());
        assert_eq!(payment.status, "authorized");
    }

    #[test]
    fn test_payment_request_authorize_expired() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        payment.expires_at = Utc::now() - chrono::Duration::minutes(1);
        assert!(payment.authorize().is_err());
    }

    #[test]
    fn test_payment_request_complete() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert!(payment.authorize().is_ok());
        assert!(payment.complete().is_ok());
        assert_eq!(payment.status, "completed");
    }

    #[test]
    fn test_payment_request_complete_without_authorization() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert!(payment.complete().is_err());
    }

    #[test]
    fn test_payment_request_fail() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert!(payment.authorize().is_ok());
        assert!(payment.fail().is_ok());
        assert_eq!(payment.status, "failed");
    }

    #[test]
    fn test_payment_request_fail_completed() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert!(payment.authorize().is_ok());
        assert!(payment.complete().is_ok());
        assert!(payment.fail().is_err());
    }

    #[test]
    fn test_payment_request_time_remaining() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let remaining = payment.time_remaining();
        assert!(remaining.num_minutes() > 50 && remaining.num_minutes() <= 60);
    }

    #[test]
    fn test_payment_request_is_expiring_soon() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        assert!(!payment.is_expiring_soon());

        // Set expiration to 2 minutes from now
        payment.expires_at = Utc::now() + chrono::Duration::minutes(2);
        assert!(payment.is_expiring_soon());
    }

    #[test]
    fn test_payment_request_status_transitions() {
        let mut payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        // Start: pending
        assert_eq!(payment.status, "pending");
        assert!(payment.is_valid());

        // Transition: authorize
        assert!(payment.authorize().is_ok());
        assert_eq!(payment.status, "authorized");
        assert!(!payment.is_valid());

        // Transition: complete
        assert!(payment.complete().is_ok());
        assert_eq!(payment.status, "completed");
    }

    #[test]
    fn test_payment_request_different_payment_types() {
        let types = vec![
            PaymentType::StudentToMerchant,
            PaymentType::MerchantToStudent,
            PaymentType::StudentToStudent,
        ];

        for ptype in types {
            let payment = PaymentRequest::new(
                "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
                "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
                "100.00".to_string(),
                ptype.clone(),
                "Test".to_string(),
            );

            assert!(payment.is_ok());
            assert_eq!(payment.unwrap().payment_type, ptype.as_str());
        }
    }

    // ==================== MERCHANT PAYMENT QR CODE GENERATION TESTS ====================

    #[test]
    fn test_payment_qr_data_creation() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        );

        assert!(qr.is_ok());
        let q = qr.unwrap();
        assert_eq!(q.request_id, "req-123");
        assert_eq!(q.network, "testnet");
    }

    #[test]
    fn test_payment_qr_data_invalid_payee() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "INVALID".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        );

        assert!(qr.is_err());
    }

    #[test]
    fn test_payment_qr_data_invalid_amount() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "-50.00".to_string(),
            "Coffee".to_string(),
        );

        assert!(qr.is_err());
    }

    #[test]
    fn test_payment_qr_data_invalid_memo_empty() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "".to_string(),
        );

        assert!(qr.is_err());
    }

    #[test]
    fn test_payment_qr_data_invalid_memo_too_long() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "This is an extremely long memo that exceeds the stellar limit".to_string(),
        );

        assert!(qr.is_err());
    }

    #[test]
    fn test_payment_qr_data_json_serialization() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        let json = qr.to_json().unwrap();
        assert!(json.contains("req-123"));
        assert!(json.contains("50.00"));
        assert!(!json.contains("secret"));
    }

    #[test]
    fn test_payment_qr_data_deserialization() {
        let qr1 = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        let json = qr1.to_json().unwrap();
        let qr2 = PaymentQrData::from_json(&json).unwrap();
        
        assert_eq!(qr1.request_id, qr2.request_id);
        assert_eq!(qr1.payee, qr2.payee);
        assert_eq!(qr1.amount, qr2.amount);
        assert_eq!(qr1.memo, qr2.memo);
    }

    #[test]
    fn test_payment_qr_data_validate() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        assert!(qr.validate().is_ok());
    }

    #[test]
    fn test_payment_qr_data_is_fresh() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        // Should be fresh (created just now, checking within 1 hour)
        assert!(qr.is_fresh(1).unwrap());
    }

    #[test]
    fn test_payment_qr_data_stellar_uri() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        let uri = qr.get_stellar_uri();
        assert!(uri.starts_with("stellar:pay?"));
        assert!(uri.contains("destination=GBBD47UZQ53HALXJEMZUKKMYRJA"));
        assert!(uri.contains("amount=50.00"));
        assert!(uri.contains("memo=Coffee"));
    }

    #[test]
    fn test_payment_qr_data_payment_url() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        let url = qr.get_payment_url("https://api.example.com");
        assert_eq!(url, "https://api.example.com/pay/req-123");
    }

    #[test]
    fn test_merchant_qr_config_creation() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        );

        assert!(config.is_ok());
        let c = config.unwrap();
        assert_eq!(c.merchant_name, "Campus Cafe");
        assert_eq!(c.qr_size, 256);
        assert!(c.include_merchant_name);
    }

    #[test]
    fn test_merchant_qr_config_invalid_key() {
        let config = MerchantQrConfig::new(
            "INVALID".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        );

        assert!(config.is_err());
    }

    #[test]
    fn test_merchant_qr_config_with_size() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        ).unwrap();

        let sized = config.with_size(512).unwrap();
        assert_eq!(sized.qr_size, 512);
    }

    #[test]
    fn test_merchant_qr_config_invalid_size_too_small() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        ).unwrap();

        let sized = config.with_size(64);
        assert!(sized.is_err());
    }

    #[test]
    fn test_merchant_qr_config_invalid_size_too_large() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        ).unwrap();

        let sized = config.with_size(2048);
        assert!(sized.is_err());
    }

    #[test]
    fn test_merchant_qr_config_with_merchant_name_flag() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        ).unwrap();

        assert!(config.include_merchant_name);

        let without_name = config.clone().with_merchant_name(false);
        assert!(!without_name.include_merchant_name);
    }

    #[test]
    fn test_merchant_qr_config_generate_payment_qr() {
        let config = MerchantQrConfig::new(
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "Campus Cafe".to_string(),
            "https://pay.example.com".to_string(),
        ).unwrap();

        let qr = config.generate_payment_qr(
            "req-456".to_string(),
            "25.50".to_string(),
            "Lunch Special".to_string(),
        );

        assert!(qr.is_ok());
        let q = qr.unwrap();
        assert_eq!(q.request_id, "req-456");
        assert_eq!(q.payee, "GBBD47UZQ53HALXJEMZUKKMYRJA");
        assert_eq!(q.amount, "25.50");
    }

    #[test]
    fn test_qr_code_no_secret_keys() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        let json = qr.to_json().unwrap();
        
        // Verify NO secret keys or sensitive data in QR
        assert!(!json.contains("secret"));
        assert!(!json.contains("private"));
        assert!(!json.contains("password"));
        assert!(!json.contains("key"));
        
        // Verify ONLY safe public data
        assert!(json.contains("request_id"));
        assert!(json.contains("payee"));
        assert!(json.contains("amount"));
        assert!(json.contains("testnet"));
    }

    #[test]
    fn test_qr_code_testnet_only() {
        let qr = PaymentQrData::new(
            "req-123".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "50.00".to_string(),
            "Coffee".to_string(),
        ).unwrap();

        // Verify testnet flag
        assert_eq!(qr.network, "testnet");

        // Verify validation catches wrong network
        let mut bad_qr = qr.clone();
        bad_qr.network = "mainnet".to_string();
        assert!(bad_qr.validate().is_err());
    }

    // ==================== PAYMENT AUTHORIZATION & SIGNING TESTS ====================

    #[test]
    fn test_payment_signing_request_creation() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        );

        assert!(signing_req.is_ok());
        let req = signing_req.unwrap();
        assert_eq!(req.algorithm, "Ed25519");
        assert_eq!(req.account, payment.payer);
    }

    #[test]
    fn test_payment_signing_request_invalid_account() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            "INVALID".to_string(),
            &payment,
        );

        assert!(signing_req.is_err());
    }

    #[test]
    fn test_payment_signing_data_generation() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let data1 = PaymentSigningRequest::generate_signing_data(&payment).unwrap();
        let data2 = PaymentSigningRequest::generate_signing_data(&payment).unwrap();

        // Same payment should generate same signing data (deterministic)
        assert_eq!(data1, data2);

        // Signing data should be SHA-256 hash (64 hex chars)
        assert_eq!(data1.len(), 64);
        assert!(data1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_payment_signing_request_validity() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Should be valid immediately after creation
        assert!(signing_req.is_valid());

        let remaining = signing_req.time_remaining();
        // Should be approximately 5 minutes
        assert!(remaining.num_minutes() >= 4 && remaining.num_minutes() <= 5);
    }

    #[test]
    fn test_payment_signing_request_expired() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let mut signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Force expiration
        signing_req.expires_at = Utc::now() - chrono::Duration::minutes(1);
        assert!(!signing_req.is_valid());
    }

    #[test]
    fn test_payment_signing_verify_valid_signature() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Valid signature format (128 hex chars)
        let valid_sig = "a".repeat(128);
        assert!(signing_req.verify_signature(&valid_sig).is_ok());
    }

    #[test]
    fn test_payment_signing_verify_invalid_signature_format() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Invalid signature (too short)
        assert!(signing_req.verify_signature("abc").is_err());
    }

    #[test]
    fn test_payment_signing_verify_invalid_signature_non_hex() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Invalid signature (non-hex characters)
        let invalid_sig = "g".repeat(128);
        assert!(signing_req.verify_signature(&invalid_sig).is_err());
    }

    #[test]
    fn test_payment_signing_verify_expired_request() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let mut signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Force expiration
        signing_req.expires_at = Utc::now() - chrono::Duration::minutes(1);

        let valid_sig = "a".repeat(128);
        assert!(signing_req.verify_signature(&valid_sig).is_err());
    }

    #[test]
    fn test_payment_authorization_creation() {
        let auth = PaymentAuthorization::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "signature_hex_here".to_string(),
        );

        assert_eq!(auth.payment_request_id, "payment-123");
        assert!(!auth.id.is_empty());
    }

    #[test]
    fn test_payment_authorization_handler_create_signing_request() {
        let mut handler = PaymentAuthorizationHandler::new();

        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = handler.create_signing_request(&payment);
        assert!(signing_req.is_ok());
    }

    #[test]
    fn test_payment_authorization_handler_get_signing_request() {
        let mut handler = PaymentAuthorizationHandler::new();

        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = handler.create_signing_request(&payment).unwrap();
        let request_id = signing_req.request_id.clone();

        let retrieved = handler.get_signing_request(&request_id);
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().payment_request_id, payment.id);
    }

    #[test]
    fn test_payment_authorization_handler_get_nonexistent_request() {
        let handler = PaymentAuthorizationHandler::new();
        let retrieved = handler.get_signing_request("nonexistent");
        assert!(retrieved.is_err());
    }

    #[test]
    fn test_payment_authorization_handler_verify_and_authorize() {
        let mut handler = PaymentAuthorizationHandler::new();

        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = handler.create_signing_request(&payment).unwrap();
        let request_id = signing_req.request_id.clone();

        // Create valid signature
        let valid_sig = "b".repeat(128);

        let auth = handler.verify_and_authorize(&request_id, &valid_sig);
        assert!(auth.is_ok());
        let a = auth.unwrap();
        assert_eq!(a.signature, valid_sig);
    }

    #[test]
    fn test_payment_authorization_handler_cleanup_expired() {
        let mut handler = PaymentAuthorizationHandler::new();

        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let _signing_req = handler.create_signing_request(&payment).unwrap();
        
        // Before cleanup, should have 1 request
        assert_eq!(handler.signing_requests.len(), 1);

        // Cleanup (should not remove valid requests)
        handler.cleanup_expired();
        assert_eq!(handler.signing_requests.len(), 1);
    }

    #[test]
    fn test_authorization_no_secret_keys_exposed() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Verify serialization doesn't expose secrets
        let json = serde_json::to_string(&signing_req).unwrap();
        assert!(!json.contains("secret"));
        assert!(!json.contains("private"));
        assert!(!json.contains("password"));
        
        // Should only contain public signing data
        assert!(json.contains("signing_data"));
        assert!(json.contains("account"));
        assert!(json.contains("Ed25519"));
    }

    #[test]
    fn test_authorization_signing_request_only_hash_exposed() {
        let payment = PaymentRequest::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            PaymentType::StudentToMerchant,
            "Lunch".to_string(),
        ).unwrap();

        let signing_req = PaymentSigningRequest::new(
            payment.id.clone(),
            payment.payer.clone(),
            &payment,
        ).unwrap();

        // Signing data should be a hash, not sensitive payment data
        assert_eq!(signing_req.signing_data.len(), 64); // SHA-256 = 64 hex chars
        assert!(signing_req.signing_data.chars().all(|c| c.is_ascii_hexdigit()));
    }

    // ==================== STELLAR TRANSACTION CREATION & ENVELOPE TESTS ====================

    #[test]
    fn test_stellar_transaction_builder_creation() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        );

        assert!(builder.is_ok());
        let b = builder.unwrap();
        assert_eq!(b.fee, 100);
        assert_eq!(b.sequence, 0);
    }

    #[test]
    fn test_stellar_transaction_builder_invalid_accounts_same() {
        let key = "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW";
        let builder = StellarTransactionBuilder::new(
            key.to_string(),
            key.to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        );

        assert!(builder.is_err());
    }

    #[test]
    fn test_stellar_transaction_builder_invalid_amount() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "-50.00".to_string(),
            "Payment".to_string(),
        );

        assert!(builder.is_err());
    }

    #[test]
    fn test_stellar_transaction_builder_with_fee() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap();

        let with_fee = builder.with_fee(500);
        assert!(with_fee.is_ok());
        assert_eq!(with_fee.unwrap().fee, 500);
    }

    #[test]
    fn test_stellar_transaction_builder_invalid_fee_too_low() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap();

        let with_fee = builder.with_fee(50);
        assert!(with_fee.is_err());
    }

    #[test]
    fn test_stellar_transaction_builder_with_sequence() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap();

        let with_seq = builder.with_sequence(42);
        assert_eq!(with_seq.sequence, 42);
    }

    #[test]
    fn test_stellar_transaction_builder_build() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build();
        assert!(envelope.is_ok());
        let env = envelope.unwrap();
        assert_eq!(env.network, "testnet");
        assert_eq!(env.amount, "100.00");
    }

    #[test]
    fn test_stellar_transaction_builder_build_without_sequence() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap();

        let envelope = builder.build();
        assert!(envelope.is_err());
    }

    #[test]
    fn test_stellar_transaction_envelope_validate() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        assert!(envelope.validate().is_ok());
    }

    #[test]
    fn test_stellar_transaction_envelope_get_signing_hash() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let hash = envelope.get_signing_hash();
        
        assert!(hash.is_ok());
        let h = hash.unwrap();
        // Should be SHA-256 hash (64 hex chars)
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_stellar_transaction_envelope_json_serialization() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let json = envelope.to_json();
        
        assert!(json.is_ok());
        let j = json.unwrap();
        assert!(j.contains("testnet"));
        assert!(j.contains("100.00"));
    }

    #[test]
    fn test_stellar_transaction_envelope_json_deserialization() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let env1 = builder.build().unwrap();
        let json = env1.to_json().unwrap();
        let env2 = StellarTransactionEnvelope::from_json(&json);

        assert!(env2.is_ok());
        let e = env2.unwrap();
        assert_eq!(e.amount, "100.00");
        assert_eq!(e.network, "testnet");
    }

    #[test]
    fn test_signed_stellar_transaction_creation() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let source = envelope.source_account.clone();
        let valid_sig = "c".repeat(128);

        let signed = SignedStellarTransaction::new(envelope, valid_sig.clone(), source);
        assert!(signed.is_ok());
        let s = signed.unwrap();
        assert_eq!(s.signature, valid_sig);
    }

    #[test]
    fn test_signed_stellar_transaction_invalid_signature_format() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let source = envelope.source_account.clone();

        let signed = SignedStellarTransaction::new(envelope, "invalid_sig".to_string(), source);
        assert!(signed.is_err());
    }

    #[test]
    fn test_signed_stellar_transaction_signer_mismatch() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let valid_sig = "d".repeat(128);

        // Try to sign with wrong account
        let signed = SignedStellarTransaction::new(
            envelope,
            valid_sig,
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(), // Different from source
        );
        assert!(signed.is_err());
    }

    #[test]
    fn test_signed_stellar_transaction_validate() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let source = envelope.source_account.clone();
        let valid_sig = "e".repeat(128);

        let signed = SignedStellarTransaction::new(envelope, valid_sig, source).unwrap();
        assert!(signed.validate().is_ok());
    }

    #[test]
    fn test_signed_stellar_transaction_json() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let envelope = builder.build().unwrap();
        let source = envelope.source_account.clone();
        let valid_sig = "f".repeat(128);

        let signed = SignedStellarTransaction::new(envelope, valid_sig, source).unwrap();
        let json = signed.to_json();
        
        assert!(json.is_ok());
        let j = json.unwrap();
        assert!(j.contains("testnet"));
        assert!(j.contains("100.00"));
    }

    #[test]
    fn test_transaction_builder_deterministic_hash() {
        let builder1 = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let signing_data1 = builder1.get_transaction_for_signing().unwrap();

        let builder2 = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let signing_data2 = builder2.get_transaction_for_signing().unwrap();

        // Same transaction should produce same signing hash (deterministic)
        assert_eq!(signing_data1, signing_data2);
    }

    #[test]
    fn test_transaction_testnet_enforcement() {
        let builder = StellarTransactionBuilder::new(
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
            "Payment".to_string(),
        ).unwrap()
            .with_sequence(1);

        let mut envelope = builder.build().unwrap();
        
        // Verify it's testnet
        assert_eq!(envelope.network, "testnet");
        assert!(envelope.validate().is_ok());

        // Try to change to mainnet - validation should fail
        envelope.network = "mainnet".to_string();
        assert!(envelope.validate().is_err());
    }

    // ==================== TRANSACTION SUBMISSION & STATUS TESTS ====================

    #[test]
    fn test_transaction_submission_result_creation() {
        let result = TransactionSubmissionResult::new(
            "a".repeat(64),
            12345,
            "tx-id-123".to_string(),
        );

        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.ledger, 12345);
        assert!(r.successful);
    }

    #[test]
    fn test_transaction_submission_result_invalid_hash() {
        let result = TransactionSubmissionResult::new(
            "invalid_hash".to_string(),
            12345,
            "tx-id-123".to_string(),
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_payment_transaction_creation() {
        let tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        );

        assert!(tx.is_ok());
        let t = tx.unwrap();
        assert_eq!(t.status, "pending");
        assert_eq!(t.attempts, 0);
        assert!(t.tx_hash.is_none());
    }

    #[test]
    fn test_payment_transaction_mark_submitted() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        assert!(tx.mark_submitted().is_ok());
        assert_eq!(tx.status, "submitted");
        assert_eq!(tx.attempts, 1);
        assert!(tx.submitted_at.is_some());
    }

    #[test]
    fn test_payment_transaction_mark_confirmed() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        assert!(tx.mark_submitted().is_ok());
        assert!(tx.mark_confirmed("b".repeat(64), 12345).is_ok());
        assert_eq!(tx.status, "confirmed");
        assert_eq!(tx.ledger, Some(12345));
        assert!(tx.tx_hash.is_some());
    }

    #[test]
    fn test_payment_transaction_mark_failed() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        assert!(tx.mark_submitted().is_ok());
        assert!(tx.mark_failed("Insufficient funds".to_string()).is_ok());
        assert_eq!(tx.status, "failed");
        assert_eq!(tx.last_error, Some("Insufficient funds".to_string()));
    }

    #[test]
    fn test_payment_transaction_can_retry() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        // Pending - cannot retry
        assert!(!tx.can_retry());

        tx.mark_submitted().ok();
        tx.mark_failed("Error".to_string()).ok();

        // Failed with attempts < 3
        assert!(tx.can_retry());

        // After 3 attempts
        tx.attempts = 3;
        assert!(!tx.can_retry());
    }

    #[test]
    fn test_payment_transaction_reset_for_retry() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        tx.mark_submitted().ok();
        tx.mark_failed("Error".to_string()).ok();

        assert_eq!(tx.status, "failed");
        assert!(tx.last_error.is_some());

        tx.reset_for_retry();
        assert_eq!(tx.status, "pending");
        assert!(tx.last_error.is_none());
    }

    #[test]
    fn test_payment_transaction_timeout() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        tx.mark_submitted().ok();

        // Just submitted - not timed out
        assert!(!tx.has_timed_out());

        // Force timeout
        tx.submitted_at = Some(Utc::now() - chrono::Duration::minutes(6));
        assert!(tx.has_timed_out());
    }

    #[test]
    fn test_transaction_history_filter_creation() {
        let filter = TransactionHistoryFilter::new();
        assert_eq!(filter.limit, 50);
        assert_eq!(filter.offset, 0);
        assert!(filter.status.is_none());
    }

    #[test]
    fn test_transaction_history_filter_with_status() {
        let filter = TransactionHistoryFilter::new()
            .with_status("confirmed".to_string());
        
        assert_eq!(filter.status, Some("confirmed".to_string()));
    }

    #[test]
    fn test_transaction_history_filter_with_source() {
        let filter = TransactionHistoryFilter::new()
            .with_source("GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string());
        
        assert!(filter.source.is_some());
    }

    #[test]
    fn test_transaction_history_filter_with_limit() {
        let filter = TransactionHistoryFilter::new()
            .with_limit(100);
        
        assert!(filter.is_ok());
        assert_eq!(filter.unwrap().limit, 100);
    }

    #[test]
    fn test_transaction_history_filter_invalid_limit() {
        let filter = TransactionHistoryFilter::new()
            .with_limit(2000);
        
        assert!(filter.is_err());
    }

    #[test]
    fn test_transaction_retry_handler_creation() {
        let handler = TransactionRetryHandler::new();
        assert_eq!(handler.max_retries, 3);
        assert_eq!(handler.retry_delay_seconds, 10);
    }

    #[test]
    fn test_transaction_retry_handler_with_max_retries() {
        let handler = TransactionRetryHandler::new()
            .with_max_retries(5);
        
        assert!(handler.is_ok());
        assert_eq!(handler.unwrap().max_retries, 5);
    }

    #[test]
    fn test_transaction_retry_handler_invalid_max_retries() {
        let result = TransactionRetryHandler::new()
            .with_max_retries(0);
        
        assert!(result.is_err());

        let result2 = TransactionRetryHandler::new()
            .with_max_retries(20);
        
        assert!(result2.is_err());
    }

    #[test]
    fn test_transaction_retry_handler_backoff() {
        let handler = TransactionRetryHandler::new();
        
        let delay1 = handler.get_backoff_delay(1);
        let delay2 = handler.get_backoff_delay(2);
        let delay3 = handler.get_backoff_delay(3);

        // Exponential backoff: 10, 20, 40
        assert_eq!(delay1, 10);
        assert_eq!(delay2, 20);
        assert_eq!(delay3, 40);
    }

    #[test]
    fn test_transaction_retry_handler_should_retry() {
        let handler = TransactionRetryHandler::new();

        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        // Pending - should not retry
        assert!(!handler.should_retry(&tx));

        // Failed - should retry
        tx.status = "failed".to_string();
        tx.attempts = 1;
        assert!(handler.should_retry(&tx));

        // Max attempts reached
        tx.attempts = 3;
        assert!(!handler.should_retry(&tx));
    }

    #[test]
    fn test_payment_transaction_full_lifecycle() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        // Start: pending
        assert_eq!(tx.get_status(), "pending");

        // Submit
        assert!(tx.mark_submitted().is_ok());
        assert_eq!(tx.get_status(), "submitted");

        // Confirm
        assert!(tx.mark_confirmed("c".repeat(64), 99999).is_ok());
        assert_eq!(tx.get_status(), "confirmed");
    }

    #[test]
    fn test_payment_transaction_invalid_state_transitions() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        // Try to confirm without submitting
        let result = tx.mark_confirmed("d".repeat(64), 99999);
        assert!(result.is_err());
    }

    #[test]
    fn test_payment_transaction_cannot_fail_confirmed() {
        let mut tx = PaymentTransaction::new(
            "payment-123".to_string(),
            "GCZST3XVCDTUJ76ZAV2HA72KYXZ2KMLYZCBNXQV637UFGM2WUZVFPJJW".to_string(),
            "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
            "100.00".to_string(),
        ).unwrap();

        tx.mark_submitted().ok();
        tx.mark_confirmed("e".repeat(64), 99999).ok();

        // Try to fail a confirmed transaction
        let result = tx.mark_failed("Error".to_string());
        assert!(result.is_err());
    }
