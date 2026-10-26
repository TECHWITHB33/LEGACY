-- Invoice Escrow Schema for Soroban Smart Contracts
-- Handles tokenized invoice lifecycle and investor escrow contracts on Stellar
-- All records immutable after creation: state changes recorded as new history entries

-- Invoices table: core invoice records with state lifecycle
CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    borrower_id TEXT NOT NULL,
    amount TEXT NOT NULL,
    state TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (borrower_id) REFERENCES users(id)
);

CREATE INDEX idx_invoices_borrower_id ON invoices(borrower_id);
CREATE INDEX idx_invoices_state ON invoices(state);
CREATE INDEX idx_invoices_created_at ON invoices(created_at);

-- Escrow Contracts: investor funds held for tokenized invoices
CREATE TABLE IF NOT EXISTS escrow_contracts (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL UNIQUE,
    investor_account TEXT NOT NULL,
    borrower_account TEXT NOT NULL,
    amount TEXT NOT NULL,
    settlement_expiry DATETIME NOT NULL,
    soroban_contract_id TEXT,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id)
);

CREATE INDEX idx_escrow_contracts_invoice_id ON escrow_contracts(invoice_id);
CREATE INDEX idx_escrow_contracts_investor_account ON escrow_contracts(investor_account);
CREATE INDEX idx_escrow_contracts_settlement_expiry ON escrow_contracts(settlement_expiry);

-- Escrow State History: immutable audit trail of all state transitions
CREATE TABLE IF NOT EXISTS escrow_state_history (
    id TEXT PRIMARY KEY,
    escrow_id TEXT NOT NULL,
    previous_state TEXT NOT NULL,
    new_state TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    details TEXT NOT NULL,
    FOREIGN KEY (escrow_id) REFERENCES escrow_contracts(id)
);

CREATE INDEX idx_escrow_state_history_escrow_id ON escrow_state_history(escrow_id);
CREATE INDEX idx_escrow_state_history_timestamp ON escrow_state_history(timestamp);

-- Settlement Fees: track all fees associated with invoice settlement
CREATE TABLE IF NOT EXISTS settlement_fees (
    id TEXT PRIMARY KEY,
    invoice_id TEXT NOT NULL,
    fee_type TEXT NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL,
    recorded_at DATETIME NOT NULL,
    FOREIGN KEY (invoice_id) REFERENCES invoices(id)
);

CREATE INDEX idx_settlement_fees_invoice_id ON settlement_fees(invoice_id);
CREATE INDEX idx_settlement_fees_fee_type ON settlement_fees(fee_type);
CREATE INDEX idx_settlement_fees_recorded_at ON settlement_fees(recorded_at);

-- Constraints and Rules
-- 1. Immutability: never UPDATE invoice/escrow records after creation
-- 2. State changes: INSERT new history entries, never modify contract amounts or dates
-- 3. settlement_expiry must be after created_at
-- 4. escrow_contracts.amount matches invoices.amount for the linked invoice
-- 5. fee_type must be one of: 'platform', 'network', 'currency_conversion'
