-- Stellar Payment Tables
-- Phase 5: Payment functionality on Stellar testnet

-- Student and Merchant Stellar Accounts
CREATE TABLE IF NOT EXISTS stellar_accounts (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    public_key TEXT UNIQUE NOT NULL,
    account_type TEXT NOT NULL CHECK(account_type IN ('student', 'merchant')),
    balance TEXT NOT NULL DEFAULT '0.0',
    sequence TEXT NOT NULL DEFAULT '0',
    verified BOOLEAN NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_stellar_accounts_user_id ON stellar_accounts(user_id);
CREATE INDEX idx_stellar_accounts_public_key ON stellar_accounts(public_key);
CREATE INDEX idx_stellar_accounts_type ON stellar_accounts(account_type);
CREATE INDEX idx_stellar_accounts_verified ON stellar_accounts(verified);

-- Payment Requests (pending, authorized, completed, failed)
CREATE TABLE IF NOT EXISTS payment_requests (
    id TEXT PRIMARY KEY,
    payer TEXT NOT NULL,
    payee TEXT NOT NULL,
    amount TEXT NOT NULL,
    payment_type TEXT NOT NULL CHECK(payment_type IN ('student_to_merchant', 'merchant_to_student', 'student_to_student')),
    memo TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'authorized', 'completed', 'failed')),
    expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (payer) REFERENCES stellar_accounts(public_key),
    FOREIGN KEY (payee) REFERENCES stellar_accounts(public_key)
);

CREATE INDEX idx_payment_requests_status ON payment_requests(status);
CREATE INDEX idx_payment_requests_payer ON payment_requests(payer);
CREATE INDEX idx_payment_requests_payee ON payment_requests(payee);
CREATE INDEX idx_payment_requests_expires_at ON payment_requests(expires_at);

-- Payment Authorizations (signatures + authorization records)
CREATE TABLE IF NOT EXISTS payment_authorizations (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL UNIQUE,
    authorized_by TEXT NOT NULL,
    signature TEXT NOT NULL,
    authorized_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (authorized_by) REFERENCES stellar_accounts(public_key)
);

CREATE INDEX idx_payment_authorizations_request_id ON payment_authorizations(payment_request_id);
CREATE INDEX idx_payment_authorizations_authorized_by ON payment_authorizations(authorized_by);

-- Payment Transactions (submitted to Stellar network)
CREATE TABLE IF NOT EXISTS payment_transactions (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL,
    tx_hash TEXT UNIQUE,
    source TEXT NOT NULL,
    destination TEXT NOT NULL,
    amount TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('pending', 'submitted', 'confirmed', 'failed')),
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    ledger INTEGER,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    submitted_at DATETIME,
    confirmed_at DATETIME,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (source) REFERENCES stellar_accounts(public_key),
    FOREIGN KEY (destination) REFERENCES stellar_accounts(public_key)
);

CREATE INDEX idx_payment_transactions_status ON payment_transactions(status);
CREATE INDEX idx_payment_transactions_source ON payment_transactions(source);
CREATE INDEX idx_payment_transactions_destination ON payment_transactions(destination);
CREATE INDEX idx_payment_transactions_tx_hash ON payment_transactions(tx_hash);
CREATE INDEX idx_payment_transactions_created_at ON payment_transactions(created_at);
CREATE INDEX idx_payment_transactions_confirmed_at ON payment_transactions(confirmed_at);

-- Payment QR Codes (for merchant use, no sensitive data)
CREATE TABLE IF NOT EXISTS payment_qr_codes (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL,
    request_id TEXT NOT NULL,
    payee TEXT NOT NULL,
    amount TEXT NOT NULL,
    memo TEXT NOT NULL,
    network TEXT NOT NULL CHECK(network = 'testnet'),
    created_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (payee) REFERENCES stellar_accounts(public_key)
);

CREATE INDEX idx_payment_qr_codes_request_id ON payment_qr_codes(request_id);
CREATE INDEX idx_payment_qr_codes_payee ON payment_qr_codes(payee);
CREATE INDEX idx_payment_qr_codes_created_at ON payment_qr_codes(created_at);

-- Payment Signing Requests (for authorization flow)
CREATE TABLE IF NOT EXISTS payment_signing_requests (
    id TEXT PRIMARY KEY,
    request_id TEXT NOT NULL UNIQUE,
    payment_request_id TEXT NOT NULL,
    account TEXT NOT NULL,
    signing_data TEXT NOT NULL,
    algorithm TEXT NOT NULL DEFAULT 'Ed25519',
    expires_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (account) REFERENCES stellar_accounts(public_key)
);

CREATE INDEX idx_payment_signing_requests_request_id ON payment_signing_requests(request_id);
CREATE INDEX idx_payment_signing_requests_account ON payment_signing_requests(account);
CREATE INDEX idx_payment_signing_requests_expires_at ON payment_signing_requests(expires_at);

-- Payment Transaction History (audit trail)
CREATE TABLE IF NOT EXISTS payment_transaction_history (
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    status_from TEXT NOT NULL,
    status_to TEXT NOT NULL,
    reason TEXT,
    timestamp DATETIME NOT NULL,
    FOREIGN KEY (transaction_id) REFERENCES payment_transactions(id)
);

CREATE INDEX idx_payment_transaction_history_transaction_id ON payment_transaction_history(transaction_id);
CREATE INDEX idx_payment_transaction_history_timestamp ON payment_transaction_history(timestamp);

-- Payment Retry Log (for failed transaction handling)
CREATE TABLE IF NOT EXISTS payment_retry_log (
    id TEXT PRIMARY KEY,
    transaction_id TEXT NOT NULL,
    attempt_number INTEGER NOT NULL,
    status TEXT NOT NULL,
    error_message TEXT,
    next_retry_at DATETIME,
    created_at DATETIME NOT NULL,
    FOREIGN KEY (transaction_id) REFERENCES payment_transactions(id)
);

CREATE INDEX idx_payment_retry_log_transaction_id ON payment_retry_log(transaction_id);
CREATE INDEX idx_payment_retry_log_attempt_number ON payment_retry_log(attempt_number);
CREATE INDEX idx_payment_retry_log_next_retry_at ON payment_retry_log(next_retry_at);
