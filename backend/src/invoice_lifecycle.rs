//! Invoice lifecycle state machine
//!
//! Manages the complete lifecycle of invoices:
//! Draft → Submitted → Funded → Processing → Settled → Closed

use crate::error::ApiError;
use crate::models::{Invoice, InvoiceState};
use crate::audit::{log_event, AuditEvent};
use sqlx::SqlitePool;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

pub struct InvoiceLifecycleService {
    pool: Arc<SqlitePool>,
}

impl InvoiceLifecycleService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        InvoiceLifecycleService { pool }
    }

    /// Create a new invoice in Draft state
    pub async fn create_invoice(
        &self,
        borrower_id: &str,
        amount: &str,
    ) -> Result<String, ApiError> {
        let invoice_id = format!("INV-{}", Uuid::new_v4());
        let now = Utc::now();

        tracing::info!(
            "Creating invoice: invoice_id={}, borrower_id={}, amount={}",
            invoice_id, borrower_id, amount
        );

        sqlx::query!(
            r#"INSERT INTO invoices (id, borrower_id, amount, state, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            invoice_id,
            borrower_id,
            amount,
            "Draft",
            now,
            now,
        )
        .execute(self.pool.as_ref())
        .await?;

        log_event(
            &self.pool,
            borrower_id,
            AuditEvent::InvoiceCreated,
            "Invoice",
            Some(&invoice_id),
            Some(&format!("Created in Draft state, amount: {}", amount)),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        Ok(invoice_id)
    }

    /// Transition invoice from Draft to Submitted
    pub async fn submit_invoice(&self, invoice_id: &str) -> Result<(), ApiError> {
        tracing::info!("Submitting invoice: invoice_id={}", invoice_id);

        // Fetch current invoice
        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        // Validate state transition
        let current_state = InvoiceState::from_str(&invoice.state)
            .ok_or_else(|| ApiError::ValidationError(format!("Invalid invoice state: {}", invoice.state)))?;
        
        let next_state = InvoiceState::Submitted;
        if !current_state.can_transition_to(&next_state) {
            return Err(ApiError::ValidationError(
                format!("Cannot transition from {} to {}", current_state.as_str(), next_state.as_str())
            ));
        }

        // Update state
        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Submitted",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        log_event(
            &self.pool,
            &invoice.borrower_id,
            AuditEvent::InvoiceStateChanged,
            "Invoice",
            Some(invoice_id),
            Some("Draft -> Submitted"),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        Ok(())
    }

    /// Transition invoice from Submitted to Funded
    pub async fn fund_invoice(&self, invoice_id: &str, investor_id: &str) -> Result<(), ApiError> {
        tracing::info!(
            "Funding invoice: invoice_id={}, investor_id={}",
            invoice_id, investor_id
        );

        // Fetch current invoice
        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        // Validate state transition
        let current_state = InvoiceState::from_str(&invoice.state)
            .ok_or_else(|| ApiError::ValidationError(format!("Invalid invoice state: {}", invoice.state)))?;
        
        let next_state = InvoiceState::Funded;
        if !current_state.can_transition_to(&next_state) {
            return Err(ApiError::ValidationError(
                format!("Cannot transition from {} to {}", current_state.as_str(), next_state.as_str())
            ));
        }

        // Update state
        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Funded",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        log_event(
            &self.pool,
            investor_id,
            AuditEvent::InvoiceStateChanged,
            "Invoice",
            Some(invoice_id),
            Some("Submitted -> Funded"),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        Ok(())
    }

    /// Transition invoice to Processing (internal state before settlement)
    pub async fn mark_processing(&self, invoice_id: &str) -> Result<(), ApiError> {
        tracing::info!("Marking invoice as processing: invoice_id={}", invoice_id);

        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Processing",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Transition invoice from Processing to Settled
    pub async fn settle_invoice(&self, invoice_id: &str, settlement_details: &str) -> Result<(), ApiError> {
        tracing::info!("Settling invoice: invoice_id={}", invoice_id);

        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        // Validate state transition
        let current_state = InvoiceState::from_str(&invoice.state)
            .ok_or_else(|| ApiError::ValidationError(format!("Invalid invoice state: {}", invoice.state)))?;
        
        let next_state = InvoiceState::Settled;
        if !current_state.can_transition_to(&next_state) {
            return Err(ApiError::ValidationError(
                format!("Cannot transition from {} to {}", current_state.as_str(), next_state.as_str())
            ));
        }

        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Settled",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        log_event(
            &self.pool,
            "system",
            AuditEvent::SettlementRecorded,
            "Invoice",
            Some(invoice_id),
            Some(&format!("Settlement: {}", settlement_details)),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        Ok(())
    }

    /// Transition invoice from Settled to Closed
    pub async fn close_invoice(&self, invoice_id: &str) -> Result<(), ApiError> {
        tracing::info!("Closing invoice: invoice_id={}", invoice_id);

        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Closed",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Refund invoice (back to Draft or initial state)
    pub async fn refund_invoice(&self, invoice_id: &str, reason: &str) -> Result<(), ApiError> {
        tracing::info!("Refunding invoice: invoice_id={}, reason={}", invoice_id, reason);

        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        let now = Utc::now();
        sqlx::query!(
            "UPDATE invoices SET state = ?, updated_at = ? WHERE id = ?",
            "Draft",
            now,
            invoice_id,
        )
        .execute(self.pool.as_ref())
        .await?;

        log_event(
            &self.pool,
            "system",
            AuditEvent::InvoiceStateChanged,
            "Invoice",
            Some(invoice_id),
            Some(&format!("Refunded: {}", reason)),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        Ok(())
    }
}
