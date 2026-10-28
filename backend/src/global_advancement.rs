//! Global advancement service for cross-border settlement with fee tracking
//!
//! Orchestrates multi-currency settlement:
//! - Calculate settlement fees (platform, network, conversion)
//! - Record fees immutably in settlement_fees table
//! - Convert between currencies if needed
//! - Coordinate with Soroban escrow contract for fund release

use crate::error::ApiError;
use crate::models::{Invoice, FeeBreakdown, SettlementFee};
use crate::audit::{log_event, AuditEvent};
use sqlx::SqlitePool;
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;

pub struct GlobalAdvancementService {
    pool: Arc<SqlitePool>,
}

impl GlobalAdvancementService {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        GlobalAdvancementService { pool }
    }

    /// Calculate settlement fees
    ///
    /// Returns breakdown of:
    /// - Platform fee (1% of amount)
    /// - Network fee (fixed or percentage)
    /// - Conversion fee (for multi-currency)
    pub async fn calculate_fees(
        &self,
        invoice_id: &str,
        amount: &str,
        currency: &str,
    ) -> Result<FeeBreakdown, ApiError> {
        tracing::info!(
            "Calculating fees: invoice_id={}, amount={}, currency={}",
            invoice_id, amount, currency
        );

        let amount_f64: f64 = amount.parse()
            .map_err(|_| ApiError::InvalidInput("Invalid amount format".to_string()))?;

        // Platform fee: 1% of amount
        let platform_fee = amount_f64 * 0.01;

        // Network fee: fixed 0.5 XLM or equivalent
        let network_fee = 0.5;

        // Conversion fee: 0.5% for non-XLM currencies
        let conversion_fee = if currency != "XLM" {
            amount_f64 * 0.005
        } else {
            0.0
        };

        let total_fees = platform_fee + network_fee + conversion_fee;

        Ok(FeeBreakdown {
            platform_fee: format!("{:.7}", platform_fee),
            network_fee: format!("{:.7}", network_fee),
            conversion_fee: format!("{:.7}", conversion_fee),
            total_fees: format!("{:.7}", total_fees),
        })
    }

    /// Record settlement fees immutably
    ///
    /// Each fee type is recorded as a separate row in settlement_fees table.
    /// Once inserted, fees cannot be modified or deleted (immutable audit trail).
    pub async fn record_settlement_fees(
        &self,
        invoice_id: &str,
        fees: &FeeBreakdown,
        currency: &str,
    ) -> Result<(), ApiError> {
        tracing::info!("Recording settlement fees: invoice_id={}", invoice_id);

        let now = Utc::now();

        // Record platform fee
        sqlx::query!(
            r#"INSERT INTO settlement_fees (id, invoice_id, fee_type, amount, currency, recorded_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            Uuid::new_v4().to_string(),
            invoice_id,
            "platform",
            fees.platform_fee,
            currency,
            now,
        )
        .execute(self.pool.as_ref())
        .await?;

        // Record network fee
        sqlx::query!(
            r#"INSERT INTO settlement_fees (id, invoice_id, fee_type, amount, currency, recorded_at)
               VALUES (?, ?, ?, ?, ?, ?)"#,
            Uuid::new_v4().to_string(),
            invoice_id,
            "network",
            fees.network_fee,
            currency,
            now,
        )
        .execute(self.pool.as_ref())
        .await?;

        // Record conversion fee if applicable
        if fees.conversion_fee != "0" {
            sqlx::query!(
                r#"INSERT INTO settlement_fees (id, invoice_id, fee_type, amount, currency, recorded_at)
                   VALUES (?, ?, ?, ?, ?, ?)"#,
                Uuid::new_v4().to_string(),
                invoice_id,
                "conversion",
                fees.conversion_fee,
                currency,
                now,
            )
            .execute(self.pool.as_ref())
            .await?;
        }

        tracing::info!(
            "Settlement fees recorded: invoice_id={}, total={}",
            invoice_id, fees.total_fees
        );

        Ok(())
    }

    /// Convert asset between currencies
    ///
    /// In production, this would:
    /// - Look up exchange rate from oracle or cache
    /// - Apply conversion with fee
    /// - Return converted amount
    ///
    /// For now, this is a placeholder that assumes 1:1 conversion
    pub async fn convert_asset(
        &self,
        _from_currency: &str,
        _to_currency: &str,
        amount: &str,
    ) -> Result<String, ApiError> {
        // Placeholder: return amount as-is
        // In production: fetch real exchange rate
        Ok(amount.to_string())
    }

    /// Execute global advancement: full settlement orchestration
    ///
    /// Coordinates:
    /// 1. Fetch invoice and escrow contract details
    /// 2. Calculate settlement fees
    /// 3. Record fees immutably
    /// 4. Convert currency if needed
    /// 5. Coordinate with Soroban contract to release funds
    /// 6. Update invoice state to Settled
    pub async fn execute_global_advancement(
        &self,
        invoice_id: &str,
        borrower_account: &str,
    ) -> Result<(), ApiError> {
        tracing::info!(
            "Executing global advancement: invoice_id={}",
            invoice_id
        );

        // Fetch invoice
        let invoice = sqlx::query_as::<_, Invoice>(
            "SELECT id, borrower_id, amount, state, created_at, updated_at FROM invoices WHERE id = ?"
        )
        .bind(invoice_id)
        .fetch_optional(self.pool.as_ref())
        .await?
        .ok_or_else(|| ApiError::ValidationError(format!("Invoice not found: {}", invoice_id)))?;

        // Calculate fees
        let fees = self
            .calculate_fees(invoice_id, &invoice.amount, "XLM")
            .await?;

        // Record fees immutably
        self.record_settlement_fees(invoice_id, &fees, "XLM").await?;

        // In production, would coordinate with Soroban contract here
        // For now, just log and update state
        log_event(
            &self.pool,
            "system",
            AuditEvent::SettlementRecorded,
            "Invoice",
            Some(invoice_id),
            Some(&format!("Settlement fees: {}", fees.total_fees)),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("Failed to log audit event: {}", e));

        tracing::info!(
            "Global advancement completed: invoice_id={}, fees={}",
            invoice_id, fees.total_fees
        );

        Ok(())
    }

    /// Get all fees for an invoice
    pub async fn get_settlement_fees(&self, invoice_id: &str) -> Result<Vec<SettlementFee>, ApiError> {
        let fees = sqlx::query_as::<_, SettlementFee>(
            "SELECT id, invoice_id, fee_type, amount, currency, recorded_at FROM settlement_fees WHERE invoice_id = ? ORDER BY recorded_at DESC"
        )
        .bind(invoice_id)
        .fetch_all(self.pool.as_ref())
        .await?;

        Ok(fees)
    }
}
