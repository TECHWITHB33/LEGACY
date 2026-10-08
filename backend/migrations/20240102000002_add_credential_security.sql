-- Add credential security fields
-- This migration adds support for credential hashing, digital signatures, versioning, and lifecycle management

ALTER TABLE credentials ADD COLUMN IF NOT EXISTS version INTEGER NOT NULL DEFAULT 1;
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS schema_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS credential_hash TEXT NOT NULL DEFAULT '';
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS signature TEXT NOT NULL DEFAULT '';
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS signed_at TIMESTAMP;
ALTER TABLE credentials ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMP;

-- Update status enum to include 'suspended'
-- Note: For SQLite, we can't directly alter enums, so we rely on application-level validation

CREATE INDEX IF NOT EXISTS idx_credentials_status ON credentials(status);
CREATE INDEX IF NOT EXISTS idx_credentials_user_id_status ON credentials(user_id, status);
CREATE INDEX IF NOT EXISTS idx_credentials_created_at ON credentials(created_at);
