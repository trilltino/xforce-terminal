-- Add wallet fields to users table
ALTER TABLE users ADD COLUMN wallet_address TEXT;
ALTER TABLE users ADD COLUMN wallet_connected_at DATETIME;
ALTER TABLE users ADD COLUMN wallet_setup_token TEXT;
ALTER TABLE users ADD COLUMN wallet_setup_token_expires_at DATETIME;

-- Create unique index on wallet_address
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_wallet_address ON users(wallet_address) WHERE wallet_address IS NOT NULL;
