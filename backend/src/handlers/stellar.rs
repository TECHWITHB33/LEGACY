// Stellar Payment REST Handlers
// IMPORTANT: Testnet only, no secret keys exposed

use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;
use crate::stellar_payment::*;

/// Create payment request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePaymentRequestRequest {
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub payment_type: String,
    pub memo: String,
    pub expiry_hours: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequestResponse {
    pub id: String,
    pub payer: String,
    pub payee: String,
    pub amount: String,
    pub payment_type: String,
    pub memo: String,
    pub status: String,
    pub expires_at: String,
    pub created_at: String,
}

impl From<PaymentRequest> for PaymentRequestResponse {
    fn from(pr: PaymentRequest) -> Self {
        PaymentRequestResponse {
            id: pr.id,
            payer: pr.payer,
            payee: pr.payee,
            amount: pr.amount,
            payment_type: pr.payment_type,
            memo: pr.memo,
            status: pr.status,
            expires_at: pr.expires_at.to_rfc3339(),
            created_at: pr.created_at.to_rfc3339(),
        }
    }
}

/// Create a new payment request
pub async fn create_payment_request(
    req: web::Json<CreatePaymentRequestRequest>,
) -> Result<HttpResponse, ApiError> {
    // Parse payment type
    let ptype = match req.payment_type.as_str() {
        "student_to_merchant" => PaymentType::StudentToMerchant,
        "merchant_to_student" => PaymentType::MerchantToStudent,
        "student_to_student" => PaymentType::StudentToStudent,
        _ => return Err(ApiError::ValidationError(
            "Invalid payment type".to_string(),
        )),
    };

    // Create payment request with optional custom expiry
    let payment = if let Some(hours) = req.expiry_hours {
        PaymentRequest::with_expiration(
            req.payer.clone(),
            req.payee.clone(),
            req.amount.clone(),
            ptype,
            req.memo.clone(),
            hours,
        )?
    } else {
        PaymentRequest::new(
            req.payer.clone(),
            req.payee.clone(),
            req.amount.clone(),
            ptype,
            req.memo.clone(),
        )?
    };

    let response: PaymentRequestResponse = payment.into();
    Ok(HttpResponse::Created().json(response))
}

/// Get payment request by ID
pub async fn get_payment_request(
    request_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // In production, fetch from database
    // For now, return placeholder
    Ok(HttpResponse::NotFound().finish())
}

/// Generate QR code for payment
#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateQrRequest {
    pub payment_request_id: String,
    pub payee: String,
    pub amount: String,
    pub memo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateQrResponse {
    pub request_id: String,
    pub qr_uri: String,
    pub payment_url: String,
    pub network: String,
}

pub async fn generate_payment_qr(
    req: web::Json<GenerateQrRequest>,
) -> Result<HttpResponse, ApiError> {
    // Create QR data
    let qr_data = PaymentQrData::new(
        req.payment_request_id.clone(),
        req.payee.clone(),
        req.amount.clone(),
        req.memo.clone(),
    )?;

    // Generate Stellar URI
    let qr_uri = qr_data.get_stellar_uri();
    let payment_url = qr_data.get_payment_url("https://api.example.com");

    let response = GenerateQrResponse {
        request_id: qr_data.request_id,
        qr_uri,
        payment_url,
        network: qr_data.network,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Create signing request for payment authorization
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSigningRequestRequest {
    pub payment_request_id: String,
    pub account: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SigningRequestResponse {
    pub request_id: String,
    pub payment_request_id: String,
    pub account: String,
    pub signing_data: String,
    pub algorithm: String,
    pub expires_at: String,
}

pub async fn create_signing_request(
    req: web::Json<CreateSigningRequestRequest>,
) -> Result<HttpResponse, ApiError> {
    // Create dummy payment for signing request
    let payment = PaymentRequest::new(
        req.account.clone(),
        "GBBD47UZQ53HALXJEMZUKKMYRJA".to_string(),
        "100.00".to_string(),
        PaymentType::StudentToMerchant,
        "Test".to_string(),
    )?;

    let signing_req = PaymentSigningRequest::new(
        payment.id.clone(),
        req.account.clone(),
        &payment,
    )?;

    let response = SigningRequestResponse {
        request_id: signing_req.request_id,
        payment_request_id: signing_req.payment_request_id,
        account: signing_req.account,
        signing_data: signing_req.signing_data,
        algorithm: signing_req.algorithm,
        expires_at: signing_req.expires_at.to_rfc3339(),
    };

    Ok(HttpResponse::Created().json(response))
}

/// Submit payment signature for authorization
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitSignatureRequest {
    pub signing_request_id: String,
    pub signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentAuthorizationResponse {
    pub id: String,
    pub payment_request_id: String,
    pub authorized_by: String,
    pub authorized_at: String,
}

pub async fn submit_payment_signature(
    req: web::Json<SubmitSignatureRequest>,
) -> Result<HttpResponse, ApiError> {
    // In production, verify signature and create authorization
    // For now, return placeholder
    Err(ApiError::ValidationError(
        "Not implemented in this phase".to_string(),
    ))
}

/// Get transaction status
pub async fn get_transaction_status(
    tx_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    // In production, fetch from database
    Ok(HttpResponse::NotFound().finish())
}

/// Get transaction history for account
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionHistoryQuery {
    pub account: String,
    pub status: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub async fn get_transaction_history(
    query: web::Query<TransactionHistoryQuery>,
) -> Result<HttpResponse, ApiError> {
    // In production, fetch from database with filters
    Ok(HttpResponse::Ok().json(vec![]))
}

/// Confirm payment (webhook from Stellar or polling)
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmPaymentRequest {
    pub transaction_id: String,
    pub tx_hash: String,
    pub ledger: u32,
}

pub async fn confirm_payment(
    req: web::Json<ConfirmPaymentRequest>,
) -> Result<HttpResponse, ApiError> {
    // In production, update transaction status in database
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "transaction_id": req.transaction_id
    })))
}

/// Health check for payment service
pub async fn health_check() -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "service": "stellar_payment",
        "network": "testnet"
    })))
}

// Register handlers with Actix
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1/stellar")
            .route("/health", web::get().to(health_check))
            .route("/payment-request", web::post().to(create_payment_request))
            .route("/payment-request/{id}", web::get().to(get_payment_request))
            .route("/qr-code", web::post().to(generate_payment_qr))
            .route("/signing-request", web::post().to(create_signing_request))
            .route("/submit-signature", web::post().to(submit_payment_signature))
            .route("/transaction/{id}", web::get().to(get_transaction_status))
            .route("/transaction-history", web::get().to(get_transaction_history))
            .route("/confirm-payment", web::post().to(confirm_payment))
    );
}
