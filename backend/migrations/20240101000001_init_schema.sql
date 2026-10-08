-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    institution TEXT NOT NULL,
    matric_number TEXT NOT NULL,
    age INTEGER NOT NULL,
    gender TEXT NOT NULL,
    department TEXT NOT NULL,
    phone_contact TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    public_key TEXT NOT NULL,
    private_key_encrypted TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create credentials table
CREATE TABLE IF NOT EXISTS credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key_hash TEXT NOT NULL,
    qr_code TEXT NOT NULL,
    qr_hash TEXT NOT NULL UNIQUE,
    status TEXT NOT NULL DEFAULT 'active',
    expires_at DATETIME,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create blockchain table
CREATE TABLE IF NOT EXISTS blockchain_blocks (
    id TEXT PRIMARY KEY,
    block_index INTEGER NOT NULL UNIQUE,
    previous_hash TEXT NOT NULL,
    hash TEXT NOT NULL UNIQUE,
    timestamp DATETIME NOT NULL,
    data TEXT NOT NULL,
    nonce INTEGER NOT NULL,
    difficulty INTEGER NOT NULL DEFAULT 1
);

-- Create blockchain records table
CREATE TABLE IF NOT EXISTS blockchain_records (
    id TEXT PRIMARY KEY,
    credential_id TEXT NOT NULL,
    block_index INTEGER NOT NULL,
    block_hash TEXT NOT NULL,
    verified BOOLEAN DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (credential_id) REFERENCES credentials(id),
    FOREIGN KEY (block_index) REFERENCES blockchain_blocks(block_index)
);

-- Create payments table
CREATE TABLE IF NOT EXISTS payments (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    amount TEXT NOT NULL,
    asset_code TEXT NOT NULL,
    recipient_address TEXT NOT NULL,
    merchant_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    transaction_hash TEXT,
    qr_code TEXT,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    action TEXT NOT NULL,
    resource_type TEXT NOT NULL,
    resource_id TEXT,
    details TEXT NOT NULL,
    ip_address TEXT,
    timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create biometric credentials table
CREATE TABLE IF NOT EXISTS biometric_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    credential_data TEXT NOT NULL,
    device_name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Create indexes
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_credentials_user_id ON credentials(user_id);
CREATE INDEX idx_credentials_qr_hash ON credentials(qr_hash);
CREATE INDEX idx_blockchain_hash ON blockchain_blocks(hash);
CREATE INDEX idx_payments_user_id ON payments(user_id);
CREATE INDEX idx_payments_status ON payments(status);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp);
