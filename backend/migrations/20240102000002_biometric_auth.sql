-- Biometric Authentication Schema
-- Stores biometric credentials and challenges for WebAuthn

-- Biometric credentials table
CREATE TABLE IF NOT EXISTS biometric_credentials (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    public_key TEXT NOT NULL,              -- Base64-encoded public key (NEVER private key)
    credential_type TEXT NOT NULL,         -- fingerprint, face, platform
    device_id TEXT NOT NULL,               -- Device identifier
    device_name TEXT NOT NULL,             -- Human-readable device name
    sign_count INTEGER NOT NULL DEFAULT 0, -- Counter for replay protection
    status TEXT NOT NULL DEFAULT 'active', -- active, revoked, expired
    created_at DATETIME NOT NULL,
    expires_at DATETIME,                   -- Optional expiration
    last_used_at DATETIME,                 -- Last authentication time
    revoked_at DATETIME,                   -- Revocation time
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(user_id, device_id)
);

-- Index for user lookups
CREATE INDEX IF NOT EXISTS idx_biometric_user ON biometric_credentials(user_id);
CREATE INDEX IF NOT EXISTS idx_biometric_status ON biometric_credentials(status);
CREATE INDEX IF NOT EXISTS idx_biometric_device ON biometric_credentials(device_id);

-- Biometric challenges table
CREATE TABLE IF NOT EXISTS biometric_challenges (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    challenge TEXT NOT NULL,               -- Base64-encoded challenge
    challenge_type TEXT NOT NULL,          -- registration, authentication
    credential_id TEXT,                    -- For authentication challenges
    expires_at DATETIME NOT NULL,
    used BOOLEAN NOT NULL DEFAULT 0,       -- Prevents challenge reuse
    created_at DATETIME NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (credential_id) REFERENCES biometric_credentials(id)
);

-- Index for challenge lookups
CREATE INDEX IF NOT EXISTS idx_biometric_challenges_user ON biometric_challenges(user_id);
CREATE INDEX IF NOT EXISTS idx_biometric_challenges_credential ON biometric_challenges(credential_id);
CREATE INDEX IF NOT EXISTS idx_biometric_challenges_expires ON biometric_challenges(expires_at);
CREATE INDEX IF NOT EXISTS idx_biometric_challenges_used ON biometric_challenges(used);

-- IMPORTANT SECURITY NOTES:
-- 1. NO fingerprint_image, fingerprint_template, or raw biometric data columns
-- 2. NO face_image, face_template, or raw biometric data columns
-- 3. NO passwords or secrets stored alongside credentials
-- 4. public_key is cryptographic material only (Ed25519/ECDSA)
-- 5. Device performs all biometric matching locally
-- 6. Server only receives and verifies signatures
-- 7. challenge stores random bytes for signature input
-- 8. used flag prevents challenge reuse (replay protection)
-- 9. sign_count increments on each use (additional replay protection)
-- 10. expires_at ensures credential expiration
