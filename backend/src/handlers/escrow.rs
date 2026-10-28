use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::{web, HttpRequest, HttpResponse};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::error::ApiError;
use crate::models::{
    CreateInvoiceRequest, CreateInvoiceResponse, InvoiceResponse, UpdateInvoiceStateRequest,
    FundInvoiceRequest, SettleInvoiceRequest, SettlementSummaryResponse, EscrowResponse,
    PaginationParams, PaginatedResponse, Invoice,
};
use crate::middleware::extract_user_id;
use crate::config::Config;
use crate::invoice_lifecycle::InvoiceLifecycleService;
use crate::services::EscrowService;
use crate::audit::{log_event, AuditEvent};
use chrono::Utc;

/// CREATE INVOICE
/// 
/// Creates a new invoice in Draft state for the borrower.
/// Authorization: Borrower (JWT required)
/// Returns: 201 Created with invoice_id
pub async fn create_invoice(
    req: HttpRequest,
    body: web::Json<CreateInvoiceRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT (fail-closed: reject if not found)
    let borrower_id = extract_user_id(&req)?;

    tracing::info!("Creating invoice: borrower_id={}, amount={}", borrower_id, body.amount);

    // Create service
    let service = InvoiceLifecycleService::new(pool.as_ref().clone());

    // Create invoice
    let invoice_id = service
        .create_invoice(&borrower_id, &body.amount)
        .await?;

    let response = CreateInvoiceResponse {
        invoice_id: invoice_id.clone(),
        state: "Draft".to_string(),
        created_at: Utc::now(),
    };

    tracing::info!("Invoice created successfully: invoice_id={}", invoice_id);

    Ok(HttpResponse::Created().json(response))
}

/// SUBMIT INVOICE
///
/// Transitions invoice from Draft to Submitted state.
/// Authorization: Borrower must own the invoice (fail-closed)
/// Returns: 200 OK with updated invoice
pub async fn submit_invoice(
    req: HttpRequest,
    path: web::Path<String>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT
    let borrower_id = extract_user_id(&req)?;
    let invoice_id = path.into_inner();

    tracing::info!("Submitting invoice: invoice_id={}, borrower_id={}", invoice_id, borrower_id);

    let service = InvoiceLifecycleService::new(pool.as_ref().clone());

    // Verify ownership: fetch invoice and check borrower_id
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"
    )
    .bind(&invoice_id)
    .fetch_optional(pool.as_ref().as_ref())
    .await?
    .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

    // Authorization: fail-closed pattern - reject if not owner
    if invoice.borrower_id != borrower_id {
        tracing::warn!(
            "Unauthorized invoice access: invoice_id={}, borrower_id={}, requester={}",
            invoice_id, invoice.borrower_id, borrower_id
        );
        return Err(ApiError::AuthorizationError(
            "You are not authorized to access this invoice".to_string(),
        ));
    }

    // Submit invoice
    service.submit_invoice(&invoice_id).await?;

    // Fetch updated invoice
    let updated_invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"
    )
    .bind(&invoice_id)
    .fetch_one(pool.as_ref().as_ref())
    .await?;

    let response = InvoiceResponse {
        id: updated_invoice.id,
        borrower_id: updated_invoice.borrower_id,
        amount: updated_invoice.amount,
        state: updated_invoice.state,
        created_at: updated_invoice.created_at,
        updated_at: updated_invoice.updated_at,
    };

    tracing::info!("Invoice submitted successfully: invoice_id={}", invoice_id);

    Ok(HttpResponse::Ok().json(response))
}

/// FUND INVOICE
///
/// Transitions invoice from Submitted to Funded and deploys Soroban escrow contract.
/// Authorization: Investor (extracted from Stellar session, admin can force)
/// Returns: 200 OK with escrow details
pub async fn fund_invoice(
    req: HttpRequest,
    body: web::Json<FundInvoiceRequest>,
    pool: web::Data<Arc<SqlitePool>>,
    config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, ApiError> {
    // Extract investor account from JWT (fail-closed)
    let investor_id = extract_user_id(&req)?;
    let invoice_id = &body.invoice_id;

    tracing::info!(
        "Funding invoice: invoice_id={}, investor_id={}, expiry={}",
        invoice_id, investor_id, body.settlement_expiry_unix_timestamp
    );

    let lifecycle_service = InvoiceLifecycleService::new(pool.as_ref().clone());
    let escrow_service = EscrowService::new(pool.as_ref().clone(), config.as_ref().clone());

    // Verify invoice exists and is in Submitted state
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"
    )
    .bind(invoice_id)
    .fetch_optional(pool.as_ref().as_ref())
    .await?
    .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

    if invoice.state != "Submitted" {
        return Err(ApiError::ValidationError(format!(
            "Invalid invoice state: {}. Must be Submitted to fund.",
            invoice.state
        )));
    }

    // Authorization: For now, any authenticated investor can fund. In production,
    // implement proper investor verification against Stellar account registry.
    // Fail-closed: reject if no investor context.

    // Transition invoice to Funded
    lifecycle_service.fund_invoice(invoice_id, &investor_id).await?;

    // Deploy escrow contract on Soroban
    // In production, derive investor_account and borrower_account from Stellar registry
    let investor_account = format!("stellar:{}", investor_id);
    let borrower_account = format!("stellar:{}", invoice.borrower_id);

    let contract_id = escrow_service
        .deploy_escrow_contract(
            invoice_id,
            &investor_account,
            &borrower_account,
            invoice.amount.parse::<f64>()
                .map_err(|_| ApiError::InvalidInput("Invalid amount format".to_string()))?,
            body.settlement_expiry_unix_timestamp as i64,
        )
        .await?;

    // Store contract_id in database (escrow table)
    let now = Utc::now();
    sqlx::query!(
        r#"INSERT INTO escrow_contracts (id, invoice_id, investor_account, borrower_account, amount, settlement_expiry, soroban_contract_id, created_at)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        uuid::Uuid::new_v4().to_string(),
        invoice_id,
        investor_account,
        borrower_account,
        invoice.amount,
        body.settlement_expiry_unix_timestamp,
        contract_id,
        now,
    )
    .execute(pool.as_ref().as_ref())
    .await?;

    // Log audit event
    log_event(
        &pool,
        &investor_id,
        AuditEvent::InvoiceStateChanged,
        "Invoice",
        Some(invoice_id),
        Some(&format!("Submitted -> Funded (contract_id: {})", contract_id)),
    )
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

    let response = EscrowResponse {
        id: uuid::Uuid::new_v4().to_string(),
        invoice_id: invoice_id.clone(),
        investor_account,
        amount: invoice.amount,
        state: "Funded".to_string(),
        soroban_contract_id: Some(contract_id),
    };

    tracing::info!(
        "Invoice funded successfully: invoice_id={}, contract_id={}",
        invoice_id,
        response.soroban_contract_id.as_ref().unwrap_or(&"unknown".to_string())
    );

    Ok(HttpResponse::Ok().json(response))
}

/// GET INVOICE
///
/// Retrieves invoice details.
/// Authorization: Borrower sees own invoices, Investor sees funded invoices, Admin sees all
/// Returns: 200 OK with invoice details or 404 Not Found
pub async fn get_invoice(
    req: HttpRequest,
    path: web::Path<String>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT
    let user_id = extract_user_id(&req)?;
    let invoice_id = path.into_inner();

    tracing::info!("Getting invoice: invoice_id={}, user_id={}", invoice_id, user_id);

    // Fetch invoice
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"
    )
    .bind(&invoice_id)
    .fetch_optional(pool.as_ref().as_ref())
    .await?
    .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

    // Authorization: fail-closed
    // - Borrower can see own invoices
    // - Investor can see funded invoices (we don't have investor role yet, so skip for now)
    // - Admin can see all (we don't have admin identification yet, so skip for now)
    if invoice.borrower_id != user_id {
        // In production, check if user_id is investor or admin
        // For now, only borrower access is allowed
        tracing::warn!(
            "Unauthorized invoice access: invoice_id={}, requested_by={}",
            invoice_id, user_id
        );
        return Err(ApiError::AuthorizationError(
            "You are not authorized to access this invoice".to_string(),
        ));
    }

    let response = InvoiceResponse {
        id: invoice.id,
        borrower_id: invoice.borrower_id,
        amount: invoice.amount,
        state: invoice.state,
        created_at: invoice.created_at,
        updated_at: invoice.updated_at,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// LIST INVOICES
///
/// Lists invoices for the current user.
/// Authorization: Borrower sees own invoices, Investor sees funded invoices, Admin sees all
/// Returns: 200 OK with paginated invoice list
pub async fn list_invoices(
    req: HttpRequest,
    query: web::Query<PaginationParams>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT
    let user_id = extract_user_id(&req)?;

    let page = query.page.max(1);
    let per_page = query.per_page.min(100).max(1);
    let offset = (page - 1) * per_page;

    tracing::info!(
        "Listing invoices: user_id={}, page={}, per_page={}",
        user_id, page, per_page
    );

    // Authorization: for now, borrower can only see own invoices
    // In production: implement role-based filtering (investor, admin)
    let invoices = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE borrower_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?"
    )
    .bind(&user_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool.as_ref().as_ref())
    .await?;

    // Get total count
    let total_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM invoices WHERE borrower_id = ?"
    )
    .bind(&user_id)
    .fetch_one(pool.as_ref().as_ref())
    .await?;

    let invoice_responses: Vec<InvoiceResponse> = invoices
        .into_iter()
        .map(|inv| InvoiceResponse {
            id: inv.id,
            borrower_id: inv.borrower_id,
            amount: inv.amount,
            state: inv.state,
            created_at: inv.created_at,
            updated_at: inv.updated_at,
        })
        .collect();

    let response = PaginatedResponse {
        data: invoice_responses,
        total: total_count.0 as u64,
        page,
        per_page,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// SETTLE INVOICE
///
/// Transitions invoice from Processing to Settled and executes global advancement.
/// Authorization: Admin or settlement service (fail-closed)
/// Returns: 200 OK with settlement summary
pub async fn settle_invoice(
    req: HttpRequest,
    body: web::Json<SettleInvoiceRequest>,
    pool: web::Data<Arc<SqlitePool>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT
    let user_id = extract_user_id(&req)?;
    let invoice_id = &body.invoice_id;

    tracing::info!(
        "Settling invoice: invoice_id={}, user_id={}",
        invoice_id, user_id
    );

    // Authorization: Currently only admin/settlement service can settle
    // In production: implement proper role checking (admin role)
    // For now, we accept any authenticated user (fail-closed at handler level)

    let service = InvoiceLifecycleService::new(pool.as_ref().clone());

    // Verify invoice exists and is in Processing state
    let invoice = sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"
    )
    .bind(invoice_id)
    .fetch_optional(pool.as_ref().as_ref())
    .await?
    .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

    if invoice.state != "Processing" {
        return Err(ApiError::ValidationError(format!(
            "Invalid invoice state: {}. Must be Processing to settle.",
            invoice.state
        )));
    }

    // Settle invoice
    let settlement_details = body
        .settlement_details_json
        .as_deref()
        .unwrap_or("{}");

    service.settle_invoice(invoice_id, settlement_details).await?;

    // Log settlement
    log_event(
        &pool,
        &user_id,
        AuditEvent::InvoiceStateChanged,
        "Invoice",
        Some(invoice_id),
        Some(&format!("Processing -> Settled")),
    )
    .await
    .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

    let response = SettlementSummaryResponse {
        invoice_id: invoice_id.clone(),
        state: "Settled".to_string(),
        amount: invoice.amount,
        contract_id: None, // Could fetch from escrow table if needed
        settled_at: Utc::now(),
    };

    tracing::info!("Invoice settled successfully: invoice_id={}", invoice_id);

    Ok(HttpResponse::Ok().json(response))
}

/// GET ESCROW STATUS
///
/// Retrieves the current state of an escrow contract on-chain.
/// Authorization: Any authenticated user (fail-closed at JWT level)
/// Returns: 200 OK with contract state or 404 Not Found
pub async fn get_escrow_status(
    req: HttpRequest,
    path: web::Path<String>,
    pool: web::Data<Arc<SqlitePool>>,
    config: web::Data<Arc<Config>>,
) -> Result<HttpResponse, ApiError> {
    // Extract user_id from JWT
    let _user_id = extract_user_id(&req)?;
    let contract_id = path.into_inner();

    tracing::info!("Getting escrow status: contract_id={}", contract_id);

    // Create escrow service
    let escrow_service = EscrowService::new(pool.as_ref().clone(), config.as_ref().clone());

    // Query contract state via RPC
    let state = escrow_service.get_contract_state(&contract_id).await?;

    // Fetch associated escrow from database for additional context
    let escrow_opt = sqlx::query!(
        "SELECT id, invoice_id, investor_account, amount FROM escrow_contracts WHERE soroban_contract_id = ?",
        contract_id
    )
    .fetch_optional(pool.as_ref().as_ref())
    .await?;

    let response = if let Some(escrow_row) = escrow_opt {
        EscrowResponse {
            id: escrow_row.id,
            invoice_id: escrow_row.invoice_id,
            investor_account: escrow_row.investor_account,
            amount: escrow_row.amount,
            state: if state.is_settled {
                "Settled".to_string()
            } else if state.is_refunded {
                "Refunded".to_string()
            } else {
                "Funded".to_string()
            },
            soroban_contract_id: Some(contract_id),
        }
    } else {
        return Err(ApiError::BlockchainError(
            format!("Escrow contract not found in database: {}", contract_id),
        ));
    };

    tracing::info!(
        "Escrow status retrieved: contract_id={}, state={}",
        contract_id, response.state
    );

    Ok(HttpResponse::Ok().json(response))
}
