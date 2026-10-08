-- Payment Authentication Gateway Schema
-- Phase 5 Extension: Biometric authentication gateway for payment authorization
-- CRITICAL: All payments must pass through this gateway before Stellar submission

-- Payment Authorizations (gateway check results)
CREATE TABLE IF NOT EXISTS payment_gateway_authorizations (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT UNIQUE NOT NULL,
    student_user_id TEXT NOT NULL,
    authorized BOOLEAN NOT NULL,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (student_user_id) REFERENCES users(id)
);

CREATE INDEX idx_payment_gateway_auth_payment_id ON payment_gateway_authorizations(payment_request_id);
CREATE INDEX idx_payment_gateway_auth_user_id ON payment_gateway_authorizations(student_user_id);
CREATE INDEX idx_payment_gateway_auth_authorized ON payment_gateway_authorizations(authorized);
CREATE INDEX idx_payment_gateway_auth_expires_at ON payment_gateway_authorizations(expires_at);

-- Gateway Check Results (audit trail - each check recorded)
CREATE TABLE IF NOT EXISTS payment_gateway_checks (
    id TEXT PRIMARY KEY,
    authorization_id TEXT NOT NULL,
    check_name TEXT NOT NULL,
    passed BOOLEAN NOT NULL,
    failure_reason TEXT,
    checked_at DATETIME NOT NULL,
    FOREIGN KEY (authorization_id) REFERENCES payment_gateway_authorizations(id)
);

CREATE INDEX idx_payment_gateway_checks_auth_id ON payment_gateway_checks(authorization_id);
CREATE INDEX idx_payment_gateway_checks_name ON payment_gateway_checks(check_name);
CREATE INDEX idx_payment_gateway_checks_passed ON payment_gateway_checks(passed);

-- Credential Verification Cache (for payment flow)
CREATE TABLE IF NOT EXISTS payment_credential_cache (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL UNIQUE,
    credential_id TEXT NOT NULL,
    credential_status TEXT NOT NULL,
    verified BOOLEAN NOT NULL,
    verified_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id)
);

CREATE INDEX idx_payment_credential_cache_payment_id ON payment_credential_cache(payment_request_id);
CREATE INDEX idx_payment_credential_cache_credential_id ON payment_credential_cache(credential_id);
CREATE INDEX idx_payment_credential_cache_status ON payment_credential_cache(credential_status);

-- Biometric Challenge Cache (for payment authentication)
CREATE TABLE IF NOT EXISTS payment_biometric_challenges (
    id TEXT PRIMARY KEY,
    authorization_id TEXT NOT NULL,
    challenge_id TEXT NOT NULL UNIQUE,
    credential_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    signature_received TEXT,
    verified BOOLEAN,
    created_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    used_at DATETIME,
    FOREIGN KEY (authorization_id) REFERENCES payment_gateway_authorizations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_payment_biometric_challenges_auth_id ON payment_biometric_challenges(authorization_id);
CREATE INDEX idx_payment_biometric_challenges_challenge_id ON payment_biometric_challenges(challenge_id);
CREATE INDEX idx_payment_biometric_challenges_user_id ON payment_biometric_challenges(user_id);
CREATE INDEX idx_payment_biometric_challenges_verified ON payment_biometric_challenges(verified);

-- Anti-Replay Log (audit trail for replay detection)
CREATE TABLE IF NOT EXISTS payment_anti_replay_log (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL,
    recorded_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id)
);

CREATE INDEX idx_payment_anti_replay_log_payment_id ON payment_anti_replay_log(payment_request_id);
CREATE INDEX idx_payment_anti_replay_log_status ON payment_anti_replay_log(status);

-- Payment Policy Enforcement (daily limits per student)
CREATE TABLE IF NOT EXISTS payment_policies (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL UNIQUE,
    daily_limit REAL NOT NULL DEFAULT 500.0,
    daily_used REAL NOT NULL DEFAULT 0.0,
    reset_at DATETIME NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_payment_policies_user_id ON payment_policies(user_id);
CREATE INDEX idx_payment_policies_reset_at ON payment_policies(reset_at);

-- Payment Policy History (audit trail for limit changes)
CREATE TABLE IF NOT EXISTS payment_policy_history (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL,
    old_daily_limit REAL,
    new_daily_limit REAL,
    changed_by TEXT,
    changed_at DATETIME NOT NULL,
    FOREIGN KEY (policy_id) REFERENCES payment_policies(id)
);

CREATE INDEX idx_payment_policy_history_policy_id ON payment_policy_history(policy_id);
CREATE INDEX idx_payment_policy_history_changed_at ON payment_policy_history(changed_at);

-- Gateway Audit Log (comprehensive audit of all payment attempts)
CREATE TABLE IF NOT EXISTS payment_gateway_audit_log (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL,
    student_user_id TEXT NOT NULL,
    merchant_account TEXT NOT NULL,
    payment_amount TEXT NOT NULL,
    authorization_result BOOLEAN NOT NULL,
    failure_summary TEXT,
    total_checks INTEGER NOT NULL,
    passed_checks INTEGER NOT NULL,
    attempted_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id),
    FOREIGN KEY (student_user_id) REFERENCES users(id)
);

CREATE INDEX idx_payment_gateway_audit_log_payment_id ON payment_gateway_audit_log(payment_request_id);
CREATE INDEX idx_payment_gateway_audit_log_user_id ON payment_gateway_audit_log(student_user_id);
CREATE INDEX idx_payment_gateway_audit_log_result ON payment_gateway_audit_log(authorization_result);
CREATE INDEX idx_payment_gateway_audit_log_timestamp ON payment_gateway_audit_log(attempted_at);

-- Merchant Verification Cache (for payment flow)
CREATE TABLE IF NOT EXISTS payment_merchant_cache (
    id TEXT PRIMARY KEY,
    payment_request_id TEXT NOT NULL UNIQUE,
    merchant_account TEXT NOT NULL,
    verified BOOLEAN NOT NULL,
    verified_at DATETIME NOT NULL,
    FOREIGN KEY (payment_request_id) REFERENCES payment_requests(id)
);

CREATE INDEX idx_payment_merchant_cache_payment_id ON payment_merchant_cache(payment_request_id);
CREATE INDEX idx_payment_merchant_cache_merchant_account ON payment_merchant_cache(merchant_account);
CREATE INDEX idx_payment_merchant_cache_verified ON payment_merchant_cache(verified);

-- Constraints and Rules
-- 1. payment_gateway_authorizations.expires_at must be > created_at
-- 2. payment_gateway_checks.checked_at must be >= authorization_created_at
-- 3. payment_biometric_challenges.used_at must be after created_at or NULL
-- 4. payment_anti_replay_log.status must be one of: 'in_flight', 'completed', 'cancelled'
-- 5. payment_policies.daily_used must be <= daily_limit
-- 6. payment_policies.reset_at must be reset daily (typically at UTC midnight)
